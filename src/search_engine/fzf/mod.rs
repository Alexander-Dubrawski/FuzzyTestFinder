use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::str::{self, FromStr};

use crate::errors::FztError;
use crate::runner::config::Preview;

use super::Append;
use super::SearchEngine;

const BAT_PREVIEW_SCRIPT: &str = include_str!("bat_preview_command.sh");
const BAT_LINE_PREVIEW_CONTEXT: i8 = 20;

fn run_fzf(
    input: &str,
    read_null: bool,
    preview: &Option<Preview>,
    query: &Option<String>,
) -> Result<Output, FztError> {
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

    if let Some(query) = query {
        command.arg("--query").arg(query);
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
                    .arg(format!(
                        "bash -c '{}' -- {{2}} {{1}} {BAT_LINE_PREVIEW_CONTEXT}",
                        BAT_PREVIEW_SCRIPT.replace('\'', "'\"'\"'")
                    ));
            }
            Preview::Directory => {
                command
                    .arg("--delimiter")
                    .arg("::")
                    .arg("--preview")
                    .arg(" ls '{1}'");
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

fn run_fzf_select(input: &str, preview: Option<&str>) -> Result<Output, FztError> {
    let mut command = Command::new("fzf");
    command.arg("--height").arg("50%");
    if let Some(preview) = preview {
        command.arg("--preview").arg(format!("echo '{}'", preview));
    }
    command.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = command.spawn()?;

    // Write the input (which may contain NUL bytes) to fzf's stdin
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(output)
}

#[derive(Default, Clone, Debug)]
pub struct FzfSearchEngine {}

impl SearchEngine for FzfSearchEngine {
    fn get_tests_to_run(
        &self,
        all_test: &[&str],
        preview: &Option<Preview>,
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError> {
        let mut input = String::new();
        all_test.iter().for_each(|test| {
            input.push_str(format!("{}\n", test).as_str());
        });
        let output = run_fzf(input.as_str(), false, preview, query)?;
        let tests: Vec<String> = str::from_utf8(output.stdout.as_slice())?
            .lines()
            .map(|line| line.to_string())
            .collect();
        Ok(tests)
    }

    fn get_from_history(
        &self,
        history: &[Vec<String>],
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError> {
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
        let mut output = run_fzf(input.as_str(), true, &None, query)?.stdout;
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

    fn appened(&self, preview: &str) -> Result<Append, FztError> {
        let output = run_fzf_select("Done\nDirectory\nFile\nRuntime\nTest", Some(preview))?;
        let mode: String = str::from_utf8(output.stdout.as_slice())?.to_string();
        Ok(Append::from_str(mode.trim())
            .expect("THIS IS A BUG. Search engine should return append option"))
    }

    fn select(&self, selected_items: &[&str]) -> Result<String, FztError> {
        let output = run_fzf_select(selected_items.join("\n").as_str(), None)?;
        Ok(str::from_utf8(output.stdout.as_slice())?.trim().to_string())
    }
}
