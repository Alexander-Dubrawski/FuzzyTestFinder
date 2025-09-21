use serde::{Deserialize, Serialize};

use crate::errors::FztError;

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

pub trait RuntimeFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError>;
    fn finish(self);
}

pub struct DefaultFormatter;
impl RuntimeFormatter for DefaultFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{}", line);
        Ok(())
    }
    fn finish(self) {}
}

pub trait Runtime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
    ) -> Result<Option<String>, FztError>;
    fn name(&self) -> String;
}
