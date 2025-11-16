use std::sync::mpsc::Receiver;

use crate::{
    errors::FztError,
    runtime::{
        Debugger, Runtime, RuntimeOutput, engine::Engine,
        java::formatter::gradle_formatter::GradleFormatter,
    },
};
use colored::Colorize;

pub struct GradleRuntime {
    gradle_command: String,
}

impl GradleRuntime {
    pub fn new(gradle_command: &str) -> Self {
        GradleRuntime {
            gradle_command: gradle_command.to_string(),
        }
    }
}

impl Runtime for GradleRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        _debugger: &Option<Debugger>,
        receiver: Option<Receiver<String>>,
        run_coverage: bool,
    ) -> Result<RuntimeOutput, FztError> {
        if run_coverage {
            println!(
                "{}",
                &"--covered is not supported for java."
                    .red()
                    .bold()
                    .to_string()
            );
            return Ok(RuntimeOutput::new_empty());
        }
        let mut engine = Engine::new(None, None);
        // unbuffer merges stdout and stderr
        engine.base_args(&["unbuffer", self.gradle_command.as_str(), "-i"]);
        engine.base_args_string(runtime_ags);
        engine.base_arg("test");
        let formatted_tests = tests
            .into_iter()
            .map(|test| vec![String::from("--tests"), format!("\"{}\"", test)])
            .flatten()
            .collect::<Vec<String>>();
        engine.execute_single_batch_sequential(
            false,
            receiver,
            formatted_tests,
            &mut GradleFormatter::new(),
            verbose,
        )
    }

    fn name(&self) -> String {
        String::from("gradle")
    }
}
