use std::{collections::HashMap, sync::mpsc::Receiver};

use crate::{
    errors::FztError,
    runtime::{Debugger, Runtime, engine::Engine},
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
        _coverage: &mut Option<HashMap<String, Vec<String>>>,
    ) -> Result<Option<String>, FztError> {
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
        engine.execute_single_batch(false, receiver, verbose)
    }

    fn name(&self) -> String {
        String::from("gradle")
    }
}
