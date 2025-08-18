use serde::{Deserialize, Serialize};

use crate::errors::FztError;

pub mod java;
pub mod python;
pub mod rust;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PythonDebugger {
    Pdb,
    Ipdb,
    IPython,
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
}

pub trait Runtime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
    ) -> Result<(), FztError>;
    fn name(&self) -> String;
}
