use std::collections::HashSet;

use crate::{
    FztError,
    runtime::{FailedTest, OutputFormatter},
};

use super::test_report::TestReport;

#[derive(Clone, Debug, Default)]
pub struct NextestFormatter {
    failed_tests: HashSet<FailedTest>,
}

impl NextestFormatter {
    pub fn new() -> Self {
        Self {
            failed_tests: HashSet::new(),
        }
    }
}

impl OutputFormatter for NextestFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        let plain_line = String::from_utf8(plain_bytes).map_err(FztError::from)?;
        if plain_line.starts_with("{\"type\":\"test\"") {
            let report: TestReport = serde_json::from_str(&plain_line)?;
            if report.event == "failed" {
                let test_name: Vec<&str> = report.name.split("$").collect();
                let err_msg = if let Some(msg) = report.stdout {
                    msg
                } else {
                    "No output captured.".to_string()
                };
                self.failed_tests
                    .insert(FailedTest::new(test_name[1], err_msg.as_str()));
            }
        }
        if !plain_line.starts_with("{\"type\"") {
            println!("{}", line);
        }
        Ok(())
    }

    fn err_line(&mut self, line: &str) -> Result<(), crate::FztError> {
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        let plain_line = String::from_utf8(plain_bytes).map_err(FztError::from)?;
        if !plain_line.starts_with("{\"type\"") {
            println!("{}", line);
        }
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
