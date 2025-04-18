use std::collections::{HashMap, HashSet};

use crate::cache::types::CacheEntry;

#[derive(Debug, Default, PartialEq)]
pub struct PyTests {
    pub tests: HashMap<String, HashSet<String>>,
}

impl PyTests {
    pub fn new(tests: HashMap<String, HashSet<String>>) -> Self {
        Self { tests }
    }
}

impl Into<String> for PyTests {
    fn into(self) -> String {
        let mut output = String::new();
        for (path, values) in self.tests.into_iter() {
            for test_name in values.into_iter() {
                output.push_str(format!("{}::{}\n", path, test_name).as_str());
            }
        }
        output
    }
}

trait Parser {
    fn parse_test(&self, root: &str) -> CacheEntry;
    fn update_tests(&self, cache_entry: &mut CacheEntry) -> bool;
}
