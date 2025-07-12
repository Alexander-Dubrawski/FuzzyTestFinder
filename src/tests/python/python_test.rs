use crate::tests::Test;

pub struct PythonTest {
    path: String,
    test: String,
}

impl PythonTest {
    pub fn new(path: String, test: String) -> Self {
        Self { path, test }
    }
}

impl Test for PythonTest {
    fn runtime_argument(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }

    fn name(&self) -> String {
        format!("{}::{}", self.path, self.test)
    }

    fn file_path(&self) -> String {
        self.path.clone()
    }
}
