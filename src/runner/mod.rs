use serde::{Deserialize, Serialize};

use crate::errors::FztError;

pub mod general_runner;

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

pub enum RunnerMode {
    All,
    Last,
    History,
    Select,
}

pub struct RunnerConfig {
    pub clear_cache: bool,
    pub verbose: bool,
    pub clear_history: bool,
    pub runtime_args: Vec<String>,
    pub mode: RunnerMode,
}

impl RunnerConfig {
    pub fn new(
        clear_cache: bool,
        verbose: bool,
        clear_history: bool,
        runtime_args: Vec<String>,
        mode: RunnerMode,
    ) -> Self {
        Self {
            clear_cache,
            verbose,
            clear_history,
            runtime_args,
            mode,
        }
    }
}
