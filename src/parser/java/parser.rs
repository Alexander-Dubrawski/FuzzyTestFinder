use std::process::Command;
use std::str;

use crate::errors::FztError;

use super::java_test::JavaTests;



#[derive(Default)]
pub struct JavaParser {
    root_dir: String,
}

impl JavaParser {
    pub fn new(root_dir: String) -> Self {
        Self { root_dir }
    }

    fn get_tests(&self, test_json: &str) -> Result<String, FztError> {
        let binding = Command::new("fzt-java-parser")
            .arg("-0")
            .arg(self.root_dir.as_str())
            .arg("-c")
            .arg(test_json)
            .output()
            .expect("failed to retrieve python tests");
        str::from_utf8(binding.stdout.as_slice())
            .map(|out| out.to_string())
            .map_err(FztError::from)
    }


    pub fn parse_tests(&self, tests: &mut JavaTests) -> Result<bool, FztError> {
        let test_json = serde_json::to_string(&tests)?;
        let updated_test_json = self.get_tests(test_json.as_str())?;
        let updated = test_json != updated_test_json;
        *tests = serde_json::from_str(updated_test_json.as_str())?;
        Ok(updated)
    }
}