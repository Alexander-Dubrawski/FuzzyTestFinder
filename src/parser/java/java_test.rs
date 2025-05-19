use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::parser::{Test, Tests};

use super::parser::JavaParser;

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, Vec<JavaTest>>,
}

impl JavaTests {
    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaTest {
    pub class_path: String,
    pub method_name: String,
}

pub struct JavaTestItem {
    pub path: String,
    pub class_path: String,
    pub test: String,
}

impl JavaTestItem {
    pub fn new(path: String, class_path: String, test: String) -> Self {
        Self {
            path,
            class_path,
            test
        }
    }
}

impl Test for JavaTestItem {
    fn runtime_argument(&self) -> String {
         format!("{}.{}", self.class_path, self.test)
    }

    fn search_item_name(&self) -> String {
        format!("{}->{}", self.path, self.test)
    }
}

impl Tests for JavaTests {
    fn to_json(&self) -> Result<String, crate::errors::FztError> {
        todo!()
    }

    fn tests(self) -> Vec<impl Test> {
        let mut output = vec![];
        self.tests.into_iter().for_each(|(path, tests)| {
            tests.into_iter().for_each(|test| {
                output.push(JavaTestItem::new(path.clone(), test.class_path, test.method_name));
            });
        });
        output
    }

    fn update(&mut self, only_check_for_update: bool) -> Result<bool, crate::errors::FztError> {
        let parser = JavaParser::new(self.root_folder.clone());
        let updated = parser.parse_tests(self, only_check_for_update)?;
        Ok(updated)
    }
}