use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
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
    Ok(output)
}
