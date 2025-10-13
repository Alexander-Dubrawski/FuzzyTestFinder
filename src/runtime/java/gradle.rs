use std::sync::mpsc::Receiver;

use crate::{
    errors::FztError,
    runtime::{Debugger, Runtime, RuntimeOutput, engine::Engine},
    utils::process::DefaultFormatter,
};

const JUNIT_FAILURE_EXIT_CODE: i32 = 1;

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
        _run_coverage: bool,
    ) -> Result<RuntimeOutput, FztError> {
        let mut engine = Engine::new("", DefaultFormatter, None, JUNIT_FAILURE_EXIT_CODE);
        // unbuffer merges stdout and stderr
        engine.base_args(&["unbuffer", "./gradlew", "-i"]);
        engine.base_args_string(runtime_ags);
        engine.base_arg("test");
        engine.tests(
            tests
                .into_iter()
                .map(|test| vec![String::from("--tests"), test])
                .flatten()
                .collect::<Vec<String>>()
                .as_slice(),
        );
        engine.execute_single_batch_sequential(false, receiver, verbose)
    }

    fn name(&self) -> String {
        String::from("gradle")
    }
}
