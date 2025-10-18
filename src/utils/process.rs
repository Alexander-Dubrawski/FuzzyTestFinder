use std::{
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
};

use crossbeam_channel::{Receiver as CrossbeamReceiver, TryRecvError as CrossbeamTryRecvError};
use std::sync::mpsc::{Receiver as StdReceiver, TryRecvError as StdTryRecvError};

use crate::errors::FztError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct FailedTest {
    pub name: String,
    pub error_msg: String,
}

impl FailedTest {
    pub fn new(name: &str, error_msg: &str) -> Self {
        Self {
            name: name.to_string(),
            error_msg: error_msg.to_string(),
        }
    }
}

// TODO: Add function that prints all at once, so threats do not print in each other
pub trait OutputFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError>;
    fn err_line(&mut self, line: &str) -> Result<(), FztError>;
    fn add(&mut self, other: &Self);
    fn finish(self);
    fn coverage(&self) -> Vec<String>;
    fn skipped(&self) -> bool;
    fn reset_coverage(&mut self);
    fn failed_tests(&self) -> Vec<FailedTest>;
    fn update(&mut self) -> Result<(), FztError>;
    fn print(&self);
}

#[derive(Debug, Clone)]
pub struct DefaultFormatter;
impl OutputFormatter for DefaultFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{}", line);
        Ok(())
    }
    fn err_line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{}", line);
        Ok(())
    }

    fn add(&mut self, _other: &Self) {
        unimplemented!()
    }

    fn finish(self) {
        unimplemented!()
    }

    fn coverage(&self) -> Vec<String> {
        unimplemented!()
    }

    fn reset_coverage(&mut self) {
        unimplemented!()
    }

    fn failed_tests(&self) -> Vec<FailedTest> {
        todo!()
    }

    fn print(&self) {}

    fn update(&mut self) -> Result<(), FztError> {
        Ok(())
    }

    fn skipped(&self) -> bool {
        false
    }
}

pub struct OnlyStdoutFormatter;
impl OutputFormatter for OnlyStdoutFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{}", line);
        Ok(())
    }
    fn err_line(&mut self, _line: &str) -> Result<(), FztError> {
        Ok(())
    }

    fn add(&mut self, _other: &Self) {
        unimplemented!()
    }

    fn finish(self) {
        unimplemented!()
    }

    fn coverage(&self) -> Vec<String> {
        unimplemented!()
    }

    fn reset_coverage(&mut self) {
        unimplemented!()
    }

    fn failed_tests(&self) -> Vec<FailedTest> {
        todo!()
    }

    fn print(&self) {}
    fn update(&mut self) -> Result<(), FztError> {
        Ok(())
    }
    fn skipped(&self) -> bool {
        false
    }
}

pub struct OnlyStderrFormatter;
impl OutputFormatter for OnlyStderrFormatter {
    fn line(&mut self, _line: &str) -> Result<(), FztError> {
        Ok(())
    }
    fn err_line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{}", line);
        Ok(())
    }

    fn add(&mut self, _other: &Self) {
        unimplemented!()
    }

    fn finish(self) {
        unimplemented!()
    }

    fn coverage(&self) -> Vec<String> {
        unimplemented!()
    }

    fn reset_coverage(&mut self) {
        unimplemented!()
    }

    fn failed_tests(&self) -> Vec<FailedTest> {
        todo!()
    }

    fn print(&self) {}
    fn update(&mut self) -> Result<(), FztError> {
        Ok(())
    }
    fn skipped(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct CaptureOutput {
    pub stopped: bool,
    pub stdout: String,
    pub stderr: String,
    pub status: Option<ExitStatus>,
}

impl StringReceiver for StdReceiver<String> {
    type TryError = StdTryRecvError;

    fn try_recv(&self) -> Result<String, Self::TryError> {
        StdReceiver::try_recv(self)
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
        CrossbeamReceiver::try_recv(self)
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
    formatter: &mut F,
    receiver: Option<R>,
) -> Result<CaptureOutput, FztError>
where
    F: OutputFormatter,
    R: StringReceiver,
{
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let mut stdout_output = String::new();

    let mut stopped = false;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if check_stop_run(&receiver)? {
                stopped = true;
                break;
            }
            let line = line?;
            formatter.line(&line)?;
            stdout_output.push_str(&line);
            stdout_output.push('\n');
        }
    }

    let mut stderr_output = String::new();

    if !stopped {
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if check_stop_run(&receiver)? {
                    stopped = true;
                    break;
                }
                let line = line?;
                formatter.err_line(&line)?;
                stderr_output.push_str(&line);
                stderr_output.push('\n');
            }
        }
    }

    let status = if stopped {
        child.kill()?;
        None
    } else {
        Some(child.wait()?)
    };
    // print all at once so that threads do not overwrite each other
    // TODO: Check Status

    formatter.update()?;
    formatter.print();
    let stdout_plain = String::from_utf8(strip_ansi_escapes::strip(stdout_output.as_bytes()))
        .map_err(|e| FztError::from(e))?;
    let stderr_plain = String::from_utf8(strip_ansi_escapes::strip(stderr_output.as_bytes()))
        .map_err(|e| FztError::from(e))?;

    Ok(CaptureOutput {
        stopped,
        stdout: stdout_plain,
        stderr: stderr_plain,
        status,
    })
}
