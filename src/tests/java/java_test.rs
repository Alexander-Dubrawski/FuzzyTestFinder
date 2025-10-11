use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    errors::FztError,
    tests::{Test, Tests},
};

use super::{helper::parse_failed_tests, parser::JavaParser};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JavaTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, Vec<JavaTest>>,
    pub failed_tests: HashMap<String, Vec<JavaTest>>,
}

impl JavaTests {
    pub fn new_empty(root_folder: String) -> Self {
        Self {
            root_folder,
            timestamp: 0,
            tests: HashMap::new(),
            failed_tests: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct JavaTest {
    pub class_path: String,
    pub method_name: String,
}

#[derive(Debug, Clone, PartialEq)]
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

    fn file_path(&self) -> String {
        self.path.clone()
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
        let mut output = vec![];
        self.failed_tests.iter().for_each(|(path, tests)| {
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

    fn update_file_coverage(
        &mut self,
        _coverage: &HashMap<String, Vec<String>>,
    ) -> Result<bool, FztError> {
        todo!()
    }

    #[allow(unreachable_code)]
    fn get_covered_tests(&self) -> Vec<impl Test> {
        todo!();
        return Vec::<JavaTestItem>::new();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::utils::test_utils::copy_dict;

    use super::*;
    use pretty_assertions::assert_eq;

    fn compare(java_tests: &JavaTests, mut expected: Vec<JavaTestItem>) {
        expected.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        let mut results = java_tests.tests();
        results.sort_by(|a, b| a.runtime_argument().cmp(&b.runtime_argument()));
        assert_eq!(results.len(), expected.len());

        for (res, exp) in results.iter().zip(expected.iter()) {
            assert_eq!(res.runtime_argument(), exp.runtime_argument());
            assert_eq!(res.name(), exp.name());
        }
    }

    #[test]
    fn collect_tests() {
        let mut path = std::env::current_dir().unwrap();
        path.push("parsers/java/app/src/test/resources/tests");
        let (_temp_dir, dir_path) = copy_dict(path.as_path()).unwrap();
        let test_path = dir_path.as_path().to_str().unwrap();
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
        assert!(java_tests.update().unwrap());
        compare(&java_tests, expected.clone());

        assert!(!java_tests.update().unwrap());
        compare(&java_tests, expected);

        // Remove test
        std::fs::remove_file(format!("{test_path}/java/b/testThree.java")).unwrap();
        expected = vec![
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
        ];
        assert!(java_tests.update().unwrap());
        compare(&java_tests, expected);

        // Update test
        std::fs::write(
            &Path::new(test_path).join("java/a/testOne.java"),
            "package tests.java.a; import org.junit.jupiter.api.Test; class TestOne { @Test void oneNew() {}}",
        )
        .unwrap();
        expected = vec![
            JavaTestItem::new(
                "java/a/testOne.java".to_string(),
                "tests.java.a.TestOne".to_string(),
                "oneNew".to_string(),
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
        ];
        assert!(java_tests.update().unwrap());
        compare(&java_tests, expected);
    }
}
