use std::{collections::HashSet, ffi::OsStr, path::Path, time::UNIX_EPOCH};

use rustpython_parser::{Mode, lexer::lex, parse_tokens};
use walkdir::WalkDir;

use crate::cache::types::CacheUpdate;

#[derive(Default)]
pub struct PytestUpdate {}

impl PytestUpdate {
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

    fn check_file_for_new_tests(path: &Path, tests: &HashSet<String>) -> bool {
        let collected_tests = Self::collect_tests_from_file(path);
        &collected_tests != tests
    }
}

impl CacheUpdate for PytestUpdate {
    fn update(&self, cache_entry: &mut crate::cache::types::CacheEntry) -> bool {
        for entry in WalkDir::new(cache_entry.root_folder.as_str()) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let metadata = std::fs::metadata(entry.path()).unwrap();
                if entry.path().extension().is_none() {
                    continue;
                }

                if entry.path().extension().and_then(OsStr::to_str).unwrap() != "py" {
                    continue;
                }

                // TODO: Check if modified, that if modified, that there are new/removed functions
                // If so Cache entry can be replaced in place
                if let Ok(modified) = metadata.modified() {
                    if modified.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        > cache_entry.timestamp
                    {
                        println!("Modified: {:?}", entry.path());
                        // TODO: tests should
                        let full_path = entry.path().as_os_str().to_str().unwrap();
                        let start_dir = Path::new(cache_entry.root_folder.as_str())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();
                        let relative_path = format!(
                            "{}{}",
                            start_dir,
                            full_path
                                .strip_prefix(cache_entry.root_folder.as_str())
                                .unwrap()
                        );
                        if Self::check_file_for_new_tests(
                            entry.path(),
                            &cache_entry.tests[&relative_path],
                        ) {
                            println!("New tests found");
                            return true;
                        }
                    }
                }
                // TODO: Check if created, that the file includes test function
                if let Ok(created) = metadata.created() {
                    if created.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        > cache_entry.timestamp
                    {
                        println!("New file: {:?}", entry.path());
                        if !Self::collect_tests_from_file(entry.path()).is_empty() {
                            println!("New tests found");
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
