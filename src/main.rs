use std::process::{Command, Stdio};
use std::str;


fn get_current_python_tests() -> String {
    let output = Command::new("python")
        .arg("-m")
        .arg("pytest")
        .arg("--co")
        .arg("-q").output()
        .expect("failed to retrieve python tests");
    str::from_utf8(output.stdout.as_slice()).unwrap().to_string()
}

fn get_tests_to_run(all_test: String) -> Vec<String> {
    let input = Command::new("echo")
        .arg(all_test)                 
        .stdout(Stdio::piped())     
        .spawn().unwrap();


    let output = Command::new("fzf")
        .arg("-m")
        .arg("--bind")
        .arg("ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all")
        .arg("--height")
        .arg("50%")
        .stdin(Stdio::from(input.stdout.unwrap())).output().expect("failed to retrieve selected python tests");
    str::from_utf8(output.stdout.as_slice()).unwrap().lines().map(|line| line.to_string()).collect()
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

fn main() {
    let python_tests = get_current_python_tests();
    let tests_to_run = get_tests_to_run(python_tests);
    run_tests(tests_to_run);
}
