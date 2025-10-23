use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    FztError,
    runtime::{FailedTest, OutputFormatter},
};
use colored::Colorize;

#[derive(Clone, Debug, Default)]
pub struct NextestFormatter {
    failed_tests: HashSet<FailedTest>,
    temp_report_log_path: PathBuf,
}

impl NextestFormatter {
    pub fn new(temp_report_log_path: PathBuf) -> Self {
        Self {
            temp_report_log_path,
            failed_tests: HashSet::new(),
        }
    }
}

impl OutputFormatter for NextestFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{}", line);
        Ok(())
    }

    fn err_line(&mut self, line: &str) -> Result<(), crate::FztError> {
        println!("{}", line);
        Ok(())
    }

    fn add(&mut self, _other: &Self) {}

    fn finish(self) {}

    fn coverage(&self) -> Vec<String> {
        vec![]
    }

    fn skipped(&self) -> bool {
        false
    }

    fn reset_coverage(&mut self) {}

    fn failed_tests(&self) -> Vec<FailedTest> {
        self.failed_tests.iter().cloned().collect()
    }

    fn update(&mut self) -> Result<(), FztError> {
        Ok(())
    }

    fn print(&self) {}
}
