use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    tests::{
        Test, Tests,
        python::{
            helper::{filter_out_deleted_files, update_tests},
            python_test::PythonTest,
        },
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RustPytonTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<String>>,
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
        }
    }

    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
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
        filter_out_deleted_files(&self.root_folder, &mut self.tests);
        let updated = update_tests(
            self.root_folder.as_str(),
            &mut self.timestamp,
            &mut self.tests,
            false,
        )?;
        Ok(updated)
    }
}
