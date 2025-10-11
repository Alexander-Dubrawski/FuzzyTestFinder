use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::sync::mpsc::Receiver;

use crate::errors::FztError;

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

pub trait Runtime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
        receiver: Option<Receiver<String>>,
        coverage: &mut Option<HashMap<String, Vec<String>>>,
    ) -> Result<Option<String>, FztError>;
    fn name(&self) -> String;
}
