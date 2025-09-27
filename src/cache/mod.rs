use std::{fs::File, io::BufReader};

use manager::HistoryGranularity;

use crate::errors::FztError;

pub mod helper;
pub mod manager;
pub mod types;

pub trait Cache {
    fn get_entry(&self) -> Result<Option<BufReader<File>>, FztError>;
    fn add_entry(&self, entry: &str) -> Result<(), FztError>;
    fn clear_cache(&self) -> Result<(), FztError>;
    fn clear_history(&self) -> Result<(), FztError>;
    fn update_history(
        &self,
        selection: &[String],
        granularity: &HistoryGranularity,
    ) -> Result<(), FztError>;
    fn recent_history_command(
        &self,
        granularity: &HistoryGranularity,
    ) -> Result<Vec<String>, FztError>;
    fn history(&self, granularity: &HistoryGranularity) -> Result<Vec<Vec<String>>, FztError>;
}
