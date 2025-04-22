pub mod cli_parser;

pub enum Language {
    Python((PythonParser, PythonRuntime)),
}

pub enum PythonParser {
    Pytest,
    RustPython,
}

pub enum PythonRuntime {
    Pytest,
}

pub enum SearchEngine {
    FzF,
}

pub struct Config {
    pub language: Option<Language>,
    pub search_engine: SearchEngine,
    pub clear_cache: bool,
    pub history: bool,
    pub last: bool,
}
