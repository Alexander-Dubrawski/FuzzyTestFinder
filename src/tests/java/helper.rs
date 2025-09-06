use std::collections::HashMap;

use super::java_test::JavaTest;

pub fn parse_failed_tests(
    output: &str,
    current_tests: &HashMap<String, Vec<JavaTest>>,
) -> HashMap<String, Vec<JavaTest>> {
    todo!()
}