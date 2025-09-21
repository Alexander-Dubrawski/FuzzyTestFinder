use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::PathBuf,
    str::FromStr,
};

use crate::{errors::FztError, search_engine::Append};

use super::{Test, Tests};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectGranularity {
    Test,
    File,
    Directory,
    RunTime,
}

impl fmt::Display for SelectGranularity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SelectGranularity::Test => write!(f, "Test"),
            SelectGranularity::File => write!(f, "File"),
            SelectGranularity::Directory => write!(f, "Directory"),
            SelectGranularity::RunTime => write!(f, "RunTime"),
        }
    }
}

impl FromStr for SelectGranularity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "test" => Ok(SelectGranularity::Test),
            "file" => Ok(SelectGranularity::File),
            "directory" => Ok(SelectGranularity::Directory),
            "runtime" => Ok(SelectGranularity::RunTime),
            _ => Err(format!("Invalid selection: {}", s)),
        }
    }
}

impl From<Append> for SelectGranularity {
    fn from(value: Append) -> Self {
        match value {
            Append::Test => SelectGranularity::Test,
            Append::File => SelectGranularity::File,
            Append::Directory => SelectGranularity::Directory,
            Append::RunTime => SelectGranularity::RunTime,
            _ => panic!("Unsupported Append variant for SelectGranularity conversion"),
        }
    }
}

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

fn extract_runtime_selection<T: Test>(tests: &[T]) -> HashSet<String> {
    HashSet::from_iter(tests.iter().map(|test| test.runtime_argument()))
}

pub struct TestProvider {
    test_selection: HashMap<String, String>,
    file_selection: HashMap<String, Vec<String>>,
    dictionary_selection: HashMap<String, Vec<String>>,
    runtime_selection: HashSet<String>,
    // If set runtime arguments are always returned by
    // this test provider. This allows a level of
    // abstraction of selection items and the corresponding
    // runtime arguments that actually exist
    default_test_provider: Option<Box<TestProvider>>,
}

impl TestProvider {
    pub fn new<T: Tests>(tests: &T) -> Self {
        let available_tests = tests.tests();
        Self {
            test_selection: extract_test_selection(available_tests.as_slice()),
            file_selection: extract_file_section(available_tests.as_slice()),
            dictionary_selection: extract_dictionary_selection(available_tests.as_slice()),
            runtime_selection: extract_runtime_selection(available_tests.as_slice()),
            default_test_provider: None,
        }
    }

    pub fn new_failed<T: Tests>(tests: &T) -> Self {
        let available_tests = tests.tests_failed();
        Self {
            test_selection: extract_test_selection(available_tests.as_slice()),
            file_selection: extract_file_section(available_tests.as_slice()),
            dictionary_selection: extract_dictionary_selection(available_tests.as_slice()),
            runtime_selection: extract_runtime_selection(available_tests.as_slice()),
            default_test_provider: Some(Box::new(TestProvider::new(tests))),
        }
    }

    pub fn select_option(&self, select_granularity: &SelectGranularity) -> Vec<&str> {
        match select_granularity {
            SelectGranularity::Test => self
                .test_selection
                .keys()
                .map(|test| test.as_str())
                .collect(),
            SelectGranularity::File => self
                .file_selection
                .keys()
                .map(|test| test.as_str())
                .collect(),
            SelectGranularity::Directory => self
                .dictionary_selection
                .keys()
                .map(|test| test.as_str())
                .collect(),
            SelectGranularity::RunTime => self
                .runtime_selection
                .iter()
                .map(|test| test.as_str())
                .collect(),
        }
    }

    pub fn runtime_arguments(
        &self,
        select_granularity: &SelectGranularity,
        selection: &[String],
    ) -> Vec<String> {
        // all_test_provider always takes precedence
        if let Some(all_test_provider) = self.default_test_provider.as_ref() {
            return all_test_provider.runtime_arguments(select_granularity, selection);
        }
        match select_granularity {
            SelectGranularity::Test => selection
                .iter()
                .filter(|select| {
                    if !self.test_selection.contains_key(*select) {
                        println!("{select} test could not be found in application. Skipped.");
                        false
                    } else {
                        true
                    }
                })
                .map(|select| self.test_selection[select].clone())
                .collect(),
            SelectGranularity::File => selection
                .iter()
                .filter(|select| {
                    if !self.file_selection.contains_key(*select) {
                        println!("{select} file could not be found in application. Skipped.");
                        false
                    } else {
                        true
                    }
                })
                .flat_map(|select| self.file_selection[select].clone())
                .collect(),
            SelectGranularity::Directory => selection
                .iter()
                .filter(|select| {
                    if !self.file_selection.contains_key(*select) {
                        println!("{select} directory could not be found in application. Skipped.");
                        false
                    } else {
                        true
                    }
                })
                .flat_map(|select| self.dictionary_selection[select].clone())
                .collect(),
            SelectGranularity::RunTime => selection
                .iter()
                .filter(|select| {
                    if !self.runtime_selection.contains(*select) {
                        println!("{select} could not be found in application. Skipped.");
                        false
                    } else {
                        true
                    }
                })
                .cloned()
                .collect(),
        }
    }

    pub fn all(&self, select_granularity: &SelectGranularity) -> Vec<String> {
        match select_granularity {
            SelectGranularity::Test => self.test_selection.values().cloned().collect(),
            SelectGranularity::File => self.file_selection.values().flatten().cloned().collect(),
            SelectGranularity::Directory => self
                .dictionary_selection
                .values()
                .flatten()
                .cloned()
                .collect(),
            SelectGranularity::RunTime => self.runtime_selection.iter().cloned().collect(),
        }
    }
}
