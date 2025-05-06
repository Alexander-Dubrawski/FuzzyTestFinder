use crate::errors::FztError;

pub mod python;

pub trait Runtime {
    fn run_tests(&self, tests: Vec<String>, verbose: bool) -> Result<(), FztError>;
}
