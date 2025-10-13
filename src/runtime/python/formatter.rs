use std::collections::HashSet;

use crate::{
    FztError,
    utils::process::{FailedTest, OutputFormatter},
};

const HEADER_END: &str = "collected";
const HEADER_END_ALT: &str = "collected";
const NEW_SECTION: &str = "====";

#[derive(Clone, Debug)]
pub struct PytestFormatter {
    failed_tests: HashSet<FailedTest>,
    skipped_test: HashSet<String>,
    passed: usize,
    error: usize,
    skipped: usize,
    in_header: bool,
    in_body: bool,
    in_coverage: bool,
    in_footer: bool,
    seconds: f64,
    coverage: HashSet<String>,
    print_output: String,
}

impl PytestFormatter {
    pub fn new() -> Self {
        Self {
            failed_tests: HashSet::new(),
            skipped_test: HashSet::new(),
            passed: 0,
            error: 0,
            skipped: 0,
            in_header: true,
            in_body: false,
            in_coverage: false,
            in_footer: false,
            seconds: 0f64,
            coverage: HashSet::new(),
            print_output: String::new(),
        }
    }
}

impl OutputFormatter for PytestFormatter {
    fn line(&mut self, line: &str) -> Result<(), crate::FztError> {
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        let plain_line = String::from_utf8(plain_bytes)
            .map_err(FztError::from)?
            .trim()
            .to_string();
        // TODO: HAndle collected
        if !plain_line.contains(HEADER_END) && self.in_header {
            return Ok(());
        }

        if plain_line.contains(HEADER_END) && self.in_header {
            self.in_header = false;
            self.in_body = true;
            return Ok(());
        }

        if plain_line.trim().starts_with(NEW_SECTION) {
            self.in_body = false;
            if plain_line.contains("tests coverage") {
                self.in_coverage = true;
            } else {
                self.in_coverage = false;
                self.in_footer = true
            }
            return Ok(());
        }

        if self.in_coverage {
            if let Some(possible_file) = plain_line.trim().split(" ").next() {
                if possible_file.ends_with(".py") && !possible_file.contains("::") {
                    self.coverage.insert(possible_file.trim().to_string());
                }
            }
            return Ok(());
        }

        if self.in_body {
            self.print_output.push_str(line);
            self.print_output.push_str("\n");
            return Ok(());
        }

        if self.in_footer && line.starts_with("FAILED ") || line.starts_with("ERROR ") {
            let test_part = if line.starts_with("FAILED ") {
                line["FAILED ".len()..].to_string()
            } else {
                line["ERROR ".len()..].to_string()
            };
            let test_name = test_part.trim().split("-").collect::<Vec<&str>>()[0]
                .split("[")
                .collect::<Vec<&str>>()[0]
                .trim()
                .to_string();
            self.failed_tests.insert(FailedTest {
                name: test_name,
                error_msg: String::new(),
            });
            return Ok(());
        }

        if self.in_footer && line.starts_with("SKIPPED ") {
            self.print_output.push_str(line);
            self.print_output.push_str("\n");
            return Ok(());
        }

        if self.in_footer && line.starts_with("Results") {
            let parsed = line
                .replace("Results", "")
                .replace("(", "")
                .replace(")", "")
                .replace(":", "")
                .trim()
                .to_string();
            let end_idx = parsed.len() - 1;
            let value_str = &parsed[..end_idx];
            let unit_str = parsed[end_idx..].trim();
            if let Ok(value) = value_str.parse::<f64>() {
                self.seconds = match unit_str {
                    "s" => value,
                    "ms" => value / 1000.0,
                    "µs" => value / 1_000_000.0,
                    _ => {
                        println!("Unknown time unit: {}", unit_str);
                        value
                    }
                };
            }
            return Ok(());
        }

        if self.in_footer && line.ends_with("passed") {
            line.trim()
                .split(" ")
                .next()
                .and_then(|num_str| num_str.parse::<usize>().ok())
                .map(|num| self.passed += num);
            return Ok(());
        }

        if self.in_footer && line.ends_with("error") {
            line.trim()
                .split(" ")
                .next()
                .and_then(|num_str| num_str.parse::<usize>().ok())
                .map(|num| self.error += num);
            return Ok(());
        }

        if self.in_footer && line.ends_with("skipped") {
            line.trim()
                .split(" ")
                .next()
                .and_then(|num_str| num_str.parse::<usize>().ok())
                .map(|num| self.skipped += num);
            return Ok(());
        }
        Ok(())
    }

    fn err_line(&mut self, _line: &str) -> Result<(), crate::FztError> {
        Ok(())
    }

    fn add(&mut self, other: Self) {
        self.failed_tests.extend(other.failed_tests);
        self.skipped_test.extend(other.skipped_test);
        self.passed += other.passed;
        self.error += other.error;
        self.skipped += other.skipped;
        self.seconds += other.seconds;
    }

