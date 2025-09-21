use std::process::Command;

use crate::{
    errors::FztError,
    runtime::{Debugger, DefaultFormatter, Runtime, utils::run_and_capture_print},
};

#[derive(Default)]
pub struct GradleRuntime {}

impl Runtime for GradleRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        _debugger: &Option<Debugger>,
    ) -> Result<Option<String>, FztError> {
        let mut command = Command::new("unbuffer");
        command.arg("./gradlew");
        command.arg("-i");
        runtime_ags.iter().for_each(|arg| {
            command.arg(arg);
        });
        command.arg("test");
        tests.into_iter().for_each(|test| {
            command.arg("--tests");
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
        let output = run_and_capture_print(command, &mut DefaultFormatter)?;
        Ok(Some(output))
    }

    fn name(&self) -> String {
        String::from("gradle")
    }
}
