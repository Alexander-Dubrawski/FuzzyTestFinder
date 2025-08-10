use std::str::FromStr;

use crate::{errors::FztError, runner::Preview};

pub mod fzf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Appened {
    Test,
    File,
    Directory,
    RunTime,
    Done,
}

impl FromStr for Appened {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "Test" => Ok(Appened::Test),
            "File" => Ok(Appened::File),
            "Directory" => Ok(Appened::Directory),
            "RunTime" => Ok(Appened::RunTime),
            "Done" => Ok(Appened::Done),
            _ => Err(format!("Invalid selection: {}", s)),
        }
    }
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
    fn appened(&self, selected_items: &str) -> Result<Appened, FztError>;
}
