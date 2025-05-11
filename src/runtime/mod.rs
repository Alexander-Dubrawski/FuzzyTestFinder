use crate::errors::FztError;

pub mod python;

pub trait Runtime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
    ) -> Result<(), FztError>;
}
