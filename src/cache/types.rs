use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub root_folder: String,
    pub timestamp: u128,
    pub tests: HashMap<String, HashSet<String>>,
}

impl CacheEntry {
    pub fn new(
        root_folder: String,
        timestamp: u128,
        tests: HashMap<String, HashSet<String>>,
    ) -> Self {
        Self {
            root_folder,
            timestamp,
            tests,
        }
    }
}
