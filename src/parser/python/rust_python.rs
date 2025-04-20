use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::FztError;

use super::python_tests::PythonTests;

#[derive(Default)]
pub struct RustPytonParser {}

impl RustPytonParser {
    pub fn parse_tests(&self, tests: &mut PythonTests) -> Result<bool, FztError> {
        let updated = tests.update(false)?;
        if updated {
            let new_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
            tests.timestamp = new_timestamp;
        }
        Ok(updated)
    }
}
