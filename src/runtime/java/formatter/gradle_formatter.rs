use std::collections::HashSet;

use crate::{
    FztError,
    runtime::{FailedTest, OutputFormatter},
};

#[derive(Clone, Debug, Default)]
pub struct GradleFormatter {
    failed_tests: HashSet<FailedTest>,
    output_lines: Vec<String>,
}

impl GradleFormatter {
    pub fn new() -> Self {
        Self {
            failed_tests: HashSet::new(),
            output_lines: vec![],
        }
    }
}

impl OutputFormatter for GradleFormatter {
    fn line(&mut self, line: &str) -> Result<(), FztError> {
        println!("{line}");
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        self.output_lines
            .push(String::from_utf8(plain_bytes).map_err(FztError::from)?);
        Ok(())
    }

    fn err_line(&mut self, line: &str) -> Result<(), crate::FztError> {
        println!("{line}");
        let plain_bytes = strip_ansi_escapes::strip(line.as_bytes());
        self.output_lines
            .push(String::from_utf8(plain_bytes).map_err(FztError::from)?);
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

    fn failed_tests(&self) -> Vec<FailedTest> {
        self.failed_tests.iter().cloned().collect()
    }

    fn update(&mut self) -> Result<(), FztError> {
        let mut last_failed_block_line = String::new();
        self.output_lines.iter().fold(false, |in_test, line| {
            if line.ends_with("FAILED") {
                return true;
            }
            if line.is_empty() && in_test {
                let parts: Vec<&str> = last_failed_block_line.split('(').collect();
                if parts.len() == 2 {
                    let method_part = parts[0]
                        .trim_start_matches("at ")
                        .trim_start_matches("app//")
                        .trim();
                    self.failed_tests.insert(FailedTest {
                        name: method_part.to_string(),
                        error_msg: String::new(),
                    });
                    return false;
                }
            }
            if in_test {
                if line.trim().starts_with("at ") {
                    last_failed_block_line = line.trim().to_string();
                }
            }
            in_test
        });
        Ok(())
    }

    fn print(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

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

        let mut formatter = GradleFormatter::new();
        let expected: HashSet<FailedTest> = HashSet::from([
            FailedTest {
                name: "org.parser.ParserTest.boo".to_string(),
                error_msg: String::from(""),
            },
            FailedTest {
                name: "org.parser.ParserTest.foo".to_string(),
                error_msg: String::from(""),
            },
        ]);

        for line in output.lines() {
            formatter.line(line).unwrap();
        }
        formatter.update().unwrap();
        assert_eq!(formatter.failed_tests, expected);
    }
}
