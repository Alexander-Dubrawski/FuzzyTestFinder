use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use regex::Regex;
use rustpython_parser::{Mode, lexer::lex, parse_tokens};

use crate::{errors::FztError, utils::file_walking::collect_tests};

fn collect_tests_from_file(path: &Path) -> Result<HashSet<String>, FztError> {
    let source_code = std::fs::read_to_string(path)?;
    let tokens = lex(source_code.as_str(), Mode::Module);
    let ast = parse_tokens(tokens, Mode::Module, "<embedded>")?;
    let mut tests = HashSet::new();
    match ast {
        rustpython_parser::ast::Mod::Module(mod_module) => {
            for stmt in mod_module.body.iter() {
                match stmt {
                    rustpython_parser::ast::Stmt::FunctionDef(stmt_function_def) => {
                        let test_name = stmt_function_def.name.to_string();
                        if test_name.starts_with("test") {
                            tests.insert(stmt_function_def.name.to_string());
                        }
                    }
                    rustpython_parser::ast::Stmt::AsyncFunctionDef(stmt_async_function_def) => {
                        let test_name = stmt_async_function_def.name.to_string();
                        if test_name.starts_with("test") {
                            tests.insert(stmt_async_function_def.name.to_string());
                        }
                    }
                    _ => continue,
                }
            }
        }
        rustpython_parser::ast::Mod::Interactive(_) => {
            return Err(FztError::PythonParser(
                "Mod::Interactive not supported".to_string(),
            ));
        }
        rustpython_parser::ast::Mod::Expression(_) => {
            return Err(FztError::PythonParser(
                "Mod::Expression not supported".to_string(),
            ));
        }
        rustpython_parser::ast::Mod::FunctionType(_) => {
            return Err(FztError::PythonParser(
                "Mod::FunctionType not supported".to_string(),
            ));
        }
    }
    Ok(tests)
}

pub fn update_tests(
    root_folder: &str,
    timestamp: &mut u128,
    tests: &mut HashMap<String, HashSet<String>>,
    only_check_for_change: bool,
) -> Result<bool, FztError> {
    collect_tests(
        root_folder,
        timestamp,
        tests,
        only_check_for_change,
        "py",
        Some(Regex::new(r"^(test_.*\.py|.*_test\.py)$")?),
        collect_tests_from_file,
    )
}

pub fn parse_failed_tests(output: &str) -> HashMap<String, HashSet<String>> {
    let mut failed_tests = HashMap::new();
    output.lines().for_each(|line| {
        if line.starts_with("FAILED ") || line.starts_with("ERROR ") {
            let parts: Vec<&str> = if line.starts_with("FAILED ") {
                line["FAILED ".len()..].split("::").collect()
            } else {
                line["ERROR ".len()..].split("::").collect()
            };
            if parts.len() == 2 {
                let file_path = parts[0].trim().to_string();
                let test_name = parts[1].split("-").collect::<Vec<&str>>()[0]
                    .split("[")
                    .collect::<Vec<&str>>()[0]
                    .trim()
                    .to_string();
                failed_tests
                    .entry(file_path)
                    .or_insert_with(HashSet::new)
                    .insert(test_name);
            }
        }
    });
    failed_tests
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::utils::{file_walking::filter_out_deleted_files, test_utils::copy_dict};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn collect_tests() {
        let mut path = std::env::current_dir().unwrap();
        path.push("src/tests/python/test_data");
        let (_temp_dir, dir_path) = copy_dict(path.as_path()).unwrap();
        let test_path = dir_path.as_path().to_str().unwrap();
        let mut tests = HashMap::new();
        let mut expected_tests: HashMap<String, HashSet<String>> = HashMap::new();
        expected_tests.insert(
            "berlin/berlin_test.py".to_string(),
            HashSet::from_iter(vec!["test_berlin"].into_iter().map(|v| v.to_string())),
        );
        expected_tests.insert(
            "berlin/hamburg/test_hamburg.py".to_string(),
            HashSet::from_iter(
                vec!["test_hamburg", "test_hamburg_harburg"]
                    .into_iter()
                    .map(|v| v.to_string()),
            ),
        );
        expected_tests.insert(
            "berlin/potsdam/potsdam_test.py".to_string(),
            HashSet::from_iter(vec!["test_potsdam"].into_iter().map(|v| v.to_string())),
        );
        assert!(update_tests(test_path, &mut 0, &mut tests, false).unwrap());
        assert_eq!(tests, expected_tests);

        let mut time_stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        assert!(!update_tests(test_path, &mut time_stamp, &mut tests, false).unwrap());

        // Remove test
        std::fs::remove_file(format!("{test_path}/berlin/potsdam/potsdam_test.py")).unwrap();
        expected_tests
            .remove(&"berlin/potsdam/potsdam_test.py".to_string())
            .unwrap();
        assert!(filter_out_deleted_files(test_path, &mut tests));

        time_stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        assert!(!update_tests(test_path, &mut time_stamp, &mut tests, false).unwrap());
        assert_eq!(tests, expected_tests);

        // Change test
        std::fs::write(
            &Path::new(test_path).join("berlin/berlin_test.py"),
            "def test_berlin_new():\n\tfoo=42",
        )
        .unwrap();
        expected_tests.insert(
            "berlin/berlin_test.py".to_string(),
            HashSet::from_iter(vec!["test_berlin_new"].into_iter().map(|v| v.to_string())),
        );
        assert!(update_tests(test_path, &mut time_stamp, &mut tests, false).unwrap());
        assert_eq!(tests, expected_tests);
    }

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
        let expected: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "tests/folder_a/folder_b/test_foo.py".to_string(),
                HashSet::from_iter(vec![
                    "test_foo_two".to_string(),
                    "test_foo_three".to_string(),
                ]),
            ),
            (
                "tests/folder_a/folder_c/test_baa.py".to_string(),
                HashSet::from_iter(vec!["test_foo_two".to_string()]),
            ),
        ]);

        let result = parse_failed_tests(output);

        assert_eq!(result, expected);
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
        let expected: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "tests/folder_a/folder_b/test_foo.py".to_string(),
                HashSet::from_iter(vec![
                    "test_foo_two".to_string(),
                    "test_foo_three".to_string(),
                ]),
            ),
            (
                "tests/folder_a/folder_c/test_baa.py".to_string(),
                HashSet::from_iter(vec!["test_foo_two".to_string()]),
            ),
        ]);

        let result = parse_failed_tests(output);

        assert_eq!(result, expected);
    }

    #[test]
    fn collect_error_pytest() {
        let output = "
ERROR tests/folder_a/folder_b/test_foo.py::test_foo_two[ABC] - polars.exceptions.ColumnNotFoundError: key                                                                                                                                                                                                                                      [ 20%]
ERROR tests/folder_a/folder_b/test_foo.py::test_foo_three - polars.exceptions.ColumnNotFoundError: key                                                                                                                                                                                                                        [ 40%]
ERROR tests/folder_a/folder_c/test_baa.py::test_foo_two - polars.exceptions.ColumnNotFoundError: key
        ";
        let expected: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "tests/folder_a/folder_b/test_foo.py".to_string(),
                HashSet::from_iter(vec![
                    "test_foo_two".to_string(),
                    "test_foo_three".to_string(),
                ]),
            ),
            (
                "tests/folder_a/folder_c/test_baa.py".to_string(),
                HashSet::from_iter(vec!["test_foo_two".to_string()]),
            ),
        ]);

        let result = parse_failed_tests(output);

        assert_eq!(result, expected);
    }
}
