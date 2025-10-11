use std::{
    process::{Command},
    sync::mpsc::Receiver,
};

use itertools::Itertools;

use crate::{
    errors::FztError,
    utils::process::{OnlyStderrFormatter, run_and_capture_print},
};

use super::ParseRustTest;

#[derive(Default)]
pub struct RustTestParser {}

impl ParseRustTest for RustTestParser {
    fn parse_tests(&self) -> Result<Vec<(Vec<String>, String)>, FztError> {
        let mut command = Command::new("cargo");
        command.arg("test");
        command.arg("--");
        command.arg("--list");
        command.arg("--color=always");
        command.env("CARGO_TERM_COLOR", "always");

        let mut formatter = OnlyStderrFormatter;

        let captured = run_and_capture_print(command, &mut formatter, None::<Receiver<String>>)?;

        if let Some(status) = captured.status {
            if status.success() == false {
                return Err(FztError::RustError(format!(
                    "Failed to run `cargo test -- --list`"
                )));
            }
        } else {
            return Err(FztError::RustError(format!(
                "Failed to run `cargo test -- --list` no process status received"
            )));
        }

        let mut tests = Vec::new();
        for line in captured.stdout.lines() {
            if line.is_empty() {
                break;
            }
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
