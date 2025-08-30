use serde::{Deserialize, Serialize};

use crate::{errors::FztError, runtime::Debugger};

pub mod general_runner;
mod history_provider;

pub trait Runner {
    fn run(&mut self) -> Result<(), FztError>;
    fn meta_data(&self) -> Result<String, FztError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RunnerName {
    RustPythonRunner,
    PytestRunner,
    JavaJunit5Runner,
    RustCargoRunner,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaData {
    pub runner_name: RunnerName,
    pub search_engine: String,
    pub runtime: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RunnerMode {
    All,
    Last,
    History,
    Select,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Preview {
    File,
    Test,
    Directory,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FilterMode {
    Test,
    File,
    Directory,
    RunTime,
    Failed,
    Append,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunnerConfig {
    pub clear_cache: bool,
    pub verbose: bool,
    pub clear_history: bool,
    pub runtime_args: Vec<String>,
    pub mode: RunnerMode,
    pub preview: Option<Preview>,
    pub filter_mode: FilterMode,
    pub query: Option<String>,
    pub debugger: Option<Debugger>,
}

impl RunnerConfig {
    pub fn new(
        clear_cache: bool,
        verbose: bool,
        clear_history: bool,
        runtime_args: Vec<String>,
        mode: RunnerMode,
        preview: Option<Preview>,
        filter_mode: FilterMode,
        query: Option<String>,
        debugger: Option<Debugger>,
    ) -> Self {
        Self {
            clear_cache,
            verbose,
            clear_history,
            runtime_args,
            mode,
            preview,
            filter_mode,
            query,
            debugger,
        }
    }
}
