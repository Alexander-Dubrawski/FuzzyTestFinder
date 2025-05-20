use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use regex::Regex;
use rustpython_parser::{Mode, lexer::lex, parse_tokens};
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::{
    errors::FztError,
    parser::{Test, Tests},
};

fn is_hidden(entry: &DirEntry) -> bool {
    let hidden = entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false);
    hidden
}

fn collect_tests_from_file(path: &Path) -> Result<HashSet<String>, FztError> {
    let source_code = std::fs::read_to_string(path)?;
    let tokens = lex(source_code.as_str(), Mode::Module);
    let ast = parse_tokens(tokens, Mode::Module, "<embedded>")?;
    let mut tests = HashSet::new();
    match ast {
        rustpython_parser::ast::Mod::Module(mod_module) => {
            for stmt in mod_module.body.iter() {
                match stmt {
                    rustpython_parser::ast::Stmt::FunctionDef(stmt_function_def) => {
                        let test_name = stmt_function_def.name.to_string();
                        if test_name.starts_with("test") {
                            tests.insert(stmt_function_def.name.to_string());
                        }
                    }
                    _ => continue,
                }
            }
        }
        _ => todo!(),
    }
    Ok(tests)
}

pub struct PythonTest {
    path: String,
    test: String,
}

impl PythonTest {
    pub fn new(path: String, test: String) -> Self {
        Self { path, test }
    }
}

impl Test for PythonTest {
    fn runtime_argument(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }

    fn search_item_name(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<String>>,
}

impl PythonTests {
    pub fn new(
        root_folder: String,
        timestamp: u128,
        tests: HashMap<String, HashSet<String>>,
    ) -> Self {
        Self {
            root_folder,
            timestamp,
            tests,
        }
    }

    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
        }
    }

    pub fn filter_out_deleted_files(&mut self) -> bool {
        let mut tests_to_remove = vec![];
        for path in self.tests.keys() {
            if !Path::new(path).exists() {
                tests_to_remove.push(path.clone());
            }
        }
        let updated = tests_to_remove.len() > 0;
        tests_to_remove.into_iter().for_each(|test_path| {
            self.tests.remove(&test_path);
        });
        updated
    }

    pub fn update_tests(&mut self, only_check_for_change: bool) -> Result<bool, FztError> {
        let mut updated = false;
        for entry in WalkDir::new(self.root_folder.as_str())
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                let metadata = std::fs::metadata(entry.path())?;
                if entry.path().extension().is_none() {
                    continue;
                }

                if entry
                    .path()
                    .extension()
                    .and_then(OsStr::to_str)
                    .expect("Is file type")
                    != "py"
                {
                    continue;
                }

                let pattern = Regex::new(r"^(test_.*\.py|.*_test\.py)$")?;
                if !pattern.is_match(
                    entry
                        .path()
                        .file_name()
                        .expect("Is file type")
                        .to_str()
                        .expect("Is file type"),
                ) {
                    continue;
                }

                let full_path = entry.path().as_os_str().to_str().expect("Is file type");
                let relative_path = full_path
                    .strip_prefix(self.root_folder.as_str())
                    .map(|path| path.strip_prefix("/"))
                    .flatten()
                    .ok_or(FztError::GeneralParsingError(format!(
                        "File path could not be parsed: {}",
                        full_path
                    )))?;

                if let Ok(modified) = metadata.modified() {
                    if modified.duration_since(UNIX_EPOCH)?.as_millis() > self.timestamp {
                        let new_tests = collect_tests_from_file(entry.path())?;
                        if !self.tests.contains_key(relative_path) {
                            updated = true;
                            self.tests.insert(relative_path.to_string(), new_tests);
                            continue;
                        }
                        if new_tests != self.tests[relative_path] {
                            if only_check_for_change {
                                return Ok(true);
                            }
                            updated = true;
                            println!(
                                "Tests updated: {}",
                                entry.path().as_os_str().to_str().expect("Is file type")
                            );
                            let entry = self.tests.get_mut(relative_path).expect("contains key");
                            *entry = new_tests;
                        }
                    }
                }
                if let Ok(created) = metadata.created() {
                    if created.duration_since(UNIX_EPOCH)?.as_millis() > self.timestamp {
                        let new_tests = collect_tests_from_file(entry.path())?;
                        if !new_tests.is_empty() {
                            if only_check_for_change {
                                return Ok(true);
                            }
                            self.tests.insert(relative_path.to_string(), new_tests);
                            updated = true;
                        }
                    }
                }
            }
        }
        if updated {
            self.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        }
        Ok(updated)
    }
}

impl Tests for PythonTests {
    fn to_json(&self) -> Result<String, FztError> {
        serde_json::to_string(&self).map_err(FztError::from)
    }

    fn tests(self) -> Vec<impl Test> {
        let mut output = vec![];
        self.tests.into_iter().for_each(|(path, tests)| {
            tests.into_iter().for_each(|test| {
                output.push(PythonTest::new(path.clone(), test));
            });
        });
        output
    }

    fn update(&mut self, only_check_for_update: bool) -> Result<bool, FztError> {
        let updated = self.filter_out_deleted_files();
        if only_check_for_update && updated {
            Ok(true)
        } else {
            Ok(self.update_tests(only_check_for_update)? || updated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn collect_tests() {
        let mut path = std::env::current_dir().unwrap();
        path.push("src/parser/python/test_data");
        let path_str = path.to_string_lossy();
        let mut pytest = PythonTests::new_empty(path_str.to_string());
        let mut expected_tests: HashMap<String, HashSet<String>> = HashMap::new();
        expected_tests.insert(
            "berlin/berlin_test.py".to_string(),
            HashSet::from_iter(vec!["test_berlin"].into_iter().map(|v| v.to_string())),
        );
        expected_tests.insert(
            "berlin/hamburg/test_hamburg.py".to_string(),
            HashSet::from_iter(
                vec!["test_hamburg", "test_hamburg_harburg"]
                    .into_iter()
                    .map(|v| v.to_string()),
            ),
        );
        expected_tests.insert(
            "berlin/potsdam/potsdam_test.py".to_string(),
            HashSet::from_iter(vec!["test_potsdam"].into_iter().map(|v| v.to_string())),
        );

        assert!(pytest.update_tests(false).unwrap());

        assert_eq!(pytest.tests, expected_tests);

        assert!(!pytest.update_tests(true).unwrap());
    }
}
