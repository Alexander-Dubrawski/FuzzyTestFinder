use std::process::{Command, Stdio};
use std::str;

pub fn get_tests_to_run(all_test: String) -> Vec<String> {
    let input = Command::new("echo")
        .arg(all_test)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = Command::new("fzf")
        .arg("-m")
        .arg("--bind")
        .arg("ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all")
        .arg("--height")
        .arg("50%")
        .stdin(Stdio::from(input.stdout.unwrap()))
        .output()
        .expect("failed to retrieve selected python tests");
    str::from_utf8(output.stdout.as_slice())
        .unwrap()
        .lines()
        .map(|line| line.to_string())
        .collect()
}
