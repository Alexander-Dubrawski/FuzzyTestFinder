use std::{collections::HashMap, path::PathBuf};

use super::{Test, Tests};

fn extract_test_selection<T: Test>(tests: &[T]) -> HashMap<String, String> {
    HashMap::from_iter(
        tests
            .iter()
            .map(|test| (test.name(), test.runtime_argument())),
    )
}

fn extract_file_section<T: Test>(tests: &[T]) -> HashMap<String, Vec<String>> {
    let mut file_section: HashMap<String, Vec<String>> = HashMap::new();
    tests.iter().for_each(|test| {
        let file_path = test.file_path();
        if let Some(args) = file_section.get_mut(&file_path) {
            args.push(test.runtime_argument());
        } else {
            file_section.insert(file_path, vec![test.runtime_argument()]);
        }
    });
    file_section
}

fn extract_dictionary_selection<T: Test>(tests: &[T]) -> HashMap<String, Vec<String>> {
    let mut dictionary_selection: HashMap<String, Vec<String>> = HashMap::new();
    tests.iter().for_each(|test| {
        let file_path = test.file_path();
        let parent = PathBuf::from(file_path)
            .parent()
            .map(|path| path.to_str().expect("Expect valid path"))
            .unwrap_or("root")
            .to_string();
        if let Some(args) = dictionary_selection.get_mut(&parent) {
            args.push(test.runtime_argument());
        } else {
            dictionary_selection.insert(parent.to_string(), vec![test.runtime_argument()]);
        }
    });
    dictionary_selection
}

fn extract_runtime_selection<T: Test>(tests: &[T]) -> HashMap<String, String> {
    HashMap::from_iter(
        tests
            .iter()
            .map(|test| (test.runtime_argument(), test.runtime_argument())),
    )
}

pub struct TestProvider {
    test_selection: HashMap<String, String>,
    file_selection: HashMap<String, Vec<String>>,
    dictionary_selection: HashMap<String, Vec<String>>,
    runtime_selection: HashMap<String, String>,
}

impl TestProvider {
    pub fn new<T: Tests>(tests: &T) -> Self {
        let aviable_tests = tests.tests();
        Self {
            test_selection: extract_test_selection(aviable_tests.as_slice()),
            file_selection: extract_file_section(aviable_tests.as_slice()),
            dictionary_selection: extract_dictionary_selection(aviable_tests.as_slice()),
            runtime_selection: extract_runtime_selection(aviable_tests.as_slice()),
        }        
    }

    pub fn test_selection(&self) -> &HashMap<String, String> {
        &self.test_selection
    }
    pub fn file_selection(&self) -> &HashMap<String, Vec<String>> {
        &self.file_selection
    }
    pub fn dictionary_selection(&self) -> &HashMap<String, Vec<String>> {
        &self.dictionary_selection
    }
    pub fn runtime_selection(&self) -> &HashMap<String, String> {
        &self.runtime_selection
    }
}
