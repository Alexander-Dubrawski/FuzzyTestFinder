use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    tests::{
        Test, Tests,
        python::{
            helper::{parse_filed_tests, update_tests},
            python_test::PythonTest,
        },
    },
    utils::file_walking::filter_out_deleted_files,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RustPytonTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<String>>,
    pub failed_tests: HashMap<String, HashSet<String>>,
}

impl RustPytonTests {
    pub fn new(
        root_folder: String,
        timestamp: u128,
        tests: HashMap<String, HashSet<String>>,
    ) -> Self {
        Self {
            root_folder,
            timestamp,
            tests,
            failed_tests: HashMap::new(),
        }
    }

    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
            failed_tests: HashMap::new(),
        }
    }
}

impl Tests for RustPytonTests {
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
        let files_filtered_out = filter_out_deleted_files(&self.root_folder, &mut self.tests);
        let updated = update_tests(
            self.root_folder.as_str(),
            &mut self.timestamp,
            &mut self.tests,
            false,
        )?;
        Ok(updated || files_filtered_out)
    }

    fn update_failed(&mut self, runtime_output: &str) -> bool {
        let failed_tests = parse_filed_tests(runtime_output);
        if self.failed_tests == failed_tests {
            false
        } else {
            self.failed_tests = failed_tests;
            true
        }
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
        let mut rust_pyton_tests = RustPytonTests::new_empty(test_path.to_string());
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
