use std::{process::ExitStatus, sync::mpsc::Receiver as StdReceiver};

use crate::{
    errors::FztError,
    runtime::{
        Debugger, Runtime, RuntimeOutput,
        engine::{Engine, TestItem},
    },
    utils::process::OutputFormatter,
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
        run_coverage: bool,
    ) -> Result<RuntimeOutput, FztError> {
        let test_items: Vec<TestItem<CargoFormatter>> = tests
            .into_iter()
            .map(|test| {
                let formatter = CargoFormatter::new();
                TestItem {
                    test_name: test,
                    formatter,
                    additional_base_args: vec![],
                    additional_runtime_args: vec![],
                }
            })
            .collect();
        let mut engine = Engine::new("--", None);
        // unbuffer merges stdout and stderr
        if run_coverage {
            engine.base_args(&["unbuffer", "cargo", "tarpaulin", "--skip-clean", "--"]);
        } else {
            engine.base_args(&["unbuffer", "cargo", "test"]);
        };
        engine.runtime_args(runtime_args);

        let engine_output = engine.execute_per_item_parallel(receiver, test_items, verbose)?;

        if !engine_output.success(RUST_TEST_FAILURE_EXIT_CODE) {
            let error_msg: Vec<(String, Option<ExitStatus>)> = engine_output
                .get_error_status_test_output(RUST_TEST_FAILURE_EXIT_CODE)
                .into_iter()
                .map(|test_output| (test_output.test, test_output.output.status))
                .collect();

            println!(
                "WARNING: Some tests failed with exit codes: \n{:?}",
                error_msg
            );
        }

        engine_output.merge_formatters().finish();

        if engine_output.stopped() {
            Ok(RuntimeOutput::new_empty())
        } else {
            Ok(RuntimeOutput::from_engine_output(&engine_output))
        }
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
