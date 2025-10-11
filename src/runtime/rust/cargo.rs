use std::collections::HashMap;

use std::sync::mpsc::Receiver as StdReceiver;

use crate::{
    errors::FztError,
    runtime::{Debugger, Runtime, engine::Engine},
};

use super::formatter::CargoFormatter;

const RUST_TEST_FAILURE_EXIT_CODE: i32 = 101;

#[derive(Default)]
pub struct CargoRuntime {}

impl Runtime for CargoRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_args: &[String],
        _debugger: &Option<Debugger>,
        receiver: Option<StdReceiver<String>>,
        coverage: &mut Option<HashMap<String, Vec<String>>>,
    ) -> Result<Option<String>, FztError> {
        let mut engine = Engine::new(
            "--",
            CargoFormatter::new(),
            None,
            RUST_TEST_FAILURE_EXIT_CODE,
        );
        // unbuffer merges stdout and stderr
        if coverage.is_some() {
            engine.base_args(&["unbuffer", "cargo", "tarpaulin", "--skip-clean", "--"]);
        } else {
            engine.base_args(&["unbuffer", "cargo", "test"]);
        };
        engine.runtime_args(runtime_args);
        engine.tests(tests.as_slice());

        engine.execute_per_item(coverage, receiver, verbose)
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
