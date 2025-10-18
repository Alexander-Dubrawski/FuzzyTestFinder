use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use itertools::Itertools;

use crate::errors::FztError;

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

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout_output = String::new();

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line?;
                stdout_output.push_str(&line);
                stdout_output.push('\n');
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = line?;
                println!("{}", line);
            }
        }

        let status = child.wait()?;

        if !status.success() {
            return Err(FztError::RustError(format!(
                "Failed to run `cargo test -- --list`"
            )));
        }

        let mut tests = Vec::new();
        for line in stdout_output.lines() {
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
