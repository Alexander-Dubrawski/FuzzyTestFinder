use std::{io::{BufRead, BufReader}, process::{Command, Stdio}};

use crate::errors::FztError;

pub fn run_and_capture(mut cmd: Command) -> Result<String, FztError> {
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let mut output = String::new();
    
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            println!("{}", line);
            output.push_str(&line);
            output.push('\n');
        }
    }
    child.wait()?;
    Ok(output)
}