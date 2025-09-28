use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crossbeam_channel::{Receiver as CrossbeamReceiver, TryRecvError as CrossbeamTryRecvError};
use std::sync::mpsc::{Receiver as StdReceiver, TryRecvError as StdTryRecvError};

use crate::errors::FztError;

use super::RuntimeFormatter;

pub struct CaptureOutput {
    pub stopped: bool,
    pub message: String,
}

impl StringReceiver for StdReceiver<String> {
    type TryError = StdTryRecvError;

    fn try_recv(&self) -> Result<String, StdTryRecvError> {
        self.try_recv()
    }
    fn is_empty_error(e: &Self::TryError) -> bool {
        matches!(e, std::sync::mpsc::TryRecvError::Empty)
    }
    fn error_msg(e: &Self::TryError) -> String {
        e.to_string()
    }
}

impl StringReceiver for CrossbeamReceiver<String> {
    type TryError = CrossbeamTryRecvError;
    fn try_recv(&self) -> Result<String, Self::TryError> {
        self.try_recv()
    }
    fn is_empty_error(e: &Self::TryError) -> bool {
        matches!(e, crossbeam_channel::TryRecvError::Empty)
    }
    fn error_msg(e: &Self::TryError) -> String {
        e.to_string()
    }
}

pub trait StringReceiver {
    type TryError;
    fn try_recv(&self) -> Result<String, Self::TryError>;
    fn is_empty_error(e: &Self::TryError) -> bool;
    fn error_msg(e: &Self::TryError) -> String;
}

fn check_stop_run<R: StringReceiver>(receiver: &Option<R>) -> Result<bool, FztError> {
    if let Some(receiver) = receiver {
        match receiver.try_recv() {
            Ok(_) => {
                return Ok(true);
            }
            Err(e) => {
                if !R::is_empty_error(&e) {
                    return Err(FztError::RuntimeError(format!(
                        "Channel disconnected or error: {}",
                        R::error_msg(&e)
                    )));
                }
            }
        }
    }
    Ok(false)
}

pub fn run_and_capture_print<F, R>(
    mut cmd: Command,
    runtime_formatter: &mut F,
    receiver: Option<R>,
) -> Result<CaptureOutput, FztError>
where
    F: RuntimeFormatter,
    R: StringReceiver,
{
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let mut output = String::new();

    let mut stopped = false;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if check_stop_run(&receiver)? {
                stopped = true;
                break;
            }
            let line = line?;
            runtime_formatter.line(&line)?;
            output.push_str(&line);
            output.push('\n');
        }
    }

    if !stopped {
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let mut error_output = String::new();
            for line in reader.lines() {
                if check_stop_run(&receiver)? {
                    stopped = true;
                    break;
                }
                let line = line?;
                error_output.push_str(&line);
                error_output.push('\n');
            }
            if !error_output.is_empty() {
                return Err(FztError::RuntimeError(error_output));
            }
        }
    }

    if stopped {
        child.kill()?;
    } else {
        child.wait()?;
    }
    let plain_bytes = strip_ansi_escapes::strip(output.as_bytes());
    let output = String::from_utf8(plain_bytes).map_err(|e| FztError::from(e))?;

    Ok(CaptureOutput {
        stopped,
        message: output,
    })
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
