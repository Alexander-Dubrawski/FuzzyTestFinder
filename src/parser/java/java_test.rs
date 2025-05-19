use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaTests {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, Vec<JavaTest>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaTest {
    pub class_path: String,
    pub method_name: String,
}