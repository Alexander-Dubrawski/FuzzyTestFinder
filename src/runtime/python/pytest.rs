use std::process::Command;

#[derive(Default)]
pub struct PytestRuntime {}

impl PytestRuntime {
    pub fn run_tests(&self, tests: Vec<String>) {
        let mut command = Command::new("python");
        command.arg("-m");
        command.arg("pytest");
        command.arg("--capture=no");
        tests.into_iter().for_each(|test| {
            command.arg(test);
        });
        command.status().expect("failed to run tests");
    }
}