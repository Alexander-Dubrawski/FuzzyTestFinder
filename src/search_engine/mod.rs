use crate::errors::FztError;

pub mod fzf;

pub trait SearchEngine {
    fn get_tests_to_run(&self, all_test: &[&str], preview: bool) -> Result<Vec<String>, FztError>;
    fn get_from_history(&self, history: &[Vec<String>]) -> Result<Vec<String>, FztError>;
    fn name(&self) -> String;
}
