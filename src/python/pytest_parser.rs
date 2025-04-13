use std::collections::HashMap;
use std::collections::HashSet;
use std::str;

use itertools::Itertools;

use super::types::PyTests;

pub fn parse_python_tests(pytest_out: &str) -> PyTests {
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
        let mut expected = HashMap::new();
        expected.insert(
            "tests/foo".to_string(),
            vec!["test_a".to_string(), "test_b".to_string()],
        );
        expected.insert("tests/foo/boo".to_string(), vec!["test_c".to_string()]);

        let result = parse_python_tests(python_source);
        assert_eq!(result, PyTests::new(expected));
    }
}
