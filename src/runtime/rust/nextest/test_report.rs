use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub event: String,
    pub name: String,
    pub stdout: Option<String>,
}
