use std::process::Command;

use crate::{
    errors::FztError,
    runtime::{Debugger, Runtime, utils::run_and_capture_print},
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
        let mut output = String::new();
        for test in tests {
            let mut command = Command::new("unbuffer");
            command.arg("cargo");
            command.arg("test");
            command.arg("--color");
            command.arg("always");
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
            }
            output.push_str((run_and_capture_print(command)?).as_str());
        }
        Ok(output)
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
