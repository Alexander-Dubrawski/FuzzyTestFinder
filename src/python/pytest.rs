use crate::cache::manager::get_entry;
use crate::fzf::fzf_engine::get_tests_to_run;
use crate::parser::Parser;
use sha2::{Digest, Sha256};
use std::env;
use std::process::Command;

use super::pytest_parser::PyTestParser;
use super::rp_parser::RpParser;
use super::types::PyTests;

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
    let use_rp_parser = true;
    let path = env::current_dir().unwrap();
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let result = hasher.finalize();

    let project_id = format!("{:x}", result);

    let cache_entry = if use_rp_parser {
        let parser = RpParser::new(path_str.to_string());
        get_entry(project_id.as_str(), parser)
    } else {
        let parser = PyTestParser::new(path_str.to_string());
        get_entry(project_id.as_str(), parser)
    };
    let tests = PyTests::new(cache_entry.tests);
    let tests_to_run = get_tests_to_run(tests.into());
    run_tests(tests_to_run);
}
