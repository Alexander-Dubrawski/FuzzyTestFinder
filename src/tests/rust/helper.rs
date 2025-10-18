use std::collections::HashMap;

use crate::runtime::FailedTest;

use super::rust_test::RustTest;

pub fn parse_failed_tests(
    failed_tests: &[FailedTest],
    current_tests: &HashMap<String, Vec<RustTest>>,
) -> HashMap<String, Vec<RustTest>> {
    let mut rust_tests = vec![];

    failed_tests.iter().for_each(|failed_test| {
        let parts: Vec<&str> = failed_test.name.split("::").collect();
        if parts.len() >= 2 {
            let module_path: Vec<String> = parts[..parts.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let method_name = parts[parts.len() - 1].to_string();
            rust_tests.push(RustTest {
                module_path,
                method_name,
            });
        }
    });

    current_tests
        .iter()
        .fold(HashMap::new(), |mut acc, (file_path, tests)| {
            rust_tests.iter().for_each(|rust_test| {
                if tests.contains(rust_test) {
                    acc.entry(file_path.clone())
                        .or_insert(vec![])
                        .push(rust_test.clone());
                }
            });
            acc
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        tests::rust::{helper::parse_failed_tests, rust_test::RustTest},
        utils::process::FailedTest,
    };

    #[test]
    fn collect_failed_tests() {
        let current_tests: HashMap<String, Vec<RustTest>> = HashMap::from([(
            "tests/java/java_test/tests.java".to_string(),
            vec![
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_tests".to_string(),
                },
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_meta".to_string(),
                },
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "some_other_tests".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_meta".to_string(),
                },
            ],
        )]);

        let expected: HashMap<String, Vec<RustTest>> = HashMap::from([(
            "tests/java/java_test/tests.java".to_string(),
            vec![
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_tests".to_string(),
                },
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_meta".to_string(),
                },
            ],
        )]);

        let failed_tests = vec![
            FailedTest {
                name: "tests/java/java_test/tests.java::collect_meta".to_string(),
                error_msg: "".to_string(),
            },
            FailedTest {
                name: "tests/java/java_test/tests.java::collect_tests".to_string(),
                error_msg: "".to_string(),
            },
        ];

        let result = parse_failed_tests(failed_tests.as_slice(), &current_tests);

        assert_eq!(result.len(), expected.len());
        for (tests_path, tests) in &expected {
            let mut expected_sorted = tests.clone();
            expected_sorted.sort();
            let mut result_sorted = result.get(tests_path).unwrap().clone();
            result_sorted.sort();
            assert_eq!(expected_sorted, result_sorted)
        }
    }
}
