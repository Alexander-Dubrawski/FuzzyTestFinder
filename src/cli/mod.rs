use serde::{Deserialize, Serialize};

pub mod cli_parser;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Language {
    Python((PythonParser, PythonRuntime)),
    Java((JavaTestFramwork, JavaRuntime))
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
pub enum JavaRuntime {
    Gradle,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JavaTestFramwork {
    JUnit5,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SearchEngine {
    FzF,
}

#[derive(Clone)]
pub struct Config {
    pub language: Option<Language>,
    pub search_engine: Option<SearchEngine>,
    pub clear_cache: bool,
    pub history: bool,
    pub last: bool,
    pub default: bool,
    pub verbose: bool,
    pub clear_history: bool,
    pub runtime_args: Vec<String>,
    pub all: bool,
}
