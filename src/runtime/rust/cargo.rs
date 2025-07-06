use std::process::Command;

use crate::{errors::FztError, runtime::Runtime};

#[derive(Default)]
pub struct CargoRuntime {}

impl Runtime for CargoRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
    ) -> Result<(), FztError> {
        let mut command = Command::new("cargo");
        command.arg("test");
        tests.into_iter().for_each(|test| {
            command.arg(test);
        });
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
        command.status()?;
        Ok(())
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
