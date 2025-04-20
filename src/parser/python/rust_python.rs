use std::time::{SystemTime, UNIX_EPOCH};

use crate::parser::Parser;

use super::python_tests::PythonTests;

#[derive(Default)]
pub struct RustPytonParser {}

impl Parser<PythonTests> for RustPytonParser {
    fn parse_tests(&self, tests: &mut PythonTests) -> bool {
        let updated = tests.update(false);
        if updated {
            let new_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            tests.timestamp = new_timestamp;
        }
        updated
    }
}
