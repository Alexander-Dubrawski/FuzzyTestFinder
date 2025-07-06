use crate::{
    cache::manager::CacheManager,
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    search_engine::fzf::FzfSearchEngine,
};

use super::{java::get_java_runner, python::get_python_runner, rust::get_rust_runner};

pub fn get_default(project_id: &str, config: RunnerConfig) -> Result<Box<dyn Runner>, FztError> {
    let reader = CacheManager::get_meta(project_id)?;
    let meta_data: MetaData = match reader {
        Some(reader) => serde_json::from_reader(reader)?,
        None => {
            return Err(FztError::GeneralParsingError(
                "Metadata not found".to_string(),
            ));
        }
    };

    match meta_data.runner_name {
        RunnerName::RustPythonRunner => get_python_runner(
            "rustpython",
            meta_data.runtime.as_str(),
            config,
            FzfSearchEngine::default(),
        ),
        RunnerName::PytestRunner => get_python_runner(
            "pytest",
            meta_data.runtime.as_str(),
            config,
            FzfSearchEngine::default(),
        ),
        RunnerName::JavaJunit5Runner => get_java_runner(
            "junit5",
            meta_data.runtime.as_str(),
            config,
            FzfSearchEngine::default(),
        ),
        RunnerName::RustCargoRunner => get_rust_runner(config, FzfSearchEngine::default()),
    }
}

pub fn set_default(project_id: &str, meta_data: &str) -> Result<(), FztError> {
    CacheManager::save_meta(project_id, meta_data)
}
