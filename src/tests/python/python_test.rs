use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{FztError, tests::Test};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct PythonTest {
    pub path: String,
    pub test: String,
}

impl PythonTest {
    pub fn new(path: String, test: String) -> Self {
        Self { path, test }
    }

    pub fn try_from_pytest_test(test: &str) -> Result<Self, FztError> {
        let (path, test_name) = test
            .split("::")
            .collect_tuple()
            .map(|(path, test_name)| {
                let test_name = test_name
                    .chars()
                    .take_while(|&ch| ch != '[')
                    .collect::<String>();
                (path.to_string(), test_name)
            })
            .ok_or(FztError::GeneralParsingError(format!(
                "Parsing Pytest failed: {}",
                test
            )))?;
        Ok(Self {
            path,
            test: test_name,
        })
    }
}

impl Test for PythonTest {
    fn runtime_argument(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }

    fn name(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }

    fn file_path(&self) -> String {
        self.path.clone()
    }
}
