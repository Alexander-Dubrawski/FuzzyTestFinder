use crate::{errors::FztError, parser::Tests};

pub mod fzf;

pub trait SearchEngine {
    fn get_tests_to_run(&self, all_test: impl Tests) -> Result<Vec<String>, FztError>;
    fn get_recent_history_command(&self) -> Result<Vec<String>, FztError>;
    fn get_from_history(&self) -> Result<Vec<String>, FztError>;
}
