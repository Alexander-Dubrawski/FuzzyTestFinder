use crate::runtime::FailedTest;

use super::java_test::JavaTest;
use std::collections::HashMap;

pub fn parse_failed_tests(
    failed_test_output: &[FailedTest],
    current_tests: &HashMap<String, Vec<JavaTest>>,
) -> HashMap<String, Vec<JavaTest>> {
    let mut java_tests = vec![];

    failed_test_output.iter().for_each(|failed_test| {
        if let Some((class_path, method_name)) = failed_test.name.rsplit_once('.') {
            // We also push methods that are actually not part of the actual test,
            // but filtering them out is done later.
            java_tests.push(JavaTest {
                class_path: class_path.to_string(),
                method_name: method_name.to_string(),
            })
        }
    });

    current_tests
        .iter()
        .fold(HashMap::new(), |mut acc, (file_path, tests)| {
            java_tests.iter().for_each(|java_test| {
                if tests.contains(java_test) {
                    acc.entry(file_path.clone())
                        .or_insert(vec![])
                        .push(java_test.clone());
                }
            });
            acc
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        runtime::FailedTest,
        tests::java::{helper::parse_failed_tests, java_test::JavaTest},
    };

    #[test]
    fn collect_failed_tests() {
        let failed_tests = vec![
            FailedTest {
                name: "org.parser.ParserTest.boo".to_string(),
                error_msg: String::from(""),
            },
            FailedTest {
                name: "org.parser.ParserTest.foo".to_string(),
                error_msg: String::from(""),
            },
            FailedTest {
                name: "org.parser.ParserTest.hoo".to_string(),
                error_msg: String::from(""),
            },
        ];

        let current_tests: HashMap<String, Vec<JavaTest>> = HashMap::from([
            (
                String::from("parser/ParserTest.java"),
                vec![
                    JavaTest {
                        class_path: String::from("org.parser.ParserTest"),
                        method_name: String::from("boo"),
                    },
                    JavaTest {
                        class_path: String::from("org.parser.ParserTest"),
                        method_name: String::from("foo"),
                    },
                    JavaTest {
                        class_path: String::from("org.parser.ParserTest"),
                        method_name: String::from("hoo"),
                    },
                ],
            ),
            (
                String::from("parser/SomeParserTest.java"),
                vec![JavaTest {
                    class_path: String::from("org.parser.SomeParserTest"),
                    method_name: String::from("parseCache"),
                }],
            ),
        ]);

        let expected: HashMap<String, Vec<JavaTest>> = HashMap::from([(
            String::from("parser/ParserTest.java"),
            vec![
                JavaTest {
                    class_path: String::from("org.parser.ParserTest"),
                    method_name: String::from("boo"),
                },
                JavaTest {
                    class_path: String::from("org.parser.ParserTest"),
                    method_name: String::from("foo"),
                },
                JavaTest {
                    class_path: String::from("org.parser.ParserTest"),
                    method_name: String::from("hoo"),
                },
            ],
        )]);

        let result = parse_failed_tests(failed_tests.as_slice(), &current_tests);

        assert_eq!(result.len(), expected.len());

        for (tests_path, tests) in &expected {
            let mut expected_sorted = tests.clone();
            expected_sorted.sort();
            let mut result_sorted = result.get(tests_path).unwrap().clone();
            result_sorted.sort();
            assert_eq!(expected_sorted, result_sorted);
        }
    }
}
