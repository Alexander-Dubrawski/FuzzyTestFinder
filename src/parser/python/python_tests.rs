use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::Path,
    time::UNIX_EPOCH,
};

use regex::Regex;
use rustpython_parser::{Mode, lexer::lex, parse_tokens};
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::parser::{Test, Tests};

fn is_hidden(entry: &DirEntry) -> bool {
    let hidden = entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false);
    hidden
}

fn collect_tests_from_file(path: &Path) -> HashSet<String> {
    let source_code = std::fs::read_to_string(path).unwrap();
    let tokens = lex(source_code.as_str(), Mode::Module);
    let ast = parse_tokens(tokens, Mode::Module, "<embedded>").unwrap();
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
    tests
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
    fn runtime_argument(self) -> String {
        format!("{}::{}\n", self.path, self.test)
    }

    fn search_item_name(&self) -> String {
        format!("{}::{}\n", self.path, self.test)
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

    pub fn filter_out_deleted_files(&mut self) -> bool {
        let mut tests_to_remove = vec![];
        for path in self.tests.keys() {
            if !Path::new(path).exists() {
                tests_to_remove.push(path.clone());
            }
        }
        let updated = tests_to_remove.len() > 0;
        tests_to_remove.into_iter().for_each(|test_path| {
            println!("Removed test file");
            self.tests.remove(&test_path);
        });
        updated
    }

    pub fn update(&mut self, only_check_for_change: bool) -> bool {
        let mut updated = false;
        for entry in WalkDir::new(self.root_folder.as_str())
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let metadata = std::fs::metadata(entry.path()).unwrap();
                if entry.path().extension().is_none() {
                    continue;
                }

                if entry.path().extension().and_then(OsStr::to_str).unwrap() != "py" {
                    continue;
                }

                let pattern = Regex::new(r"^(test_.*\.py|.*_test\.py)$").unwrap();
                if !pattern.is_match(entry.path().file_name().unwrap().to_str().unwrap()) {
                    continue;
                }

                let full_path = entry.path().as_os_str().to_str().unwrap();
                let relative_path = full_path
                    .strip_prefix(self.root_folder.as_str())
                    .unwrap()
                    .strip_prefix("/")
                    .unwrap();

                if let Ok(modified) = metadata.modified() {
                    if modified.duration_since(UNIX_EPOCH).unwrap().as_millis() > self.timestamp {
                        // println!("Modified: {:?}", entry.path());
                        // println!("{}", relative_path);
                        let new_tests = collect_tests_from_file(entry.path());
                        if !self.tests.contains_key(relative_path) {
                            updated = true;
                            self.tests.insert(relative_path.to_string(), new_tests);
                            continue;
                        }
                        if new_tests != self.tests[relative_path] {
                            if only_check_for_change {
                                return true;
                            }
                            updated = true;
                            println!(
                                "Tests updated: {}",
                                entry.path().as_os_str().to_str().unwrap()
                            );
                            let entry = self.tests.get_mut(relative_path).unwrap();
                            *entry = new_tests;
                        }
                    }
                }
                if let Ok(created) = metadata.created() {
                    if created.duration_since(UNIX_EPOCH).unwrap().as_millis() > self.timestamp {
                        // println!("New file: {:?}", entry.path());
                        let new_tests = collect_tests_from_file(entry.path());
                        if !new_tests.is_empty() {
                            // println!("New tests found");
                            if only_check_for_change {
                                return true;
                            }
                            self.tests.insert(relative_path.to_string(), new_tests);
                            updated = true;
                        }
                    }
                }
            }
        }
        updated
    }
}

impl Tests for PythonTests {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
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

    fn update(&mut self, only_check_for_update: bool) -> bool {
        self.update(only_check_for_update)
    }
}
