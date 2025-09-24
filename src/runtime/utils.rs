use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crate::errors::FztError;

use super::RuntimeFormatter;

pub fn run_and_capture_print<F: RuntimeFormatter>(
    mut cmd: Command,
    runtime_formatter: &mut F,
) -> Result<String, FztError> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let mut output = String::new();

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            runtime_formatter.line(&line)?;
            output.push_str(&line);
            output.push('\n');
        }
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut error_output = String::new();
        for line in reader.lines() {
            let line = line?;
            error_output.push_str(&line);
            error_output.push('\n');
        }
        if !error_output.is_empty() {
            return Err(FztError::RuntumeError(error_output));
        }
    }

    child.wait()?;
    let plain_bytes = strip_ansi_escapes::strip(output.as_bytes());
    String::from_utf8(plain_bytes).map_err(|e| FztError::from(e))
}

pub fn partition_tests(vec: &[String], m: usize) -> Vec<Vec<String>> {
    let n = vec.len();
    if m == 0 {
        return Vec::new();
    }
    if n == 0 {
        return vec![];
    }
    let mut partitions = vec![vec![]; m];

    for (i, item) in vec.iter().enumerate() {
        partitions[i % m].push(item.clone());
    }

    partitions.into_iter().filter(|p| !p.is_empty()).collect()
}
