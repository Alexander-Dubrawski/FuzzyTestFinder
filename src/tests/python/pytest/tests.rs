use std::{collections::{HashMap, HashSet}, process::Command, time::{SystemTime, UNIX_EPOCH}};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str;
use crate::{errors::FztError, tests::{python::{helper::{filter_out_deleted_files, update_tests}, python_tests::PythonTest}, Test, Tests}};

fn get_pytests() -> Result<String, FztError> {
    let binding = Command::new("python")
        .arg("-m")
        .arg("pytest")
        .arg("--co")
        .arg("-q")
        .output()
        .expect("failed to retrieve python tests");
    str::from_utf8(binding.stdout.as_slice())
        .map(|out| out.to_string())
        .map_err(FztError::from)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PytestTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<String>>,
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
        }
    }

    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
        }
    }

    fn parse_python_tests(&mut self, pytest_output: &str) -> Result<(),FztError> {
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

    fn tests(self) -> Vec<impl Test> {
        let mut output = vec![];
        self.tests.into_iter().for_each(|(path, tests)| {
            tests.into_iter().for_each(|test| {
                // TODO: Move python test to mod
                output.push(PythonTest::new(path.clone(), test));
            });
        });
        output
    }

    fn update(&mut self) -> Result<bool, FztError> {
        filter_out_deleted_files( &mut self.tests);
        let updated = update_tests(self.root_folder.as_str(), &mut self.timestamp, &mut self.tests, true)?;
        if updated {
            self.parse_python_tests(get_pytests()?.as_str())?;
        }
        Ok(updated)
    }
}
