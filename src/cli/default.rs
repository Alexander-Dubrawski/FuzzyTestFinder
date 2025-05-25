use crate::{
    cache::manager::CacheManager,
    errors::FztError,
    runner::{Runner, RunnerConfig},
    search_engine::fzf::FzfSearchEngine,
};

use super::{java::get_java_runner, python::get_python_runner};

pub fn get_default(project_id: &str, config: RunnerConfig) -> Result<Box<dyn Runner>, FztError> {
    let meta_data = CacheManager::get_meta(project_id)?;
    match meta_data {
        Some(meta) => match meta.runner_name {
            crate::runner::RunnerName::RustPythonRunner => get_python_runner(
                "rustpython",
                meta.runtime.as_str(),
                config,
                FzfSearchEngine::default(),
            ),
            crate::runner::RunnerName::PytestRunner => get_python_runner(
                "pytest",
                meta.runtime.as_str(),
                config,
                FzfSearchEngine::default(),
            ),
            crate::runner::RunnerName::JavaJunit5Runner => get_java_runner(
                "junit5",
                meta.runtime.as_str(),
                config,
                FzfSearchEngine::default(),
            ),
        },
        None => todo!(),
    }
}

pub fn set_default(project_id: &str, meta_data: &str) -> Result<(), FztError> {
    CacheManager::save_meta(project_id, meta_data)
}
