use crate::cache::manager::add_entry;
use crate::cache::manager::get_entry;
use crate::cache::types::CacheEntry;
use crate::fzf::fzf_engine::get_tests_to_run;
use sha2::{Digest, Sha256};
use std::env;
use std::process::Command;
use std::str;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use super::cache::PytestUpdatePytest;
use super::pytest_parser::parse_python_tests;
use super::types::PyTests;

pub fn pytest() -> PyTests {
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
    let path = env::current_dir().unwrap();
    let path_str = path.to_string_lossy();

    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();

    let project_id = format!("{:x}", result);

    match get_entry(project_id.as_str(), PytestUpdatePytest::default()) {
        Some(entry) => {
            let tests = PyTests::new(entry.tests);
            let tests_to_run = get_tests_to_run(tests.into());
            run_tests(tests_to_run);
        }
        None => {
            let python_tests = pytest();
            let path = format!("{}/tests", path_str.to_string());
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let entry = CacheEntry::new(path, timestamp, python_tests.tests.clone());
            add_entry(project_id, entry);
            let tests_to_run = get_tests_to_run(python_tests.into());
            run_tests(tests_to_run);
        }
    }
}
