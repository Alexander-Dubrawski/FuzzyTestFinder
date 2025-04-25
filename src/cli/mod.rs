use serde::{Deserialize, Serialize};

pub mod cli_parser;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Language {
    Python((PythonParser, PythonRuntime)),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PythonParser {
    Pytest,
    RustPython,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PythonRuntime {
    Pytest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SearchEngine {
    FzF,
}

pub struct Config {
    pub language: Option<Language>,
    pub search_engine: Option<SearchEngine>,
    pub clear_cache: bool,
    pub history: bool,
    pub last: bool,
    pub default: bool,
}
