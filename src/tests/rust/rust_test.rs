use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    tests::{
        Test, Tests,
        rust::{ParseRustTest, mod_resolver::get_module_paths, rust_test_parser::RustTestParser},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RustTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, Vec<RustTest>>,
}

impl RustTests {
    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
        }
    }

    fn update_test(&mut self, cargo_tests: Vec<(Vec<String>, String)>) -> Result<(), FztError> {
        // TODO: check if `lib.rs` or `mod.rs` exists
        let module_paths = get_module_paths(&Path::new(&self.root_folder).join("lib.rs"))?;
        let mut updated_tests: HashMap<String, Vec<RustTest>> = HashMap::new();
        for (module_path, method_name) in cargo_tests.into_iter() {
            let test_path = module_paths
                .get(&module_path)
                .ok_or(FztError::GeneralParsingError(format!(
                    "No module path found for test: {:?} -> {}",
                    module_path, method_name
                )))?;
            let path = test_path.to_str().expect("Path needs to exist").to_string();
            let rust_test = RustTest {
                module_path: module_path,
                method_name,
            };
            let entry = updated_tests.get_mut(&path);
            match entry {
                Some(tests) => {
                    tests.push(rust_test);
                }
                None => {
                    updated_tests.insert(path, vec![rust_test]);
                }
            }
        }
        self.tests = updated_tests;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RustTest {
    pub module_path: Vec<String>,
    pub method_name: String,
}

pub struct RustTestItem {
    pub path: String,
    pub module_path: String,
    pub test: String,
}

impl RustTestItem {
    pub fn new(path: String, module_path: String, test: String) -> Self {
        Self {
            path,
            module_path,
            test,
        }
    }
}

impl Test for RustTestItem {
    fn runtime_argument(&self) -> String {
        format!("{}::{}", self.module_path, self.test)
    }

    fn name(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }
}

impl Tests for RustTests {
    fn to_json(&self) -> Result<String, FztError> {
        serde_json::to_string(&self).map_err(FztError::from)
    }

    fn tests(&self) -> Vec<impl Test> {
        self.tests
            .iter()
            .map(|(path, tests)| {
                tests
                    .iter()
                    .map(|test| {
                        RustTestItem::new(
                            path.clone(),
                            test.module_path.join("::"),
                            test.method_name.clone(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    fn update(&mut self) -> Result<bool, FztError> {
        let cargo_tests = RustTestParser::parse_tests()?;
        let mut up_to_date = true;
        for (module_path, method_name) in cargo_tests.iter() {
            let contains = self.tests.values().any(|tests| {
                tests.iter().any(|test| {
                    &test.module_path == module_path && method_name == &test.method_name
                })
            });
            if !contains {
                up_to_date = false;
                break;
            }
        }
        if !up_to_date {
            self.update_test(cargo_tests)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// Filter for test
// Build module_path_tree
// Get file name.
// Informational like line number can still be collected since we have meta information (Before doing that validate that what is needed to open a preview windoe in fzf)
// pub fn update_tests(
//     root_folder: &str,
//     timestamp: &mut u128,
//     tests: &mut HashMap<String, HashSet<String>>,
//     only_check_for_change: bool,
// ) -> Result<bool, FztError> {

// }

// #[cfg(test)]
// mod tests {
//     use std::path::Path;

//     use super::collect_tests_from_file;

//     #[test]
//     fn foo() {
//         //collect_tests_from_file(&Path::new("src/tests/rust/test_data/b/test_three.rs"));
//     }
// }
