use serde::{Deserialize, Serialize};

use crate::errors::FztError;

pub mod config;
pub mod general_runner;
pub mod java;
pub mod python;
pub mod rust;

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
