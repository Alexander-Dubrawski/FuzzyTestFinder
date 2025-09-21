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
    let mut partitions = Vec::new();

    if n < m {
        // Each element gets its own partition
        for item in vec {
            partitions.push(vec![item.clone()]);
        }
    } else {
        let base_size = n / m;
        let remainder = n % m;
        let mut start = 0;

        for i in 0..m {
            let mut end = start + base_size;
            if i < remainder {
                end += 1;
            }
            partitions.push(vec[start..end].to_vec());
            start = end;
        }
    }
    partitions
}
