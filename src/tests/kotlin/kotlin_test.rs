use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    collect_tests,
    errors::FztError,
    runtime::FailedTest,
    tests::{Test, Tests},
};

use super::parser::{collect_tests_from_file, parse_failed_tests};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KotlinTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<KotlinTest>>,
    pub failed_tests: HashMap<String, HashSet<KotlinTest>>,
}

impl KotlinTests {
    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
            failed_tests: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct KotlinTest {
    pub class_path: String,
    pub method_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KotlinTestItem {
    pub path: String,
    pub class_path: String,
    pub test: String,
}

impl KotlinTestItem {
    pub fn new(path: String, class_path: String, test: String) -> Self {
        Self {
            path,
            class_path,
            test,
        }
    }
}

impl Test for KotlinTestItem {
    fn runtime_argument(&self) -> String {
        format!("{}.{}", self.class_path, self.test)
    }

    fn name(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }

    fn file_path(&self) -> String {
        self.path.clone()
    }
}

impl Tests for KotlinTests {
    fn to_json(&self) -> Result<String, FztError> {
        serde_json::to_string(&self).map_err(FztError::from)
    }

    fn tests(&self) -> Vec<impl Test> {
        let mut output = vec![];
        self.tests.iter().for_each(|(path, tests)| {
            tests.iter().for_each(|test| {
                output.push(KotlinTestItem::new(
                    path.clone(),
                    test.class_path.clone(),
                    test.method_name.clone(),
                ));
            });
        });
        output
    }

    fn tests_failed(&self) -> Vec<impl Test> {
        let mut output = vec![];
        self.failed_tests.iter().for_each(|(path, tests)| {
            tests.iter().for_each(|test| {
                output.push(KotlinTestItem::new(
                    path.clone(),
                    test.class_path.clone(),
                    test.method_name.clone(),
                ));
            });
        });
        output
    }

    fn update(&mut self) -> Result<bool, FztError> {
        let updated = collect_tests(
            self.root_folder.as_str(),
            &mut self.timestamp,
            &mut self.tests,
            false,
            "kt",
            None,
            collect_tests_from_file,
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
        Ok(updated)
    }

    fn update_file_coverage(
        &mut self,
        coverage: &HashMap<String, Vec<String>>,
    ) -> Result<bool, FztError> {
        unimplemented!()
    }

    #[allow(unreachable_code)]
    fn get_covered_tests(&self) -> Vec<impl Test> {
        todo!();
        return Vec::<KotlinTestItem>::new();
    }

    fn update_failed(&mut self, failed_tests_output: &[FailedTest]) -> bool {
        let failed_tests = parse_failed_tests(failed_tests_output, &self.tests);
        if self.failed_tests == failed_tests {
            false
        } else {
            self.failed_tests = failed_tests;
            true
        }
    }
}
