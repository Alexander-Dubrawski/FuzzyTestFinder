use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crate::errors::FztError;

pub fn run_and_capture(mut cmd: Command) -> Result<String, FztError> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let mut output = String::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            println!("{}", line); // Show in terminal
            output.push_str(&line);
            output.push('\n');
        }
    }

    child.wait()?;
    let plain_bytes = strip_ansi_escapes::strip(output.as_bytes());
    String::from_utf8(plain_bytes).map_err(|e| FztError::from(e))
}
