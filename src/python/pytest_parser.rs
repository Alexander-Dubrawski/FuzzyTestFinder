use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::str;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use itertools::Itertools;
use rustpython_parser::Mode;
use rustpython_parser::lexer::lex;
use rustpython_parser::parse_tokens;
use walkdir::WalkDir;

use crate::cache::types::CacheEntry;
use crate::parser::Parser;

use super::types::PyTests;

#[derive(Default)]
pub struct PyTestParser {
    // absolute path
    root_dir: String,
    // absolute path
    test_dir: String,
}

impl PyTestParser {
    pub fn new(root_dir: String, test_dir: String) -> Self {
        Self { root_dir, test_dir }
    }

    fn get_pytest(_root: &str) -> PyTests {
        // TODO execute command on path
        let binding = Command::new("python")
            .arg("-m")
            .arg("pytest")
            .arg("--co")
            .arg("-q")
            .output()
            .expect("failed to retrieve python tests");
        let output = str::from_utf8(binding.stdout.as_slice()).unwrap();
        Self::parse_python_tests(output)
    }

    fn parse_python_tests(pytest_out: &str) -> PyTests {
        let mut py_tests: HashMap<String, HashSet<String>> = HashMap::new();
        for line in pytest_out.lines() {
            if line.is_empty() {
                break;
            }
            let (path, test_name) = line
                .split("::")
                .collect_tuple()
                .map(|(path, test)| {
                    let test_name = test.chars().take_while(|&ch| ch != '[').collect::<String>();
                    (path.to_string(), test_name)
                })
                .unwrap();
            let entry = py_tests.get_mut(&path);
            match entry {
                Some(tests) => {
                    tests.insert(test_name);
                }
                None => {
                    let mut new_tests = HashSet::new();
                    new_tests.insert(test_name);
                    py_tests.insert(path, new_tests);
                }
            }
        }
        PyTests::new(py_tests)
    }

    fn get_cache_entry(python_tests: PyTests, root: &str, test_folder: &str) -> CacheEntry {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        CacheEntry::new(
            root.to_string(),
            test_folder.to_string(),
            timestamp,
            python_tests.tests,
        )
    }

    fn collect_tests_from_file(path: &Path) -> HashSet<String> {
        let source_code = std::fs::read_to_string(path).unwrap();
        let tokens = lex(source_code.as_str(), Mode::Module);
        let ast = parse_tokens(tokens, Mode::Module, "<embedded>").unwrap();
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
                        _ => continue,
                    }
                }
            }
            _ => todo!(),
        }
        tests
    }

    fn check_file_for_new_tests(path: &Path, tests: &HashSet<String>) -> bool {
        let collected_tests = Self::collect_tests_from_file(path);
        &collected_tests != tests
    }

    fn needs_update(cache_entry: &mut crate::cache::types::CacheEntry) -> bool {
        for entry in WalkDir::new(cache_entry.test_folder.as_str()) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let metadata = std::fs::metadata(entry.path()).unwrap();
                if entry.path().extension().is_none() {
                    continue;
                }

                if entry.path().extension().and_then(OsStr::to_str).unwrap() != "py" {
                    continue;
                }

                // TODO: Check if modified, that if modified, that there are new/removed functions
                // If so Cache entry can be replaced in place
                if let Ok(modified) = metadata.modified() {
                    if modified.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        > cache_entry.timestamp
                    {
                        println!("Modified: {:?}", entry.path());
                        let full_path = entry.path().as_os_str().to_str().unwrap();
                        let start_dir = Path::new(cache_entry.test_folder.as_str())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();
                        let relative_path = format!(
                            "{}{}",
                            start_dir,
                            full_path
                                .strip_prefix(cache_entry.test_folder.as_str())
                                .unwrap()
                        );

                        println!("{}", relative_path);
                        if Self::check_file_for_new_tests(
                            entry.path(),
                            &cache_entry.tests[&relative_path],
                        ) {
                            println!("New tests found");
                            return true;
                        }
                    }
                }
                // TODO: Check if created, that the file includes test function
                if let Ok(created) = metadata.created() {
                    if created.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        > cache_entry.timestamp
                    {
                        println!("New file: {:?}", entry.path());
                        if !Self::collect_tests_from_file(entry.path()).is_empty() {
                            println!("New tests found");
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl Parser for PyTestParser {
    fn parse_test(&self) -> CacheEntry {
        Self::get_cache_entry(
            Self::get_pytest(&self.root_dir),
            &self.root_dir,
            &self.test_dir,
        )
    }

    fn update_tests(&self, cache_entry: &mut CacheEntry) -> bool {
        if Self::needs_update(cache_entry) {
            *cache_entry = self.parse_test();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn pytest_parsing() {
        let python_source = r#"tests/foo::test_a
tests/foo::test_b[None, None]
tests/foo/boo::test_c

------------------------------ coverage ------------------------------
Coverage HTML written to dir coverage/html
    "#;
        let mut expected: HashMap<String, HashSet<String>> = HashMap::new();
        expected.insert(
            "tests/foo".to_string(),
            HashSet::from_iter(
                vec!["test_a".to_string(), "test_b".to_string()]
                    .iter()
                    .cloned(),
            ),
        );
        expected.insert(
            "tests/foo/boo".to_string(),
            HashSet::from_iter(vec!["test_c".to_string()].iter().cloned()),
        );

        let result = PyTestParser::parse_python_tests(python_source);
        assert_eq!(result, PyTests::new(expected));
    }
}
