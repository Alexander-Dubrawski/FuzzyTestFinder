use std::{collections::HashMap, process::Command};

use crate::{
    errors::FztError,
    runtime::{
        Debugger, DefaultFormatter, Runtime, RuntimeFormatter, utils::run_and_capture_print,
    },
};

fn parse_cargo_time(line: &str) -> Option<f64> {
    if let Some(start_idx) = line.find("finished in ") {
        let time_part = &line[start_idx + "finished in ".len()..];
        // Find where the numeric part ends (before the unit)
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
            // Normalize to seconds
            let seconds = match unit_str {
                "s" => value,
                "ms" => value / 1000.0,
                "µs" => value / 1_000_000.0,
                _ => return None, // unknown unit
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
        let plain_line = String::from_utf8(plain_bytes).map_err(|e| FztError::from(e))?;
        if plain_line == "running 1 test".to_string() && !self.running {
            println!("Running {} tests", self.number_tests);
            self.running = true;
        } else if !self.running {
            println!("{}", line);
        } else if plain_line.ends_with("... ok") {
            println!("{}", line);
            self.passed += 1;
        } else if plain_line.ends_with("... FAILED") {
            println!("{}", line);
            self.failed += 1;
            let start = "test ";
            let end = " ... FAILED";

            if let Some(start_idx) = line.find(start) {
                if let Some(end_idx) = line.find(end) {
                    let test_name = &line[start_idx + start.len()..end_idx].trim();
                    self.failed_tests
                        .push((test_name.to_string(), String::from("")));
                } else {
                    // fallback if end is not found
                    self.failed_tests
                        .push((line.trim().to_string(), String::from("")));
                }
            } else {
                // fallback if start is not found
                self.failed_tests
                    .push((line.trim().to_string(), String::from("")));
            }
        } else if !self.currently_failed && plain_line.starts_with("failures:") {
            self.currently_failed = true;
        } else if self.currently_failed && plain_line.starts_with("failures:") {
            self.currently_failed = false;
        } else if self.currently_failed {
            self.failed_tests.last_mut().map(|(_, error_msg)| {
                *error_msg = format!("{}\n{}", *error_msg, line);
            });
        } else {
            if let Some(secs) = parse_cargo_time(line) {
                self.seconds += secs;
            }
        }
        Ok(())
    }

    fn finish(self) {
        if self.failed_tests.is_empty() {
            println!(
                "test result: ok. {} passed;  0 failed; finished in {}s",
                self.passed, self.seconds
            )
        }
        println!("\nfailures:");
        for (_, error) in self.failed_tests.iter() {
            println!("{}", error);
        }
        println!("failures:\n");
        for (test, _) in self.failed_tests.iter() {
            println!("    {}", test);
        }
        println!(
            "\ntest result: FAILED. {} passed; {} failed; finished in {}s",
            self.passed, self.failed, self.seconds
        )
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
