use crate::errors::FztError;

pub mod java;
pub mod python;
pub mod rust;

pub trait Runtime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
    ) -> Result<(), FztError>;
    fn name(&self) -> String;
}
