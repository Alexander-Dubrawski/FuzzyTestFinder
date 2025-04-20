use std::process::Command;

use crate::errors::FztError;

#[derive(Default)]
pub struct PytestRuntime {}

impl PytestRuntime {
    pub fn run_tests(&self, tests: Vec<String>) -> Result<(), FztError> {
        let mut command = Command::new("python");
        command.arg("-m");
        command.arg("pytest");
        command.arg("--capture=no");
        tests.into_iter().for_each(|test| {
            command.arg(test);
        });
        command.status()?;
        Ok(())
    }
}
