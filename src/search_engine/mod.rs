use crate::{errors::FztError, tests::Tests};

pub mod fzf;

pub trait SearchEngine {
    // TODO: Take ref
    fn get_tests_to_run(&self, all_test: impl Tests) -> Result<Vec<String>, FztError>;
    fn get_from_history(&self, history: Vec<Vec<String>>) -> Result<Vec<String>, FztError>;
    fn name(&self) -> String;
}
