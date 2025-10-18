use std::{collections::HashSet, fs, path::PathBuf};

use colored::Colorize;

use crate::{
    FztError,
    runtime::{
        FailedTest, OutputFormatter,
        python::{coverage_report::CoverageReport, test_report::TestReport},
    },
};

#[derive(Clone, Debug, Default)]
pub struct PytestCovFormatter {
    failed_tests: HashSet<FailedTest>,
    skipped_test: HashSet<String>,
    passed_tests: HashSet<String>,
    output: String,
    stderr: String,
    temp_cov_path: PathBuf,
    temp_report_log_path: PathBuf,
    passed: usize,
    failed: usize,
    skipped: usize,
    duration: f64,
    coverage: HashSet<String>,
    formatter_id: String,
}

impl PytestCovFormatter {
    pub fn new(temp_cov_path: PathBuf, temp_report_log_path: PathBuf, formatter_id: &str) -> Self {
        Self {
            failed_tests: HashSet::new(),
            skipped_test: HashSet::new(),
            passed_tests: HashSet::new(),
            temp_cov_path,
            temp_report_log_path,
            output: String::new(),
            stderr: String::new(),
            passed: 0,
            failed: 0,
            skipped: 0,
            duration: 0f64,
            coverage: HashSet::new(),
            formatter_id: formatter_id.to_string(),
        }
    }

    fn process_test_report(&mut self) -> Result<(), FztError> {
        if !self.temp_report_log_path.exists() {
            println!(
                "{}",
                format!(
                    "{} [{}] No test report found. STDERR: {}.",
                    &"FAILED".red().bold().to_string(),
                    self.formatter_id,
                    self.stderr
                )
            );
            return Ok(());
        }
        let json_str = fs::read_to_string(&self.temp_report_log_path)?;
        // If Fails thread crashes. And test just do not cover files.
        // TODO: Handle this case
        let report: TestReport = serde_json::from_str(&json_str)?;
        if let Some(failed) = report.summary.failed {
            self.failed += failed;
        }
        if let Some(passed) = report.summary.passed {
            self.passed += passed;
        }
        if let Some(skipped) = report.summary.skipped {
            self.skipped += skipped;
        }

        self.duration = report.duration;

        report.tests.iter().for_each(|test| {
            self.output.push_str(&test.nodeid);
            self.output.push(' ');
            if test.outcome == "failed" {
                self.failed_tests.insert(FailedTest {
                    name: test.nodeid.clone(),
                    error_msg: test
                        .call
                        .as_ref()
                        .and_then(|call| call.crash.as_ref())
                        .map_or(String::new(), |crash| crash.message.clone()),
                });
                self.output.push_str(&"FAILED".red().bold().to_string());
            } else if test.outcome == "skipped" {
                self.skipped_test.insert(test.nodeid.clone());
                self.output
                    .push_str(&"SKIPPED ".yellow().bold().to_string());
                if let Some(info) = test.setup.longrepr.as_ref() {
                    self.output.push_str(info.as_str());
                }
            } else if test.outcome == "passed" {
                self.passed_tests.insert(test.nodeid.clone());
                self.output.push_str(&"PASSED".green().bold().to_string());
            }
            self.output.push('\n');
        });

        // Add failures section if there are any
        let failures: Vec<_> = report
            .tests
            .iter()
            .filter(|t| t.outcome == "failed")
            .collect();
        if !failures.is_empty() {
            self.output.push('\n');
            self.output.push_str("FAILURES\n");

            for test in failures {
                self.output.push('\n');
                self.output.push_str(&test.nodeid);
                self.output.push('\n');
                if let Some(call) = &test.call {
                    if let Some(longrepr) = call.longrepr.as_ref() {
                        self.output.push('\n');
                        self.output.push_str(longrepr.as_str());
                        self.output.push('\n');
                    } else if let Some(crash) = call.crash.as_ref() {
                        self.output.push('\n');
                        self.output.push_str(&format!(
                            "{}:{}: {}\n",
                            crash.path, crash.lineno, crash.message
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn process_coverage_report(&mut self) -> Result<(), FztError> {
        // If test failed it does not exist
        if self.temp_cov_path.exists() {
            let json_str = fs::read_to_string(&self.temp_cov_path)?;
            let report: CoverageReport = serde_json::from_str(&json_str)?;
            self.coverage = HashSet::from_iter(
                report
                    .files
                    .into_iter()
                    .filter(|(_, file)| file.summary.percent_covered > 0.0)
                    .map(|(filepath, _)| filepath),
            );
        }
        Ok(())
    }
}

impl OutputFormatter for PytestCovFormatter {
    fn line(&mut self, _line: &str) -> Result<(), crate::FztError> {
        Ok(())
    }

    fn err_line(&mut self, line: &str) -> Result<(), crate::FztError> {
        self.stderr.push_str(line);
        self.stderr.push('\n');
        Ok(())
    }

    fn add(&mut self, other: &Self) {
        self.failed_tests.extend(other.failed_tests.clone());
        self.skipped_test.extend(other.skipped_test.clone());
        self.passed += other.passed;
        self.failed += other.failed;
        self.skipped += other.skipped;
        self.duration += other.duration;
    }

    fn finish(self) {
        println!("\n");
        println!("Result in ({:.2}s):", self.duration);
        if self.failed > 0 {
            println!(
                "{}",
                format!("    {} failed", self.failed)
                    .red()
                    .bold()
                    .to_string()
            );
        }
        if self.passed > 0 {
            println!(
                "{}",
                format!("    {} passed", self.passed).green().to_string()
            );
        }
        if self.skipped > 0 {
            println!(
                "{}",
                format!("    {} skipped", self.skipped).yellow().to_string()
            );
        }
    }

    fn coverage(&self) -> Vec<String> {
        self.coverage.iter().cloned().collect()
    }

    fn reset_coverage(&mut self) {
        self.coverage = HashSet::new();
    }

    fn failed_tests(&self) -> Vec<FailedTest> {
        self.failed_tests.iter().cloned().collect()
    }

    fn update(&mut self) -> Result<(), FztError> {
        self.process_test_report()?;
        self.process_coverage_report()
    }

    fn print(&self) {
        print!("{}", self.output);
    }

    fn skipped(&self) -> bool {
        self.skipped > 0
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_no_coverage() {
        todo!()
    }

    #[test]
    fn parse_with_coverage() {
        todo!()
    }
}
