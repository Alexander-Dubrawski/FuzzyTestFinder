use std::process::Command;

use itertools::Itertools;

use crate::errors::FztError;

use super::ParseRustTest;

pub struct RustTestParser {}

impl ParseRustTest for RustTestParser {
    fn parse_tests() -> Result<Vec<(Vec<String>, String)>, FztError> {
        let binding = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--list")
            .output()
            .expect("failed to retrieve python tests");
        let output = std::str::from_utf8(binding.stdout.as_slice())
            .map(|out| out.to_string())
            .map_err(FztError::from)?;
        let mut tests = Vec::new();
        for line in output.lines() {
            if line.is_empty() {
                break;
            }
            // Parse: cache::manager::tests::get_non_existing_entry: test
            let (path, type_name) =
                line.split(" ")
                    .collect_tuple()
                    .ok_or(FztError::GeneralParsingError(format!(
                        "Parsing cargo tests failed: {}",
                        line
                    )))?;
            if type_name != "test" {
                continue;
            }
            let mut module_path = path
                .split("::")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            let mut test_name = module_path.pop().expect("Test needs to exist");
            // Remove `:`
            test_name.pop();
            tests.push((module_path, test_name));
        }
        Ok(tests)
    }
}
