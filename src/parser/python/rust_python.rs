use crate::errors::FztError;

use super::python_tests::PythonTests;

#[derive(Default)]
pub struct RustPytonParser {}

impl RustPytonParser {
    pub fn parse_tests(&self, tests: &mut PythonTests) -> Result<bool, FztError> {
        let updated = tests.update(false)?;
        Ok(updated)
    }
}
