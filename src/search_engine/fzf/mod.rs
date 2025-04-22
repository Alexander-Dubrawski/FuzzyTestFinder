use std::process::{Command, Output, Stdio};
use std::str;

use crate::cache::manager::CacheManager;
use crate::errors::FztError;
use crate::parser::{Test, Tests};

use super::SearchEngine;

fn run_fzf(input: &str, read_null: bool) -> Result<Output, FztError> {
    let echo_input = Command::new("echo")
        .arg(input)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut command = Command::new("fzf");
    command
        .arg("-m")
        .arg("--bind")
        .arg("ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all")
        .arg("--height")
        .arg("50%");

    if read_null {
        command.arg("--read0").arg("--gap");
    }

    let output = command
        .stdin(Stdio::from(
            echo_input.stdout.expect("echo should has output"),
        ))
        .output()?;
    Ok(output)
}

#[derive(Default)]
pub struct FzfSearchEngine {}

impl SearchEngine for FzfSearchEngine {
    fn get_tests_to_run(&self, all_test: impl Tests) -> Result<Vec<String>, FztError> {
        let mut input = String::new();
        all_test.tests().into_iter().for_each(|test| {
            input.push_str(format!("{}\n", test.runtime_argument()).as_str());
        });
        let output = run_fzf(input.as_str(), false)?;
        let tests: Vec<String> = str::from_utf8(output.stdout.as_slice())?
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(tests)
    }

    fn get_from_history(&self, history: Vec<Vec<String>>) -> Result<Vec<String>, FztError> {
        let mut input = String::new();
        history.into_iter().for_each(|tests| {
            let mut command = String::new();
            tests.into_iter().for_each(|test| {
                command.push_str(format!("{test}\0").as_str());
            });
            // TODO: remove last \0
            input.push_str(format!("{command}\n").as_str());
        });
        let output = run_fzf(input.as_str(), true)?;
        Ok(str::from_utf8(output.stdout.as_slice())?
            .split("\0")
            .into_iter()
            .map(|test| test.to_string())
            .collect())
    }
}
