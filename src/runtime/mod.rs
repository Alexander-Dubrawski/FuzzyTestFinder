use std::collections::HashMap;

use engine::EngineOutput;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Receiver;

use crate::errors::FztError;

mod engine;
pub mod java;
mod process;
pub mod python;
pub mod rust;
mod utils;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PythonDebugger {
    Pdb,
    Ipdb,
    IPython,
    Pudb,
    WebPdb,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RustDebugger {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JavaDebugger {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Debugger {
    Python(PythonDebugger),
    Rust(RustDebugger),
    Java(JavaDebugger),
    Select,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct FailedTest {
    pub name: String,
    pub error_msg: String,
}

impl FailedTest {
    pub fn new(name: &str, error_msg: &str) -> Self {
        Self {
            name: name.to_string(),
            error_msg: error_msg.to_string(),
        }
    }
}

pub trait OutputFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError>;
    fn err_line(&mut self, line: &str) -> Result<(), FztError>;
    fn add(&mut self, other: &Self);
    fn finish(self);
    fn coverage(&self) -> Vec<String>;
    fn skipped(&self) -> bool;
    fn reset_coverage(&mut self);
    fn failed_tests(&self) -> Vec<FailedTest>;
    fn update(&mut self) -> Result<(), FztError>;
    fn print(&self);
}

pub struct RuntimeOutput {
    pub failed_tests: Vec<FailedTest>,
    pub output: Option<String>,
    pub coverage: HashMap<String, Vec<String>>,
}

impl RuntimeOutput {
    pub fn new_empty() -> Self {
        Self {
            failed_tests: vec![],
            output: None,
            coverage: HashMap::new(),
        }
    }

    pub fn from_engine_output<F: OutputFormatter + Clone + Sync + Send + Default>(
        engine_output: &EngineOutput<F>,
    ) -> Self {
        Self {
            failed_tests: engine_output.failed_tests(),
            output: Some(engine_output.merge_stdout()),
            coverage: engine_output.coverage(),
        }
    }
}

pub trait Runtime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
        receiver: Option<Receiver<String>>,
        run_coverage: bool,
    ) -> Result<RuntimeOutput, FztError>;
    fn name(&self) -> String;
}
