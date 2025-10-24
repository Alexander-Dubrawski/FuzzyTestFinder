use crate::{
    errors::FztError,
    runtime::{
        Debugger, Runtime, RuntimeOutput, engine::Engine,
        rust::nextest::formatter::NextestFormatter,
    },
};
use colored::Colorize;
use std::{collections::HashMap, sync::mpsc::Receiver as StdReceiver};

#[derive(Default)]
pub struct NextestRuntime {}

impl Runtime for NextestRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_args: &[String],
        _debugger: &Option<Debugger>,
        receiver: Option<StdReceiver<String>>,
        run_coverage: bool,
    ) -> Result<RuntimeOutput, FztError> {
        let base_args = vec![
            "unbuffer",
            "cargo-nextest",
            "nextest",
            "run",
            "--message-format",
            "libtest-json",
            "--show-progress",
            "counter",
        ];
        if run_coverage {
            println!(
                "{}",
                &"--covered is not supported with nextest runtime use cargo instead."
                    .red()
                    .bold()
                    .to_string()
            );
            return Ok(RuntimeOutput::new_empty());
        }
        let envs = HashMap::from([("NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "1")]);
        let mut engine = Engine::new(Some("--".to_string()), None);
        engine.envs(&envs);
        engine.base_args(base_args.as_slice());
        engine.runtime_args(runtime_args);
        engine.execute_single_batch_sequential(
            false,
            receiver,
            tests,
            &mut NextestFormatter::new(),
            verbose,
        )
    }

    fn name(&self) -> String {
        String::from("nextest")
    }
}
