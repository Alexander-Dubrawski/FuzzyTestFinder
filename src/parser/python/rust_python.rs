use crate::parser::Parser;

use super::python_tests::PythonTests;

#[derive(Default)]
pub struct RustPytonParser {
    // absolute path
    root_dir: String,
}

impl RustPytonParser {
    pub fn new(root_dir: String) -> Self {
        Self { root_dir }
    }
}

impl Parser<PythonTests> for RustPytonParser {
    fn parse_tests(&self, tests: &mut PythonTests) -> bool {
        tests.update(false)
    }
}
