use std::{ffi::OsStr, process::Command};

use crate::{errors::FztError, runtime::Runtime};

#[derive(Default)]
pub struct PytestRuntime {}

impl Runtime for PytestRuntime {
    fn run_tests(&self, tests: Vec<String>, verbose: bool, debug: bool) -> Result<(), FztError> {
        let mut command = Command::new("python");
        command.arg("-m");
        command.arg("pytest");
        if debug {
            command.arg("--pdb");
        }
        tests.into_iter().for_each(|test| {
            command.arg(test);
        });
        if verbose {
            let program = command.get_program().to_str().unwrap();
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
        }
        command.status()?;
        Ok(())
    }
}
