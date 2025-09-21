use std::process::Command;

use crate::{
    errors::FztError,
    runtime::{
        Debugger, DefaultFormatter, Runtime, RuntimeFormatter, utils::run_and_capture_print,
    },
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

pub struct CargoFormatter {
    number_tests: usize,
    failed_tests: Vec<(String, String)>,
    passed: usize,
    failed: usize,
    currently_failed: bool,
    running: bool,
    seconds: f64,
}

impl CargoFormatter {
    pub fn new(number_tests: usize) -> Self {
        Self {
            number_tests: number_tests,
            failed_tests: vec![],
            passed: 0,
            failed: 0,
            currently_failed: false,
            running: false,
            seconds: 0f64,
        }
    }
}

impl RuntimeFormatter for CargoFormatter {
    fn line(&mut self, line: &String) -> Result<(), FztError> {
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        let plain_line = String::from_utf8(plain_bytes).map_err(FztError::from)?;

        // Start running
        if plain_line == RUNNING_HEADER && !self.running {
            println!("Running {} tests", self.number_tests);
            self.running = true;
            return Ok(());
        }

        if !self.running {
            println!("{}", line);
            return Ok(());
        }

        // Test Passed
        if plain_line.ends_with("... ok") {
            println!("{}", line);
            self.passed += 1;
            return Ok(());
        }

        // Test Failed
        if plain_line.ends_with(TEST_FAILED_SUFFIX) {
            println!("{}", line);
            self.failed += 1;
            if let Some(test_name) = extract_test_name(&plain_line) {
                self.failed_tests
                    .push((test_name.to_string(), String::new()));
            } else {
                self.failed_tests
                    .push((plain_line.trim().to_string(), String::new()));
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
            if let Some((_, error_msg)) = self.failed_tests.last_mut() {
                if !error_msg.is_empty() {
                    error_msg.push('\n');
                }
                error_msg.push_str(&line);
            }
            return Ok(());
        }

        // Parse Time
        if let Some(secs) = parse_cargo_time(&plain_line) {
            self.seconds += secs;
        }

        Ok(())
    }

    fn finish(self) {
        if self.failed_tests.is_empty() {
            println!(
                "test result: \x1b[32mok\x1b[0m. {} passed; 0 failed; finished in {:.3}s",
                self.passed, self.seconds
            );
        } else {
            println!("\nfailures:");
            for (_, error) in &self.failed_tests {
                if !error.is_empty() {
                    println!("{}", error);
                }
            }
            println!("\nfailures:");
            for (test, _) in &self.failed_tests {
                println!("    {}", test);
            }
            println!(
                "\ntest result: \x1b[31mFAILED\x1b[0m. {} passed; {} failed; finished in {:.3}s",
                self.passed, self.failed, self.seconds
            );
        }
    }
}

#[derive(Default)]
pub struct CargoRuntime {}

impl Runtime for CargoRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        _debugger: &Option<Debugger>,
    ) -> Result<Option<String>, FztError> {
        let mut output = String::new();
        let number_tests = tests.len();
        let mut formatter = CargoFormatter::new(number_tests);
        for test in tests {
            let mut command = Command::new("unbuffer");
            command.arg("cargo");
            command.arg("test");
            command.arg("--color");
            command.arg("always");
            command.arg(test);
            command.arg("--");
            runtime_ags.iter().for_each(|arg| {
                command.arg(arg);
            });
            if verbose {
                let program = command.get_program().to_str().unwrap();
                let args: Vec<String> = command
                    .get_args()
                    .map(|arg| arg.to_str().unwrap().to_string())
                    .collect();
                println!("\n{} {}\n", program, args.as_slice().join(" "));
            }
            if verbose {
                output.push_str((run_and_capture_print(command, &mut DefaultFormatter)?).as_str());
            } else {
                output.push_str((run_and_capture_print(command, &mut formatter)?).as_str());
            }
        }
        if !verbose {
            formatter.finish();
        }
        Ok(Some(output))
    }

    fn name(&self) -> String {
        String::from("cargo")
    }
}
