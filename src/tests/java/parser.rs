use std::path::PathBuf;
use std::process::Command;
use std::{env, str};

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
        let home_dir = env::var("HOME").unwrap();
        let jar_path = PathBuf::from(home_dir).join(".fzt/fzt-java-parser.jar");
        let output = Command::new("java")
            .arg("-jar")
            .arg(jar_path)
            .arg("-p")
            .arg(self.root_dir.as_str())
            .arg("-c")
            .arg(test_json)
            .output()
            .expect("failed to retrieve python tests");
        // TODO: Handle error
        if !output.status.success() {
            eprintln!("Java error: {}", String::from_utf8_lossy(&output.stderr));
            //return Err(FztError::from("Java subprocess failed"));
        }
        str::from_utf8(output.stdout.as_slice())
            .map(|out| out.to_string())
            .map_err(FztError::from)
    }
    pub fn parse_tests(
        &self,
        tests: &mut JavaTests,
        only_check_for_update: bool,
    ) -> Result<bool, FztError> {
        let test_json = serde_json::to_string(&tests)?;
        let updated_test_json = self.get_tests(test_json.as_str())?;
        let updated = test_json != updated_test_json;
        if !only_check_for_update {
            *tests = serde_json::from_str(updated_test_json.as_str())?;
        }
        Ok(updated)
    }
}
