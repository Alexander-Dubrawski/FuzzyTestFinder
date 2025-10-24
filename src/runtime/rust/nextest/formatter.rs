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
                let test_name_parts: Vec<&str> = report.name.splitn(2, "$").collect();
                let err_msg = if let Some(msg) = report.stdout {
                    msg
                } else {
                    "No output captured.".to_string()
                };
                if test_name_parts.len() == 2 {
                    self.failed_tests
                        .insert(FailedTest::new(test_name_parts[1], err_msg.as_str()));
                } else {
                    self.failed_tests
                        .insert(FailedTest::new(report.name.as_str(), err_msg.as_str()));
                }
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

    fn add(&mut self, other: &Self) {
        for failed_test in &other.failed_tests {
            self.failed_tests.insert(failed_test.clone());
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nextest_formatter_error_line_parsing() {
        let test_line = r#"
 Nextest run ID a09c7739-fde2-489e-abb8-86cf0bee9aea with nextest profile: default
    Starting 23 tests across 2 binaries
        PASS [   1.513s] (23/23) FzT tests::java::java_test::tests::collect_tests
{"type":"test","name":"Foo::Boo$tests::example_test::test_case_1","event":"failed","stdout":"panicked at"}
        "#;
        let expected = vec![FailedTest::new(
            "tests::example_test::test_case_1",
            "panicked at",
        )];

        let mut formatter = NextestFormatter::new();
        for line in test_line.lines() {
            formatter.line(line).unwrap();
        }

        let failed_tests = formatter.failed_tests();
        assert_eq!(failed_tests, expected);
    }

    #[test]
    fn test_nextest_formatter_no_error_line_parsing() {
        let test_line = r#"
 Nextest run ID a09c7739-fde2-489e-abb8-86cf0bee9aea with nextest profile: default
    Starting 23 tests across 2 binaries
        PASS [   1.513s] (23/23) FzT tests::java::java_test::tests::collect_tests
{"type":"status"}
        "#;

        let mut formatter = NextestFormatter::new();
        for line in test_line.lines() {
            formatter.line(line).unwrap();
        }
        let failed_tests = formatter.failed_tests();
        assert!(failed_tests.is_empty());
    }
}
