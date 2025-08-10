use crate::{errors::FztError, runner::Preview};

pub mod fzf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Appened {
    Test,
    File,
    Directory,
    RunTime,
    List,
    Done,
}

pub trait SearchEngine {
    fn get_tests_to_run(
        &self,
        all_test: &[&str],
        preview: &Option<Preview>,
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError>;
    fn get_from_history(
        &self,
        history: &[Vec<String>],
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError>;
    fn name(&self) -> String;
    fn appened(&self) -> Result<Appened, FztError>;
}
