use std::{
    collections::{HashMap, HashSet},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    errors::FztError,
    tests::{
        Test, Tests,
        python::{
            helper::{parse_failed_tests, update_tests},
            python_test::PythonTest,
        },
    },
    utils::file_walking::filter_out_deleted_files,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str;

fn get_pytests() -> Result<String, FztError> {
    let binding = Command::new("python")
        .arg("-m")
        .arg("pytest")
        .arg("--co")
        .arg("-q")
        .output()
        .expect("failed to retrieve python tests");
    if binding.status.success() == false {
        let err = std::str::from_utf8(binding.stderr.as_slice())
            .map(|out| out.to_string())
            .map_err(FztError::from)?;
        return Err(FztError::PythonError(format!(
            "Failed to run `python -m pytest --co -q`\n{err}"
        )));
    }
    str::from_utf8(binding.stdout.as_slice())
        .map(|out| out.to_string())
        .map_err(FztError::from)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PytestTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<String>>,
    pub failed_tests: HashMap<String, HashSet<String>>,
}

impl PytestTests {
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

    fn parse_python_tests(&mut self, pytest_output: &str) -> Result<(), FztError> {
        let mut py_tests: HashMap<String, HashSet<String>> = HashMap::new();
        for line in pytest_output.lines() {
            if line.is_empty() {
                break;
            }
            let (path, test_name) = line
                .split("::")
                .collect_tuple()
                .map(|(path, test)| {
                    let test_name = test.chars().take_while(|&ch| ch != '[').collect::<String>();
                    (path.to_string(), test_name)
                })
                .ok_or(FztError::GeneralParsingError(format!(
                    "Parsing Pytest failed: {}",
                    line
                )))?;
            let entry = py_tests.get_mut(&path);
            match entry {
                Some(tests) => {
                    tests.insert(test_name);
                }
                None => {
                    let mut new_tests = HashSet::new();
                    new_tests.insert(test_name);
                    py_tests.insert(path, new_tests);
                }
            }
        }
        self.timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        self.tests = py_tests;
        Ok(())
    }
}

impl Tests for PytestTests {
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
        // TODO: also update failed tests, check if entries still exist
        let updated = update_tests(
            self.root_folder.as_str(),
            &mut self.timestamp,
            &mut self.tests,
            true,
        )?;
        if updated {
            self.parse_python_tests(get_pytests()?.as_str())?;
        }
        Ok(updated || files_filtered_out)
    }

    fn update_failed(&mut self, runtime_output: &str) -> bool {
        let failed_tests = parse_failed_tests(runtime_output);
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
        todo!()
    }

    fn get_covered_tests(&mut self) -> Vec<impl Test> {
        todo!();
        Vec::<PythonTest>::new()
    }
}
