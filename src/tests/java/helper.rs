use super::java_test::JavaTest;
use std::collections::HashMap;

pub fn parse_failed_tests(
    output: &str,
    current_tests: &HashMap<String, Vec<JavaTest>>,
) -> HashMap<String, Vec<JavaTest>> {
    let mut java_tests = vec![];

    output.lines().fold(false, |in_test, line| {
        if line.ends_with("FAILED") {
            return true;
        }
        if line.is_empty() && in_test {
            return false;
        }
        if in_test {
            if line.trim().starts_with("at ") {
                let parts: Vec<&str> = line.trim().split('(').collect();
                if parts.len() == 2 {
                    let method_part = parts[0]
                        .trim_start_matches("at ")
                        .trim_start_matches("app//")
                        .trim();
                    if let Some((class_path, method_name)) = method_part.rsplit_once('.') {
                        // We also push methods that are actually not part of the actual test,
                        // but filtering them out is done later.
                        java_tests.push(JavaTest {
                            class_path: class_path.to_string(),
                            method_name: method_name.to_string(),
                        });
                    }
                }
            }
        }
        in_test
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

    use crate::tests::java::{helper::parse_failed_tests, java_test::JavaTest};

    #[test]
    fn collect_failed_tests() {
        let output = "
ParserTest > boo() FAILED
    org.opentest4j.AssertionFailedError: expected: <true> but was: <false>
        at app//org.junit.jupiter.api.AssertionFailureBuilder.build(AssertionFailureBuilder.java:151)
        at app//org.junit.jupiter.api.AssertionFailureBuilder.buildAndThrow(AssertionFailureBuilder.java:132)
        at app//org.junit.jupiter.api.AssertTrue.failNotTrue(AssertTrue.java:63)
        at app//org.junit.jupiter.api.AssertTrue.assertTrue(AssertTrue.java:36)
        at app//org.junit.jupiter.api.AssertTrue.assertTrue(AssertTrue.java:31)
        at app//org.junit.jupiter.api.Assertions.assertTrue(Assertions.java:183)
        at app//org.parser.ParserTest.boo(ParserTest.java:113)

ParserTest > foo() FAILED
    org.opentest4j.AssertionFailedError: expected: <true> but was: <false>
        at app//org.junit.jupiter.api.AssertionFailureBuilder.build(AssertionFailureBuilder.java:151)
        at app//org.junit.jupiter.api.AssertionFailureBuilder.buildAndThrow(AssertionFailureBuilder.java:132)
        at app//org.junit.jupiter.api.AssertTrue.failNotTrue(AssertTrue.java:63)
        at app//org.junit.jupiter.api.AssertTrue.assertTrue(AssertTrue.java:36)
        at app//org.junit.jupiter.api.AssertTrue.assertTrue(AssertTrue.java:31)
        at app//org.junit.jupiter.api.Assertions.assertTrue(Assertions.java:183)
        at app//org.parser.ParserTest.foo(ParserTest.java:79)

ParserTest > parseNew() STANDARD_ERROR
    Sep 06, 2025 9:27:07 AM org.parser.JavaTests$1 visitFile
    INFO: Tests updated: java/a/testOne.java : [org.parser.JavaTest@55d9b8f0]
    Sep 06, 2025 9:27:07 AM org.parser.JavaTests$1 visitFile
    INFO: Tests updated: java/a/testTwo.java : [org.parser.JavaTest@1b5c3e5f, org.parser.JavaTest@13741d5a]
    Sep 06, 2025 9:27:07 AM org.parser.JavaTests$1 visitFile
    INFO: Tests updated: java/b/testThree.java : [org.parser.JavaTest@48840594]

ParserTest > parseCache() FAILED
    java.lang.ArithmeticException: / by zero
        at org.parser.Parser.parse(Parser.java:20)
        at org.parser.SomeParserTest.parseCache(ParserTest.java:53)
        ";

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

        let expected: HashMap<String, Vec<JavaTest>> = HashMap::from([
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

        let result = parse_failed_tests(output, &current_tests);

        assert_eq!(result.len(), expected.len());
        for (tests_path, tests) in &expected {
            assert_eq!(
                tests.clone().sort(),
                result.get(tests_path).unwrap().clone().sort()
            );
        }
    }
}
