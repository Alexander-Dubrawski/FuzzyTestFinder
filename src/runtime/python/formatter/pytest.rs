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
        self.failed_tests.iter().cloned().collect()
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
    use std::fs;
    use tempfile::TempDir;

    fn create_test_report_json(
        passed: usize,
        failed: usize,
        skipped: usize,
        duration: f64,
    ) -> String {
        let mut tests = Vec::new();

        for i in 0..passed {
            tests.push(format!(
                r#"{{
                    "nodeid": "test_file.py::test_passed_{}",
                    "lineno": 10,
                    "outcome": "passed",
                    "keywords": [],
                    "setup": {{
                        "duration": 0.01,
                        "outcome": "passed"
                    }},
                    "call": {{
                        "duration": 0.1,
                        "outcome": "passed"
                    }},
                    "teardown": {{
                        "duration": 0.01,
                        "outcome": "passed"
                    }}
                }}"#,
                i
            ));
        }

        for i in 0..failed {
            tests.push(format!(
                r#"{{
                    "nodeid": "test_file.py::test_failed_{}",
                    "lineno": 20,
                    "outcome": "failed",
                    "keywords": [],
                    "setup": {{
                        "duration": 0.01,
                        "outcome": "passed"
                    }},
                    "call": {{
                        "duration": 0.1,
                        "outcome": "failed",
                        "longrepr": "AssertionError: Test failed",
                        "crash": {{
                            "path": "test_file.py",
                            "lineno": 42,
                            "message": "assertion failed"
                        }}
                    }},
                    "teardown": {{
                        "duration": 0.01,
                        "outcome": "passed"
                    }}
                }}"#,
                i
            ));
        }

        for i in 0..skipped {
            tests.push(format!(
                r#"{{
                    "nodeid": "test_file.py::test_skipped_{}",
                    "lineno": 30,
                    "outcome": "skipped",
                    "keywords": [],
                    "setup": {{
                        "duration": 0.0,
                        "outcome": "skipped",
                        "longrepr": "Skipped: reason for skipping"
                    }},
                    "teardown": {{
                        "duration": 0.0,
                        "outcome": "passed"
                    }}
                }}"#,
                i
            ));
        }

        let total = passed + failed + skipped;

        format!(
            r#"{{
                "created": 1234567890.0,
                "duration": {},
                "exitcode": 0,
                "root": "/test/path",
                "environment": {{}},
                "summary": {{
                    "passed": {},
                    "failed": {},
                    "skipped": {},
                    "total": {},
                    "collected": {}
                }},
                "collectors": [],
                "tests": [{}]
            }}"#,
            duration,
            if passed > 0 {
                passed.to_string()
            } else {
                "null".to_string()
            },
            if failed > 0 {
                failed.to_string()
            } else {
                "null".to_string()
            },
            if skipped > 0 {
                skipped.to_string()
            } else {
                "null".to_string()
            },
            total,
            total,
            tests.join(",")
        )
    }

    #[test]
    fn collect_failed_shiny_pytest() {
        let temp_dir = TempDir::new().unwrap();
        let report_path = temp_dir.path().join("report.json");

        // Create test report with multiple failed tests
        let test_report = create_test_report_json(5, 3, 1, 2.5);
        fs::write(&report_path, test_report).unwrap();

        let mut formatter = PytestFormatter::new(report_path);

        // Update should parse the report and collect failed tests
        formatter.update().unwrap();

        let failed_tests = formatter.failed_tests();

        // Should have exactly 3 failed tests
        assert_eq!(failed_tests.len(), 3);

        // Verify the failed test names
        assert!(
            failed_tests
                .iter()
                .any(|t| t.name == "test_file.py::test_failed_0")
        );
        assert!(
            failed_tests
                .iter()
                .any(|t| t.name == "test_file.py::test_failed_1")
        );
        assert!(
            failed_tests
                .iter()
                .any(|t| t.name == "test_file.py::test_failed_2")
        );

        // Verify error messages are captured
        for failed_test in &failed_tests {
            assert_eq!(failed_test.error_msg, "assertion failed");
        }
    }

    #[test]
    fn collect_no_failures() {
        let temp_dir = TempDir::new().unwrap();
        let report_path = temp_dir.path().join("report.json");

        // Create test report with only passed tests
        let test_report = create_test_report_json(5, 0, 0, 1.0);
        fs::write(&report_path, test_report).unwrap();

        let mut formatter = PytestFormatter::new(report_path);
        formatter.update().unwrap();

        let failed_tests = formatter.failed_tests();

        // Should have no failed tests
        assert_eq!(failed_tests.len(), 0);
    }

    #[test]
    fn no_report_file() {
        let temp_dir = TempDir::new().unwrap();
        let report_path = temp_dir.path().join("nonexistent.json");

        let mut formatter = PytestFormatter::new(report_path);

        // Should not error when report file doesn't exist
        assert!(formatter.update().is_ok());

        // Should have no failed tests
        assert_eq!(formatter.failed_tests().len(), 0);
    }
}