    fn finish(self) {
        println!("Results ({}s):", self.seconds);
        if self.passed > 0 {
            println!("       \x1b[32m{}\x1b[0m passed", self.passed);
        }
        if self.error > 0 {
            println!("       \x1b[93m{}\x1b[0m error", self.error);
        }
        if self.skipped > 0 {
            println!("       {} skipped", self.skipped);
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

    fn print(&self) {
        // TODO: Just print test name and remove percentage 
        println!("{}", self.print_output);
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::process::{FailedTest, OutputFormatter};

    use super::PytestFormatter;

    #[test]
    fn parse_no_coverage() {
        let output = r"
Test session starts (platform: darwin, Python 3.12.8, pytest 8.4.1, pytest-sugar 1.1.1)
cachedir: .cache/pytest
rootdir: foo
configfile: pyproject.toml
plugins: xdist-3.8.0, anyio-4.10.0, cov-6.2.1, mock-3.14.1, sugar-1.1.1
collected 11 items

 tests/foo/test.py s                                                                                                                                                                                                                                                         9% ▉

――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――― ERROR at setup of test_export_single_algo_param_from_db_json[no_version_id] ―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

blaa

tests/foo/test.py:33: AssertionError
---------------------------------------------------------------------------------------------------------------------------------------------------- Captured stdout setup ----------------------------------------------------------------------------------------------------------------------------------------------------
2025-10-12 11:05:07 [debug    ] blaa
----------------------------------------------------------------------------------------------------------------------------------------------------- Captured log setup ------------------------------------------------------------------------------------------------------------------------------------------------------
INFO     logs
                                                                                                                                                                                                                                                                                                                 18% █▊

―――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――― ERROR at setup of test_export_single_algo_param_from_db_json[with_version_id] ――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――――

tests/foo/test.py:33: AssertionError
----------------------------------------------------------------------------------------------------------------------------------------------------- Captured log setup ------------------------------------------------------------------------------------------------------------------------------------------------------
INFO logs
                                                                                                                                                                                                                                                                                                                 27% ██▊
 blaa.py ✓                                                                                                                                                                                                                                                        36% ███▋
                                                                                                                                                                                                                                                                                                                 45% ████▋
                                                                                                                                                                                                                                                                                                                 55% █████▌
 tests/foo/test.py ✓s✓                                                                                                                                                                                                                                                 100% ██████████
====================================================================================================================================================== warnings summary =======================================================================================================================================================
src/foo/models/central/coo.py:69
======================================================================================================================================================= tests coverage ========================================================================================================================================================
______________________________________________________________________________________________________________________________________ coverage: platform darwin, python 3.12.8-final-0 _______________________________________________________________________________________________________________________________________

Coverage HTML written to dir coverage/html
==================================================================================================================================================== slowest 25 durations =====================================================================================================================================================
2.41s call     tests/foo/test.py::test

(24 durations < 0.5s hidden.)
=================================================================================================================================================== short test summary info ===================================================================================================================================================
SKIPPED [1] tests/foo/test.py:70: Need --run-cli option to run
SKIPPED [1] tests/foo/test.py:48: Need --run-cli option to run
FAILED tests/foo/test.py::test_foo[no_version_id] - assert False
FAILED tests/foo/test.py::test_baa[with_version_id] - assert False

Results (5.39s):
       1 passed
       2 error
       2 skipped
";

        let mut formatter = PytestFormatter::new();
        for line in output.lines() {
            formatter.line(line).unwrap();
        }

        assert_eq!(formatter.passed, 1);
        assert_eq!(formatter.error, 2);
        assert_eq!(formatter.skipped, 2);
        assert_eq!(formatter.seconds, 5.39);

        let failed_tests = formatter.failed_tests();

        assert_eq!(failed_tests.len(), 2);
        assert!(failed_tests.contains(&FailedTest {
            name: "tests/foo/test.py::test_foo".to_string(),
            error_msg: "".to_string()
        }));
        assert!(failed_tests.contains(&FailedTest {
            name: "tests/foo/test.py::test_baa".to_string(),
            error_msg: "".to_string()
        }));
    }

    #[test]
    fn parse_with_coverage() {
        let output = r"
Test session starts (platform: darwin, Python 3.12.8, pytest 8.4.1, pytest-sugar 1.1.1)
cachedir: .cache/pytest
rootdir: foo
configfile: pyproject.toml
plugins: xdist-3.8.0, anyio-4.10.0, cov-6.2.1, mock-3.14.1, sugar-1.1.1
collected 11 items

tests/foo/test_boo.py ✓

====================================================================================================================================================== warnings summary =======================================================================================================================================================
tests/foo/test_boo.py::test

======================================================================================================================================================= tests coverage ========================================================================================================================================================
______________________________________________________________________________________________________________________________________ coverage: platform darwin, python 3.12.8-final-0 _______________________________________________________________________________________________________________________________________

Name                                                            Stmts   Miss Branch BrPart  Cover   Missing
-----------------------------------------------------------------------------------------------------------
src/app/schemas/types.py                                1005     30     20      0    95%   28-36, 917-932, 1045-1052, 1072-1076
src/app/schemas/repo.py                             41      2      0      0    95%   37, 50
src/app/sources/types.py                    16      1      0      0    94%   22
src/app/sources/repo.py                     16      1      0      0    94%   83
-----------------------------------------------------------------------------------------------------------
TOTAL                                                            42   42    42     42    42%

42 files skipped due to complete coverage.
Coverage HTML written to dir coverage/html
==================================================================================================================================================== slowest 25 durations =====================================================================================================================================================

(3 durations < 0.5s hidden.)

Results (1.20s):
       1 passed
";

        let mut formatter = PytestFormatter::new();
        for line in output.lines() {
            formatter.line(line).unwrap();
        }

        assert_eq!(formatter.passed, 1);
        assert_eq!(formatter.error, 0);
        assert_eq!(formatter.skipped, 0);
        assert_eq!(formatter.seconds, 1.20);

        assert_eq!(formatter.failed_tests().len(), 0);
        let coverage = formatter.coverage();

        assert_eq!(coverage.len(), 4);
        assert!(coverage.contains(&"src/app/schemas/types.py".to_string()));
        assert!(coverage.contains(&"src/app/schemas/repo.py".to_string()));
        assert!(coverage.contains(&"src/app/sources/types.py".to_string()));
        assert!(coverage.contains(&"src/app/sources/repo.py".to_string()));
    }
}
