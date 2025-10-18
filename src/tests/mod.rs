use std::collections::HashMap;

use crate::{errors::FztError, utils::process::FailedTest};

pub mod java;
pub mod python;
pub mod rust;
pub mod test_provider;

pub trait Test {
    fn runtime_argument(&self) -> String;
    fn name(&self) -> String;
    fn file_path(&self) -> String;
}

pub trait Tests {
    fn to_json(&self) -> Result<String, FztError>;
    fn tests(&self) -> Vec<impl Test>;
    fn tests_failed(&self) -> Vec<impl Test>;
    fn update(&mut self) -> Result<bool, FztError>;
    fn update_file_coverage(
        &mut self,
        coverage: &HashMap<String, Vec<String>>,
    ) -> Result<bool, FztError>;
    fn get_covered_tests(&self) -> Vec<impl Test>;
    fn update_failed(&mut self, failed_tests_output: &[FailedTest]) -> bool;
}
