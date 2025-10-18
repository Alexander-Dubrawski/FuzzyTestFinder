use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageReport {
    pub files: HashMap<String, FileCoverage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCoverage {
    pub summary: CoverageSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub percent_covered: f64,
}
