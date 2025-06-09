use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    tests::{Test, Tests},
};

use super::parser::JavaParser;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            test,
        }
    }
}

impl Test for JavaTestItem {
    fn runtime_argument(&self) -> String {
        format!("{}.{}", self.class_path, self.test)
    }

    fn name(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }
}

impl Tests for JavaTests {
    fn to_json(&self) -> Result<String, FztError> {
        serde_json::to_string(&self).map_err(FztError::from)
    }

    fn tests(&self) -> Vec<impl Test> {
        let mut output = vec![];
        self.tests.iter().for_each(|(path, tests)| {
            tests.iter().for_each(|test| {
                output.push(JavaTestItem::new(
                    path.clone(),
                    test.class_path.clone(),
                    test.method_name.clone(),
                ));
            });
        });
        output
    }

    fn update(&mut self) -> Result<bool, crate::errors::FztError> {
        let parser = JavaParser::new(self.root_folder.clone());
        let updated = parser.parse_tests(self, false)?;
        Ok(updated)
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
        path.push("parsers/java/app/src/test/resources/tests");
        let (_temp_dir, dir_path) = copy_dict(path.as_path()).unwrap();
        //TODO: outside programms can not access temp dir 
        let test_path = dir_path.as_path().to_str().unwrap();
        // let mut java_tests = JavaTests::new_empty("parsers/java/app/src/test/resources/tests".to_string());
        let mut java_tests = JavaTests::new_empty(test_path.to_string());
        let mut expected = vec![
            JavaTestItem::new(
                "java/a/testOne.java".to_string(),
                "tests.java.a.TestOne".to_string(),
                "one".to_string(),
            ),
            JavaTestItem::new(
                "java/a/testTwo.java".to_string(),
                "tests.java.a.TestTwo".to_string(),
                "two".to_string(),
            ),
            JavaTestItem::new(
                "java/a/testTwo.java".to_string(),
                "tests.java.a.TestTwo".to_string(),
                "twoOne".to_string(),
            ),
            JavaTestItem::new(
                "java/b/testThree.java".to_string(),
                "tests.java.b.TestThree".to_string(),
                "three".to_string(),
            ),
        ];
        expected.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert!(java_tests.update().unwrap());
        let mut results = java_tests.tests();
        results.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert_eq!(results.len(), expected.len());

        for (res, exp) in results.iter().zip(expected.iter()) {
            assert_eq!(res.runtime_argument(), exp.runtime_argument());
            assert_eq!(res.name(), exp.name());
        }

        drop(results);

        // // Remove test
        // std::fs::remove_file(format!("{test_path}/java/b/testThree.java")).unwrap();
        // expected = vec![
        //     JavaTestItem::new(
        //         "java/a/testOne.java".to_string(),
        //         "tests.java.a.TestOne".to_string(),
        //         "one".to_string(),
        //     ),
        //     JavaTestItem::new(
        //         "java/a/testTwo.java".to_string(),
        //         "tests.java.a.TestTwo".to_string(),
        //         "two".to_string(),
        //     ),
        //     JavaTestItem::new(
        //         "tests.java.a.TestTwo".to_string(),
        //         "tests.java.a.TestTwo".to_string(),
        //         "twoOne".to_string(),
        //     ),
        // ];
        // expected.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        
        // assert!(java_tests.update().unwrap());
        // let mut results = java_tests.tests();
        // results.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        // assert_eq!(results.len(), expected.len());
        
        // for (res, exp) in results.iter().zip(expected.iter()) {
        //     assert_eq!(res.runtime_argument(), exp.runtime_argument());
        //     assert_eq!(res.name(), exp.name());
        // }
    }
}
