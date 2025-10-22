use std::{collections::HashMap, sync::mpsc::Receiver as StdReceiver};

use crate::{
    errors::FztError,
    runtime::{
        Debugger, OutputFormatter, Runtime, RuntimeOutput,
        engine::{Engine, TestItem},
    },
};

use super::formatter::CargoFormatter;

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
                    additional_command_envs: HashMap::new(),
                }
            })
            .collect();
        // Coverage only work with one thread at a time.
        // unbuffer merges stdout and stderr
        let mut engine = if run_coverage {
            let mut engine = Engine::new(Some("--".to_string()), Some(1));
            engine.base_args(&["unbuffer", "cargo", "tarpaulin", "--skip-clean", "--"]);
            engine
        } else {
            let mut engine = Engine::new(Some("--".to_string()), None);
            engine.base_args(&["unbuffer", "cargo", "test"]);
            engine
        };
        engine.runtime_args(runtime_args);

        let engine_output = engine.execute_per_item_parallel(receiver, test_items, verbose)?;

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
