use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use regex::Regex;
use rustpython_parser::{Mode, lexer::lex, parse_tokens};

use crate::{errors::FztError, runtime::FailedTest, utils::file_walking::collect_tests};

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

// TODO: Parse to hash map
pub fn parse_failed_tests(failed_tests: &[FailedTest]) -> HashMap<String, HashSet<String>> {
    let mut sorted_failed_tests = HashMap::new();

    failed_tests.iter().for_each(|failed_test| {
        let parts: Vec<&str> = failed_test.name.split("::").collect();
        if parts.len() == 2 {
            let file_path = parts[0].trim().to_string();
            let test_name = parts[1].trim().to_string();
            sorted_failed_tests
                .entry(file_path)
                .or_insert_with(HashSet::new)
                .insert(test_name);
        }
    });
    sorted_failed_tests
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
    fn collect_failed() {
        let failed_tests: Vec<FailedTest> = vec![
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
        ];

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

        let result = parse_failed_tests(failed_tests.as_slice());

        assert_eq!(result, expected);
    }
}
