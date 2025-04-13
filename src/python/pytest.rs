use crate::fzf::fzf_engine::get_tests_to_run;
use std::process::Command;
use std::str;

use super::pytest_parser::parse_python_tests;
use super::types::PyTests;

fn pytest() -> PyTests {
    let binding = Command::new("python")
        .arg("-m")
        .arg("pytest")
        .arg("--co")
        .arg("-q")
        .output()
        .expect("failed to retrieve python tests");
    let output = str::from_utf8(binding.stdout.as_slice()).unwrap();
    parse_python_tests(output)
}

fn run_tests(tests: Vec<String>) {
    let mut command = Command::new("python");
    command.arg("-m");
    command.arg("pytest");
    command.arg("--capture=no");
    tests.into_iter().for_each(|test| {
        command.arg(test);
    });
    command.status().expect("failed to run tests");
}

pub fn run() {
    let python_tests = pytest();
    let tests_to_run = get_tests_to_run(python_tests.into());
    run_tests(tests_to_run);
}
