use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    tests::{
        Test, Tests,
        rust::{ParseRustTest, mod_resolver::get_module_paths, rust_test_parser::RustTestParser},
    },
    utils::path_resolver::get_relative_path,
};

use super::helper::parse_failed_tests;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RustTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub timestamp_coverage: u128,
    pub tests: HashMap<String, Vec<RustTest>>,
    pub failed_tests: HashMap<String, Vec<RustTest>>,
    // TODO
    // When structure exist, only update if:
    // - File changes (re run corresponding tests)
    // - Test change (re-run tests) Should be done from outside and coverage report should be used to update this struct
    // - New file added (rerun all changed tests, or added once, if another file calls the new file it was changes and by that we re-run the tests)
    // The order should be:
    // 1. Check for changed tests
    // 2. Check for new tests
    // 3. Check for changed files
    // Return test set of these test and get coverage from runtime and update this struct
    //
    // If this struct is empty, all tests should be returned, but then all test should be new
    // add update file_coverage to trait
    pub file_coverage: HashMap<String, CoverageRustTests>,
}

impl RustTests {
    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            timestamp_coverage: 0,
            tests: HashMap::new(),
            failed_tests: HashMap::new(),
            file_coverage: HashMap::new(),
        }
    }

    fn update_tests<P: ParseRustTest>(&mut self, parser: &P) -> Result<bool, FztError> {
        let cargo_tests = parser.parse_tests()?;
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
        let updated = if !up_to_date {
            self.refill_tests(cargo_tests)?;
            self.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
            true
        } else {
            // Filter out old tests
            let cargo_test_set: HashSet<&(Vec<String>, String)> =
                HashSet::from_iter(cargo_tests.iter());
            for (_, rust_tests) in self.tests.iter_mut() {
                *rust_tests = rust_tests
                    .into_iter()
                    .filter(|rust_test| {
                        cargo_test_set.contains(&(
                            rust_test.module_path.clone(),
                            rust_test.method_name.clone(),
                        ))
                    })
                    .map(|rust_test| rust_test.clone())
                    .collect();
            }
            false
        };
        self.failed_tests
            .retain(|path, _| self.tests.contains_key(path));
        self.failed_tests
            .iter_mut()
            .for_each(|(path, failed_tests)| {
                let tests = self
                    .tests
                    .get(path)
                    .expect("THIS IS A BUG. Failed tests should be a subset of tests");
                failed_tests.retain(|test| tests.contains(test));
            });
        Ok(updated)
    }

    fn refill_tests(&mut self, cargo_tests: Vec<(Vec<String>, String)>) -> Result<(), FztError> {
        let mut path = Path::new(&self.root_folder).to_path_buf();
        if path.join("src").exists() {
            path = path.join("src");
        }
        if path.join("lib.rs").exists() {
            path = path.join("lib.rs");
        } else if path.join("mod.rs").exists() {
            path = path.join("mod.rs");
        } else if path.join("main.rs").exists() {
            path = path.join("main.rs");
        } else {
            return Err(FztError::GeneralParsingError(format!(
                "No valid Rust source file found in {:?}",
                path
            )));
        }
        let module_paths = get_module_paths(&path)?;
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
            let relative_path = get_relative_path(&self.root_folder, &path)?;
            let entry = updated_tests.get_mut(&relative_path);
            match entry {
                Some(tests) => {
                    tests.push(rust_test);
                }
                None => {
                    updated_tests.insert(relative_path, vec![rust_test]);
                }
            }
        }
        self.tests = updated_tests;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RustTest {
    pub module_path: Vec<String>,
    pub method_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct CoverageRustTests {
    pub path: String,
    pub tests: Vec<RustTest>,
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

    fn file_path(&self) -> String {
        self.path.clone()
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
        self.update_tests(&RustTestParser::default())
    }

    fn update_failed(&mut self, runtime_output: &str) -> bool {
        let failed_tests = parse_failed_tests(runtime_output, &self.tests);
        if self.failed_tests == failed_tests {
            false
        } else {
            self.failed_tests = failed_tests;
            true
        }
    }

    fn tests_failed(&self) -> Vec<impl Test> {
        self.failed_tests
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

    fn update_file_coverage(
        &mut self,
        coverage: &HashMap<String, Vec<String>>,
    ) -> Result<bool, FztError> {
        // resolve cargo runtime to test item
        // Refactor refill_tests to use its logic to map to Rust test

        // Reset
        //self.changed_test_files_since_last_coverage = HashMap::new();

        // merge coverage with existing coverage
        todo!()
    }

    fn get_covered_tests(&mut self) -> Vec<impl Test> {
        // Get changed tests:
        let mut coverage_tests: Vec<RustTestItem> = self
            .tests
            .iter()
            .filter(|(path, tests)| {
                // consider changed or new test files
                std::fs::metadata(path.as_str())
                    .unwrap()
                    .modified()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    > self.timestamp_coverage
            })
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
            .collect();

        self.file_coverage.iter().for_each(|(path, cov_tests)| {
            if std::fs::metadata(path.as_str())
                .unwrap()
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                > self.timestamp
            {
                cov_tests.tests.iter().for_each(|test| {
                    coverage_tests.push(RustTestItem::new(
                        cov_tests.path.clone(),
                        test.module_path.join("::"),
                        test.method_name.clone(),
                    ));
                });
            }
        });
        coverage_tests
    }
}

#[cfg(test)]
mod tests {

    use crate::tests::rust::{ParseRustTest, rust_test::RustTest};

    struct MockCargoTest {
        pub tests: Vec<(Vec<String>, String)>,
    }

    impl MockCargoTest {
        // add code here
    }

    impl ParseRustTest for MockCargoTest {
        fn parse_tests(&self) -> Result<Vec<(Vec<String>, String)>, crate::errors::FztError> {
            Ok(self.tests.clone())
        }
    }

    #[test]
    fn parse_tests() {
        let initial_tests = vec![(
            vec!["a".to_string(), "test_one".to_string()],
            "one".to_string(),
        )];
        let mut mock_parser = MockCargoTest {
            tests: initial_tests.clone(),
        };
        let mut rust_tests =
            super::RustTests::new_empty("src/tests/rust/test_data/tests".to_string());

        let expected = RustTest {
            module_path: vec!["a".to_string(), "test_one".to_string()],
            method_name: "one".to_string(),
        };

        assert!(rust_tests.update_tests(&mock_parser).unwrap());

        assert_eq!(
            rust_tests.tests.get("a/test_one.rs").unwrap(),
            &vec![expected]
        );

        assert!(!rust_tests.update_tests(&mock_parser).unwrap());

        let updated_tests = vec![
            (
                vec!["a".to_string(), "test_one".to_string()],
                "one".to_string(),
            ),
            (
                vec!["a".to_string(), "test_two".to_string()],
                "twoOne".to_string(),
            ),
            (
                vec!["a".to_string(), "test_two".to_string()],
                "two".to_string(),
            ),
            (
                vec!["b".to_string(), "test_three".to_string()],
                "three".to_string(),
            ),
        ];

        mock_parser = MockCargoTest {
            tests: updated_tests.clone(),
        };

        let expected = vec![
            (
                "a/test_one.rs",
                vec![RustTest {
                    module_path: vec!["a".to_string(), "test_one".to_string()],
                    method_name: "one".to_string(),
                }],
            ),
            (
                "a/test_two.rs",
                vec![
                    RustTest {
                        module_path: vec!["a".to_string(), "test_two".to_string()],
                        method_name: "twoOne".to_string(),
                    },
                    RustTest {
                        module_path: vec!["a".to_string(), "test_two".to_string()],
                        method_name: "two".to_string(),
                    },
                ],
            ),
            (
                "b/test_three.rs",
                vec![RustTest {
                    module_path: vec!["b".to_string(), "test_three".to_string()],
                    method_name: "three".to_string(),
                }],
            ),
        ];

        assert!(rust_tests.update_tests(&mock_parser).unwrap());

        for (path, mut expected) in expected.into_iter() {
            expected.sort();
            let mut result = rust_tests.tests.get(path).unwrap().clone();
            result.sort();
            assert_eq!(result, expected)
        }

        let remove_test = vec![
            (
                vec!["a".to_string(), "test_one".to_string()],
                "one".to_string(),
            ),
            (
                vec!["a".to_string(), "test_two".to_string()],
                "twoOne".to_string(),
            ),
            (
                vec!["b".to_string(), "test_three".to_string()],
                "three".to_string(),
            ),
        ];
        mock_parser = MockCargoTest {
            tests: remove_test.clone(),
        };

        // Tests is removed, but module path is still valid
        // so cache can be used
        assert!(!rust_tests.update_tests(&mock_parser).unwrap());

        let expected = vec![
            (
                "a/test_one.rs",
                vec![RustTest {
                    module_path: vec!["a".to_string(), "test_one".to_string()],
                    method_name: "one".to_string(),
                }],
            ),
            (
                "a/test_two.rs",
                vec![RustTest {
                    module_path: vec!["a".to_string(), "test_two".to_string()],
                    method_name: "twoOne".to_string(),
                }],
            ),
            (
                "b/test_three.rs",
                vec![RustTest {
                    module_path: vec!["b".to_string(), "test_three".to_string()],
                    method_name: "three".to_string(),
                }],
            ),
        ];

        for (path, mut expected) in expected.into_iter() {
            expected.sort();
            let mut result = rust_tests.tests.get(path).unwrap().clone();
            result.sort();
            assert_eq!(result, expected)
        }
    }
}
