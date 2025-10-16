use crate::{
    FztError,
    utils::process::{FailedTest, OutputFormatter},
};

const TEST_PREFIX: &str = "test ";
const TEST_FAILED_SUFFIX: &str = " ... FAILED";
const FAILURES_HEADER: &str = "failures:";
const RUNNING_HEADER: &str = "running 1 test";

fn extract_test_name(line: &str) -> Option<&str> {
    let start_idx = line.find(TEST_PREFIX)? + TEST_PREFIX.len();
    let end_idx = line.find(TEST_FAILED_SUFFIX)?;
    Some(line[start_idx..end_idx].trim())
}

fn parse_cargo_time(line: &str) -> Option<f64> {
    if let Some(start_idx) = line.find("finished in ") {
        let time_part = &line[start_idx + "finished in ".len()..];
        let mut end_idx = 0;
        for (i, c) in time_part.chars().enumerate() {
            if !c.is_digit(10) && c != '.' {
                end_idx = i;
                break;
            }
        }
        let value_str = &time_part[..end_idx];
        let unit_str = time_part[end_idx..].trim();
        if let Ok(value) = value_str.parse::<f64>() {
            let seconds = match unit_str {
                "s" => value,
                "ms" => value / 1000.0,
                "µs" => value / 1_000_000.0,
                _ => return None,
            };
            return Some(seconds);
        }
    }
    None
}

#[derive(Clone, Default)]
pub struct CargoFormatter {
    failed_tests: Vec<FailedTest>,
    passed: usize,
    failed: usize,
    ignored: usize,
    measured: usize,
    currently_failed: bool,
    running: bool,
    seconds: f64,
    coverage: Vec<String>,
    print_output: String,
}

impl CargoFormatter {
    pub fn new() -> Self {
        Self {
            failed_tests: vec![],
            passed: 0,
            failed: 0,
            ignored: 0,
            measured: 0,
            currently_failed: false,
            running: false,
            seconds: 0f64,
            coverage: vec![],
            print_output: String::new(),
        }
    }
}

impl OutputFormatter for CargoFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        let plain_line = String::from_utf8(plain_bytes).map_err(FztError::from)?;

        // Start running
        if plain_line == RUNNING_HEADER && !self.running {
            self.running = true;
            return Ok(());
        }

        if plain_line.trim().starts_with("|| ") {
            let line = plain_line.trim_start_matches("||").trim();
            // Split at ':'
            let mut parts = line.splitn(2, ':');
            if let Some(coverage_report) = parts.next() {
                let path = coverage_report.trim();
                if let Some(numbers) = parts.next() {
                    let numbers = numbers.trim();
                    // Split at '/'
                    if let Some(coverage) = numbers.split('/').next() {
                        if coverage.trim().parse::<usize>().is_ok_and(|v| v > 0) {
                            self.coverage.push(path.to_string());
                        }
                    }
                }
            }
        }

        // Test Passed
        if plain_line.ends_with("... ok") {
            self.print_output.push_str(line);
            self.print_output.push_str("\n");
            self.passed += 1;
            return Ok(());
        }

        // Test Ignored
        if plain_line.ends_with("... ignored") {
            self.print_output.push_str(line);
            self.print_output.push_str("\n");
            self.ignored += 1;
            return Ok(());
        }

        // Test measured
        if plain_line.ends_with("... measured") {
            self.print_output.push_str(line);
            self.print_output.push_str("\n");
            self.measured += 1;
            return Ok(());
        }

        // Test Failed
        if plain_line.ends_with(TEST_FAILED_SUFFIX) {
            self.print_output.push_str(line);
            self.print_output.push_str("\n");
            self.failed += 1;
            if let Some(test_name) = extract_test_name(&plain_line) {
                self.failed_tests.push(FailedTest::new(test_name, ""));
            } else {
                self.failed_tests
                    .push(FailedTest::new(plain_line.trim(), ""));
            }
            return Ok(());
        }

        // Enter/Exit Failure Block
        if plain_line.starts_with(FAILURES_HEADER) {
            self.currently_failed = !self.currently_failed;
            return Ok(());
        }

        // Collect Failure Details
        if self.currently_failed {
            if let Some(failed_test) = self.failed_tests.last_mut() {
                if !failed_test.error_msg.is_empty() {
                    failed_test.error_msg.push('\n');
                }
                failed_test.error_msg.push_str(&line);
            }
            return Ok(());
        }

        // Parse Time
        if let Some(secs) = parse_cargo_time(&plain_line) {
            self.seconds += secs;
        }

        Ok(())
    }

    fn err_line(&mut self, _line: &str) -> Result<(), FztError> {
        Ok(())
    }
    fn add(&mut self, other: &CargoFormatter) {
        self.failed_tests.extend(other.failed_tests.clone());
        self.passed += other.passed;
        self.failed += other.failed;
        self.seconds += other.seconds;
        self.ignored += other.ignored;
        self.measured += other.measured;
    }

    fn finish(self) {
        if self.failed_tests.is_empty() {
            println!(
                "\ntest result: \x1b[32mok\x1b[0m. {} passed; 0 failed; {} measured; {} filtered out; finished in {:.3}s",
                self.passed, self.measured, self.ignored, self.seconds
            );
        } else {
            println!("\nfailures:");
            for failed_test in &self.failed_tests {
                if !failed_test.error_msg.is_empty() {
                    println!("{}", failed_test.error_msg);
                }
            }
            println!("\nfailures:");
            for failed_test in &self.failed_tests {
                println!("    {}", failed_test.name);
            }
            println!(
                "\ntest result: \x1b[93mFAILED\x1b[0m. {} passed; {} failed; {} measured; {} filtered out; finished in {:.3}s",
                self.passed, self.failed, self.measured, self.ignored, self.seconds
            );
        }
    }

    fn coverage(&self) -> Vec<String> {
        self.coverage.clone()
    }

    fn reset_coverage(&mut self) {
        self.coverage = vec![];
    }

    fn failed_tests(&self) -> Vec<FailedTest> {
        self.failed_tests.clone()
    }

    fn print(&self) {
        println!("{}", self.print_output);
    }

    fn update(&mut self) -> Result<(), FztError> {
        Ok(())
    }
}
