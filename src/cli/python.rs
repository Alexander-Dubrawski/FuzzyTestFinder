use std::env;

use crate::{
    cache::helper::project_hash,
    errors::FztError,
    runner::{
        Runner, RunnerConfig,
        python::{pytest::PytestRunner, rust_python::RustPythonRunner},
    },
    runtime::python::pytest::PytestRuntime,
    search_engine::SearchEngine,
};

pub fn get_python_runner<SE: SearchEngine + 'static>(
    parser: &str,
    runtime: &str,
    config: RunnerConfig,
    search_engine: SE,
) -> Result<Box<dyn Runner>, FztError> {
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();
    match (
        parser.to_lowercase().as_str(),
        runtime.to_lowercase().as_str(),
    ) {
        ("rustpython", "pytest") => Ok(Box::new(RustPythonRunner::new(
            path_str.to_string(),
            search_engine,
            PytestRuntime::default(),
            config,
            project_hash()?,
        ))),
        ("pytest", "pytest") => Ok(Box::new(PytestRunner::new(
            path_str.to_string(),
            search_engine,
            PytestRuntime::default(),
            config,
            project_hash()?,
        ))),
        _ => {
            return Err(FztError::GeneralParsingError(format!(
                "Combination unknown: {parser} {runtime}"
            )));
        }
    }
}
