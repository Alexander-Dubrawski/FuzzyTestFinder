use std::process::Command;

use crate::{
    errors::FztError,
    runtime::{utils::run_and_capture, Debugger, Runtime},
};

#[derive(Default)]
pub struct CargoRuntime {}

impl Runtime for CargoRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        _debugger: &Option<Debugger>,
    ) -> Result<String, FztError> {
        if verbose {
            println!("INFO: Verbose mode enabled, does not capture failed tests");
        }
        let mut output = String::new();
        for test in tests {
            let mut command = Command::new("cargo");
            command.arg("test");
            command.arg(test);
            command.arg("--");
            runtime_ags.iter().for_each(|arg| {
                command.arg(arg);
            });
            if verbose {
                let program = command.get_program().to_str().unwrap();
                let args: Vec<String> = command
                    .get_args()
                    .map(|arg| arg.to_str().unwrap().to_string())
                    .collect();
                println!("\n{} {}\n", program, args.as_slice().join(" "));
                command.status()?;
            } else {
                output.push_str((run_and_capture(command)?).as_str());
            }
        }
        Ok(output)
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
