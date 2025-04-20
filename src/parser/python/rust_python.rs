use crate::parser::Parser;

use super::python_tests::PythonTests;

#[derive(Default)]
pub struct RustPytonParser {}

impl Parser<PythonTests> for RustPytonParser {
    fn parse_tests(&self, tests: &mut PythonTests) -> bool {
        tests.update(false)
    }
}
