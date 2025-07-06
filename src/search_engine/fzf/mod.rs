use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::str;

use crate::errors::FztError;
use crate::runner::Preview;

use super::SearchEngine;

fn run_fzf(input: &str, read_null: bool, preview: &Option<Preview>) -> Result<Output, FztError> {
    let mut command = Command::new("fzf");
    command
        .arg("-m")
        .arg("--bind")
        .arg("ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all")
        .arg("--height")
        .arg("50%")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    if read_null {
        command.arg("--read0").arg("--gap");
    }

    if let Some(preview_mode) = preview {
        match preview_mode {
            Preview::File => {
                command
                    .arg("--delimiter")
                    .arg("::")
                    .arg("--preview")
                    .arg("bat --style=numbers --color=always {1}");
            }
            Preview::Test => {
                command
                .arg("--delimiter")
                .arg("::")
                .arg("--preview")
                .arg(" rg --color=always --line-number --no-heading '{2}' '{1}' --context 5 | bat --style=numbers --color=always");
            }
        }
    }

    let mut child = command.spawn()?;

    // Write the input (which may contain NUL bytes) to fzf's stdin
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(output)
}

#[derive(Default)]
pub struct FzfSearchEngine {}

impl SearchEngine for FzfSearchEngine {
    fn get_tests_to_run(
        &self,
        all_test: &[&str],
        preview: &Option<Preview>,
    ) -> Result<Vec<String>, FztError> {
        let mut input = String::new();
        all_test.iter().for_each(|test| {
            input.push_str(format!("{}\n", test).as_str());
        });
        let output = run_fzf(input.as_str(), false, preview)?;
        let tests: Vec<String> = str::from_utf8(output.stdout.as_slice())?
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(tests)
    }

    fn get_from_history(&self, history: &[Vec<String>]) -> Result<Vec<String>, FztError> {
        let mut input = String::new();
        history
            .iter()
            .filter(|tests| !tests.is_empty())
            .for_each(|tests| {
                let mut command = String::new();
                tests.into_iter().for_each(|test| {
                    command.push_str(format!("{test}\n").as_str());
                });
                command.remove(command.len() - 1);
                input.push_str(format!("{command}\0").as_str());
            });
        let mut output = run_fzf(input.as_str(), true, &None)?.stdout;
        // Replace Null byte with new line
        output.iter_mut().filter(|p| **p == 0).for_each(|p| *p = 10);
        Ok(str::from_utf8(output.as_slice())?
            .lines()
            .map(|line| line.to_string())
            .collect())
    }

    fn name(&self) -> String {
        String::from("fzf")
    }
}
