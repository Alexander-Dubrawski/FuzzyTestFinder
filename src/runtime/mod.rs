use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::sync::mpsc::Receiver;

use crate::{errors::FztError, utils::process::FailedTest};

mod engine;
pub mod java;
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
