use std::{collections::HashMap, sync::mpsc::Receiver as StdReceiver};

use crate::{
    errors::FztError,
    runtime::{
        Debugger, OutputFormatter, Runtime, RuntimeOutput,
        engine::{Engine, TestItem},
    },
};

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
        todo!()
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
