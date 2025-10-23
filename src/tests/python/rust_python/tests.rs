use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    runtime::FailedTest,
    tests::{
        Test, Tests,
        python::{
            helper::{parse_failed_tests, update_tests},
            python_test::PythonTest,
        },
    },
    utils::{file::get_file_modification_timestamp, file_walking::filter_out_deleted_files},
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CoverageRustPythonTests {
    pub path: String,
    pub tests: HashSet<PythonTest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustPythonTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub timestamp_coverage: u128,
    pub tests: HashMap<String, HashSet<String>>,
    pub failed_tests: HashMap<String, HashSet<String>>,
    pub file_coverage: HashMap<String, CoverageRustPythonTests>,
    pub uncovered_tests: HashSet<PythonTest>,
}

impl RustPythonTests {
    pub fn new(
        root_folder: String,
        timestamp: u128,
        timestamp_coverage: u128,
        tests: HashMap<String, HashSet<String>>,
    ) -> Self {
        Self {
            root_folder,
            timestamp,
            timestamp_coverage,
            tests,
            failed_tests: HashMap::new(),
            file_coverage: HashMap::new(),
            uncovered_tests: HashSet::new(),
        }
    }

    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            timestamp_coverage: 0,
            tests: HashMap::new(),
            failed_tests: HashMap::new(),
            file_coverage: HashMap::new(),
            uncovered_tests: HashSet::new(),
        }
    }

    fn update_tests(&mut self) -> Result<bool, FztError> {
        let files_filtered_out = filter_out_deleted_files(&self.root_folder, &mut self.tests);
        let updated_tests = update_tests(
            self.root_folder.as_str(),
            &mut self.timestamp,
            &mut self.tests,
            false,
        )?;
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
        for (path, tests) in self.failed_tests.iter_mut() {
            tests.retain(|test| {
                self.tests
                    .get(path)
                    .map_or(false, |existing_tests| existing_tests.contains(test))
            });
        }
        Ok(updated_tests || files_filtered_out)
    }

    fn update_uncovered_tests(&mut self) {
        self.file_coverage
            .retain(|path, _| Path::new(path).exists());
        self.uncovered_tests
            .retain(|test| Path::new(&test.path).exists());
        let mut coverage_tests: Vec<PythonTest> = self
            .tests
            .iter()
            .filter(|(path, _)| {
                get_file_modification_timestamp(path.as_str()) > self.timestamp_coverage
            })
            .map(|(path, tests)| {
                tests
                    .iter()
                    .map(|test| PythonTest {
                        path: path.clone(),
                        test: test.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        self.file_coverage.iter().for_each(|(path, cov_tests)| {
            if get_file_modification_timestamp(path.as_str()) > self.timestamp_coverage {
                cov_tests.tests.iter().for_each(|test| {
                    coverage_tests.push(test.clone());
                });
            }
        });
        self.uncovered_tests.extend(coverage_tests);
        self.timestamp_coverage = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System clock may have gone backwards")
            .as_millis();
    }
}

impl Tests for RustPythonTests {
    fn to_json(&self) -> Result<String, FztError> {
        serde_json::to_string(&self).map_err(FztError::from)
    }

    fn tests(&self) -> Vec<impl Test> {
        let mut output = vec![];
        self.tests.iter().for_each(|(path, tests)| {
            tests.iter().for_each(|test| {
                output.push(PythonTest::new(path.clone(), test.clone()));
            });
        });
        output
    }

    fn update(&mut self) -> Result<bool, FztError> {
        let updated = self.update_tests()?;
        // For now we can not handle the path mocking, to not include all files
        // in this project
        #[cfg(not(test))]
        {
            self.update_uncovered_tests();
        }
        Ok(updated)
    }

    fn update_failed(&mut self, failed_tests_output: &[FailedTest]) -> bool {
        let failed_tests = parse_failed_tests(failed_tests_output);
        if self.failed_tests == failed_tests {
            false
        } else {
            self.failed_tests = failed_tests;
            true
        }
    }

    fn tests_failed(&self) -> Vec<impl Test> {
        let mut output = vec![];
        self.failed_tests.iter().for_each(|(path, tests)| {
            tests.iter().for_each(|test| {
                output.push(PythonTest::new(path.clone(), test.clone()));
            });
        });
        output
    }

    fn update_file_coverage(
        &mut self,
        coverage: &HashMap<String, Vec<String>>,
    ) -> Result<bool, FztError> {
        let mut updated = false;
        for (relative_path, tests) in coverage.iter() {
            let entry = self.file_coverage.get_mut(relative_path);
            match entry {
                Some(cov_tests) => {
                    tests.iter().try_for_each(|test| {
                        updated = true;
                        let item = PythonTest::try_from_pytest_test(test)?;
                        self.uncovered_tests.remove(&item);
                        cov_tests.tests.insert(item);
                        Ok::<(), FztError>(())
                    })?;
                }
                None => {
                    updated = true;
                    let tests = HashSet::from_iter(
                        tests
                            .iter()
                            .map(|test| {
                                let item = PythonTest::try_from_pytest_test(test)?;
                                self.uncovered_tests.remove(&item);
                                Ok(item)
                            })
                            .collect::<Result<Vec<PythonTest>, FztError>>()?
                            .into_iter(),
                    );
                    let cov_tests = CoverageRustPythonTests {
                        path: relative_path.clone(),
                        tests,
                    };
                    self.file_coverage
                        .insert(relative_path.to_string(), cov_tests);
                }
            }
        }

        self.file_coverage
            .retain(|path, _| Path::new(path).exists());
        self.timestamp_coverage = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        Ok(updated)
    }

    fn get_covered_tests(&self) -> Vec<impl Test> {
        self.uncovered_tests.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::copy_dict;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn collect_tests() {
        let mut path = std::env::current_dir().unwrap();
        path.push("src/tests/python/test_data");
        let (_temp_dir, dir_path) = copy_dict(path.as_path()).unwrap();
        let test_path = dir_path.as_path().to_str().unwrap();
        let mut rust_pyton_tests = RustPythonTests::new_empty(test_path.to_string());
        let mut expected = vec![
            PythonTest::new(
                "berlin/berlin_test.py".to_string(),
                "test_berlin".to_string(),
            ),
            PythonTest::new(
                "berlin/hamburg/test_hamburg.py".to_string(),
                "test_hamburg".to_string(),
            ),
            PythonTest::new(
                "berlin/hamburg/test_hamburg.py".to_string(),
                "test_hamburg_harburg".to_string(),
            ),
            PythonTest::new(
                "berlin/potsdam/potsdam_test.py".to_string(),
                "test_potsdam".to_string(),
            ),
        ];
        expected.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert!(rust_pyton_tests.update().unwrap());
        let mut results = rust_pyton_tests.tests();
        results.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert_eq!(results.len(), expected.len());

        for (res, exp) in results.iter().zip(expected.iter()) {
            assert_eq!(res.runtime_argument(), exp.runtime_argument());
            assert_eq!(res.name(), exp.name());
        }

        drop(results);

        // Remove test
        std::fs::remove_file(format!("{test_path}/berlin/potsdam/potsdam_test.py")).unwrap();
        expected = vec![
            PythonTest::new(
                "berlin/berlin_test.py".to_string(),
                "test_berlin".to_string(),
            ),
            PythonTest::new(
                "berlin/hamburg/test_hamburg.py".to_string(),
                "test_hamburg".to_string(),
            ),
            PythonTest::new(
                "berlin/hamburg/test_hamburg.py".to_string(),
                "test_hamburg_harburg".to_string(),
            ),
        ];
        expected.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert!(rust_pyton_tests.update().unwrap());
        let mut results = rust_pyton_tests.tests();
        results.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert_eq!(results.len(), expected.len());

        for (res, exp) in results.iter().zip(expected.iter()) {
            assert_eq!(res.runtime_argument(), exp.runtime_argument());
            assert_eq!(res.name(), exp.name());
        }
    }
}
