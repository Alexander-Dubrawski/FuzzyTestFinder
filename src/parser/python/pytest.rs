use itertools::Itertools;
use std::str;
use std::{
    collections::{HashMap, HashSet},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::parser::{Parser, Tests};

use super::python_tests::PythonTests;

#[derive(Default)]
pub struct PyTestParser {
    // absolute path
    root_dir: String,
}

impl PyTestParser {
    pub fn new(root_dir: String) -> Self {
        Self { root_dir }
    }

    fn parse_python_tests(&self) -> PythonTests {
        let binding = Command::new("python")
            .arg("-m")
            .arg("pytest")
            .arg("--co")
            .arg("-q")
            .output()
            .expect("failed to retrieve python tests");
        let output = str::from_utf8(binding.stdout.as_slice()).unwrap();

        let mut py_tests: HashMap<String, HashSet<String>> = HashMap::new();
        for line in output.lines() {
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
                .unwrap();
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
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        PythonTests::new(self.root_dir.clone(), timestamp, py_tests)
    }
}

impl Parser<PythonTests> for PyTestParser {
    fn parse_tests(&self, tests: &mut PythonTests) -> bool {
        if tests.update(true) {
            *tests = self.parse_python_tests();
            true
        } else {
            false
        }
    }
}
