use crate::{
    errors::FztError,
    runtime::{
        Debugger, Runtime, RuntimeOutput,
        engine::{Engine, TestItem},
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
        let base_args = vec!["unbuffer", "cargo", "nextest", "run"];
        if run_coverage {
            println!(
                "{}",
                &"--covered is not supported with nextest runtime use cargo instead."
                    .red()
                    .bold()
                    .to_string()
            );
        }
        let mut engine = Engine::new(Some("--".to_string()), None);
        let rep_dir = tempfile::tempdir()?;
        let rep_path = rep_dir.path().join("report.json").to_path_buf();
        engine.base_args(base_args.as_slice());
        engine.runtime_args(runtime_args);
        engine.execute_single_batch_sequential(
            false,
            receiver,
            tests,
            &mut NextestFormatter::new(rep_path),
            verbose,
        )
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
