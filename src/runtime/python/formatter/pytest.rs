use std::collections::HashSet;

use crate::{
    FztError,
    utils::process::{FailedTest, OutputFormatter},
};

#[derive(Clone, Debug, Default)]
pub struct PytestFormatter {
    failed_tests: HashSet<FailedTest>,
}

impl PytestFormatter {
    pub fn new() -> Self {
        Self {
            failed_tests: HashSet::new(),
        }
    }
}

impl OutputFormatter for PytestFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        let plain_line = String::from_utf8(plain_bytes).map_err(FztError::from)?;
        if plain_line.starts_with("FAILED ") || plain_line.starts_with("ERROR ") {
            let parts: Vec<&str> = if plain_line.starts_with("FAILED ") {
                plain_line["FAILED ".len()..].split("::").collect()
            } else {
                plain_line["ERROR ".len()..].split("::").collect()
            };
            if parts.len() == 2 {
                let file_path = parts[0].trim().to_string();
                let test_paths = parts[1].splitn(2, "-").collect::<Vec<&str>>();
                let test_name = test_paths[0].split("[").collect::<Vec<&str>>()[0]
                    .trim()
                    .to_string();
                let error_msg = if test_paths.len() == 2 {
                    test_paths[1].split("[").collect::<Vec<&str>>()[0]
                        .trim()
                        .to_string()
                } else {
                    String::new()
                };
                self.failed_tests.insert(FailedTest {
                    name: format!("{}::{}", file_path, test_name),
                    error_msg,
                });
            }
        }
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

    fn failed_tests(&self) -> Vec<crate::utils::process::FailedTest> {
        vec![]
    }

    fn update(&mut self) -> Result<(), crate::FztError> {
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
        let output = "
-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
==================================================================== tests coverage ====================================================================
___________________________________________________ coverage: platform darwin, python 3.12.8-final-0 ___________________________________________________

Coverage HTML written to dir coverage/html
=============================================================== short test summary info ================================================================
FAILED tests/folder_a/folder_b/test_foo.py::test_foo_two[ABCD] - assert False
FAILED tests/folder_a/folder_b/test_foo.py::test_foo_three - assert False
FAILED tests/folder_a/folder_c/test_baa.py::test_foo_two - assert False

Results (1.34s):
       2 passed
       2 failed
         - tests/folder_a/folder_b/test_foo.py:46 test_foo_two
         - tests/folder_a/folder_b/test_foo.py:49 test_foo_three
         - tests/folder_a/folder_c/test_baa.py:90 test_foo_two
";
        let mut formatter = PytestFormatter::new();
        let expected: HashSet<FailedTest> = HashSet::from([
            FailedTest {
                name: "tests/folder_a/folder_b/test_foo.py::test_foo_two".to_string(),
                error_msg: String::from("assert False"),
            },
            FailedTest {
                name: "tests/folder_a/folder_b/test_foo.py::test_foo_three".to_string(),
                error_msg: String::from("assert False"),
            },
            FailedTest {
                name: "tests/folder_a/folder_c/test_baa.py::test_foo_two".to_string(),
                error_msg: String::from("assert False"),
            },
        ]);

        for line in output.lines() {
            formatter.line(line).unwrap();
        }
        assert_eq!(formatter.failed_tests, expected);
    }

    #[test]
    fn collect_failed_pytest() {
        let output = "
    def test_food(mocked, tmp_path, mock_datetime_now):
>       assert False
E       assert False

tests/folder_a/folder_b/test_foo.py:310: AssertionError
====================================================================================================================================================== warnings summary =======================================================================================================================================================
tests/folder_a/folder_b/test_foo.py::test_foo_two[ABC]

-- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
=================================================================================================================================================== short test summary info ===================================================================================================================================================
FAILED tests/folder_a/folder_b/test_foo.py::test_foo_two[ABC] - assert False
FAILED tests/folder_a/folder_b/test_foo.py::test_foo_three - assert False
FAILED tests/folder_a/folder_c/test_baa.py::test_foo_two - assert False
";

        let mut formatter = PytestFormatter::new();
        let expected: HashSet<FailedTest> = HashSet::from([
            FailedTest {
                name: "tests/folder_a/folder_b/test_foo.py::test_foo_two".to_string(),
                error_msg: String::from("assert False"),
            },
            FailedTest {
                name: "tests/folder_a/folder_b/test_foo.py::test_foo_three".to_string(),
                error_msg: String::from("assert False"),
            },
            FailedTest {
                name: "tests/folder_a/folder_c/test_baa.py::test_foo_two".to_string(),
                error_msg: String::from("assert False"),
            },
        ]);

        for line in output.lines() {
            formatter.line(line).unwrap();
        }
        assert_eq!(formatter.failed_tests, expected);
    }

    #[test]
    fn collect_error_pytest() {
        let output = "
ERROR tests/folder_a/folder_b/test_foo.py::test_foo_two[ABC] - polars.exceptions.ColumnNotFoundError: key                                                                                                                                                                                                                                      [ 20%]
ERROR tests/folder_a/folder_b/test_foo.py::test_foo_three - polars.exceptions.ColumnNotFoundError: key                                                                                                                                                                                                                        [ 40%]
ERROR tests/folder_a/folder_c/test_baa.py::test_foo_two - polars.exceptions.ColumnNotFoundError: key
        ";

        let mut formatter = PytestFormatter::new();
        let expected: HashSet<FailedTest> = HashSet::from([
            FailedTest {
                name: "tests/folder_a/folder_b/test_foo.py::test_foo_two".to_string(),
                error_msg: String::from("polars.exceptions.ColumnNotFoundError: key"),
            },
            FailedTest {
                name: "tests/folder_a/folder_b/test_foo.py::test_foo_three".to_string(),
                error_msg: String::from("polars.exceptions.ColumnNotFoundError: key"),
            },
            FailedTest {
                name: "tests/folder_a/folder_c/test_baa.py::test_foo_two".to_string(),
                error_msg: String::from("polars.exceptions.ColumnNotFoundError: key"),
            },
        ]);

        for line in output.lines() {
            formatter.line(line).unwrap();
        }

        assert_eq!(formatter.failed_tests, expected);
    }
}
