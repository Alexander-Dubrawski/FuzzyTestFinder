use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    FztError,
    runtime::{FailedTest, OutputFormatter, python::test_report::TestReport},
};
use colored::Colorize;

#[derive(Clone, Debug, Default)]
pub struct PytestFormatter {
    failed_tests: HashSet<FailedTest>,
    temp_report_log_path: PathBuf,
}

impl PytestFormatter {
    pub fn new(temp_report_log_path: PathBuf) -> Self {
        Self {
            temp_report_log_path,
            failed_tests: HashSet::new(),
        }
    }
}

impl OutputFormatter for PytestFormatter {
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
        vec![]
    }

    fn update(&mut self) -> Result<(), FztError> {
        if !self.temp_report_log_path.exists() {
            println!(
                "{}",
                format!(
                    "{} No test report found.",
                    &"FAILED".red().bold().to_string(),
                )
            );
            return Ok(());
        }
        let json_str = fs::read_to_string(&self.temp_report_log_path)?;
        let report: TestReport = serde_json::from_str(&json_str)?;
        report.tests.iter().for_each(|test| {
            if test.outcome == "failed" {
                self.failed_tests.insert(FailedTest {
                    name: test.nodeid.clone(),
                    error_msg: test
                        .call
                        .as_ref()
                        .and_then(|call| call.crash.as_ref())
                        .map_or(String::new(), |crash| crash.message.clone()),
                });
            }
        });
        Ok(())
    }

    fn print(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn collect_failed_shiny_pytest() {
        // TODO: Implement test
    }
}
