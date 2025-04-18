use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use regex::Regex;
use rustpython_parser::{Mode, lexer::lex, parse_tokens};
use walkdir::{DirEntry, WalkDir};

use crate::{cache::types::CacheEntry, parser::Parser};

use super::types::PyTests;

fn is_hidden(entry: &DirEntry) -> bool {
    let hidden = entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false);
    hidden
}

#[derive(Default)]
pub struct RpParser {
    // absolute path
    root_dir: String,
}

impl RpParser {
    pub fn new(root_dir: String) -> Self {
        Self { root_dir }
    }

    // TODO: Refactor also in pytest parser
    fn get_cache_entry(&self, python_tests: PyTests) -> CacheEntry {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        CacheEntry::new(self.root_dir.clone(), timestamp, python_tests.tests)
    }

    // TODO: Refactor same as in pytest parser
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

    fn collect_test(&self) -> PyTests {
        let mut tests = HashMap::new();
        for entry in WalkDir::new(&self.root_dir)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
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
                let relative_path = full_path.strip_prefix(&self.root_dir).unwrap();
                let collected_tests = Self::collect_tests_from_file(entry.path());
                tests.insert(relative_path.to_string(), collected_tests);
            }
        }
        PyTests::new(tests)
    }

    fn update_in_place(cache_entry: &mut crate::cache::types::CacheEntry) -> bool {
        let mut updated = false;
        for entry in WalkDir::new(cache_entry.root_folder.as_str())
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
                    .strip_prefix(cache_entry.root_folder.as_str())
                    .unwrap();
                if let Ok(modified) = metadata.modified() {
                    if modified.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        > cache_entry.timestamp
                    {
                        println!("Modified: {:?}", entry.path());

                        let new_tests = Self::collect_tests_from_file(entry.path());
                        if new_tests != cache_entry.tests[relative_path] {
                            updated = true;
                            println!(
                                "New tests found: {}",
                                entry.path().as_os_str().to_str().unwrap()
                            );
                            let entry = cache_entry.tests.get_mut(relative_path).unwrap();
                            *entry = new_tests;
                        }
                    }
                }
                if let Ok(created) = metadata.created() {
                    if created.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        > cache_entry.timestamp
                    {
                        println!("New file: {:?}", entry.path());
                        let new_tests = Self::collect_tests_from_file(entry.path());
                        if !new_tests.is_empty() {
                            println!("New tests found");
                            cache_entry
                                .tests
                                .insert(relative_path.to_string(), new_tests);
                            updated = true;
                        }
                    }
                }
            }
        }
        updated
    }
}

impl Parser for RpParser {
    fn parse_test(&self) -> crate::cache::types::CacheEntry {
        self.get_cache_entry(self.collect_test())
    }

    fn update_tests(&self, cache_entry: &mut crate::cache::types::CacheEntry) -> bool {
        Self::update_in_place(cache_entry)
    }
}
