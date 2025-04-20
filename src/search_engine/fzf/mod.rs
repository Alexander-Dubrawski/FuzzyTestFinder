use std::process::{Command, Stdio};
use std::str;

use crate::parser::{Test, Tests};

#[derive(Default)]
pub struct FzfSearchEngine {}

impl FzfSearchEngine {
    pub fn get_tests_to_run(&self, all_test: impl Tests) -> Vec<String> {
        let mut input = String::new();
        all_test.tests().into_iter().for_each(|test| {
            input.push_str(format!("{}", test.runtime_argument()).as_str());
        });
        let echo_input = Command::new("echo")
            .arg(input)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let output = Command::new("fzf")
            .arg("-m")
            .arg("--bind")
            .arg("ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all")
            .arg("--height")
            .arg("50%")
            .stdin(Stdio::from(echo_input.stdout.unwrap()))
            .output()
            .expect("failed to retrieve selected python tests");
        str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .lines()
            .map(|line| line.to_string())
            .collect()
    }
}
