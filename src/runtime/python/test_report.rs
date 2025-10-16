use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub created: f64,
    pub duration: f64,
    pub exitcode: i32,
    pub root: String,
    pub environment: HashMap<String, String>,
    pub summary: TestSummary,
    pub collectors: Vec<TestCollector>,
    pub tests: Vec<Test>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    pub failed: Option<usize>,
    pub passed: Option<usize>,
    pub skipped: Option<usize>,
    pub total: usize,
    pub collected: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCollector {
    pub nodeid: String,
    pub outcome: String,
    pub result: Vec<CollectorResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectorResult {
    pub nodeid: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub lineno: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Test {
    pub nodeid: String,
    pub lineno: usize,
    pub outcome: String,
    pub keywords: Vec<String>,
    pub setup: TestPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<TestCall>,
    pub teardown: TestPhase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestPhase {
    pub duration: f64,
    pub outcome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCall {
    pub duration: f64,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash: Option<CrashInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceback: Option<Vec<TracebackEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longrepr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrashInfo {
    pub path: String,
    pub lineno: usize,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracebackEntry {
    pub path: String,
    pub lineno: usize,
    pub message: String,
}
