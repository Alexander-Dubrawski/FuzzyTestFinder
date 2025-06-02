use std::env;

use crate::{
    cache::helper::project_hash,
    errors::FztError,
    runner::{Runner, RunnerConfig, RunnerName, general_runner::GeneralCacheRunner},
    runtime::python::pytest::PytestRuntime,
    search_engine::SearchEngine,
    tests::python::{pytest::tests::PytestTests, rust_python::tests::RustPytonTests},
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
        ("rustpython", "pytest") => Ok(Box::new(GeneralCacheRunner::new(
            search_engine,
            PytestRuntime::default(),
            config,
            RustPytonTests::new_empty(path_str.to_string()),
            format!("{}-rust-python", project_hash()?),
            RunnerName::RustPythonRunner,
        ))),
        ("pytest", "pytest") => Ok(Box::new(GeneralCacheRunner::new(
            search_engine,
            PytestRuntime::default(),
            config,
            PytestTests::new_empty(path_str.to_string()),
            format!("{}-pytest", project_hash()?),
            RunnerName::PytestRunner,
        ))),
        _ => {
            return Err(FztError::GeneralParsingError(format!(
                "Combination unknown: {parser} {runtime}"
            )));
        }
    }
}
