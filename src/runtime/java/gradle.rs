use std::{collections::HashMap, process::Command, sync::mpsc::Receiver};

use crate::{
    errors::FztError,
    runtime::{Debugger, Runtime},
    utils::process::{DefaultFormatter, run_and_capture_print},
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
        receiver: Option<Receiver<String>>,
        _coverage: &mut Option<HashMap<String, Vec<String>>>,
    ) -> Result<Option<String>, FztError> {
        // Merge stdout and stderr
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
        let output = run_and_capture_print(command, &mut DefaultFormatter, receiver)?;
        if output.stopped {
            Ok(None)
        } else {
            Ok(Some(output.stdout))
        }
    }

    fn name(&self) -> String {
        String::from("gradle")
    }
}
