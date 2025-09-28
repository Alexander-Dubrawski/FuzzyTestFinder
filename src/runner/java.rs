use std::env;

use crate::{
    cache::Cache,
    errors::FztError,
    runner::{RunnerName, general_runner::GeneralCacheRunner},
    runtime::{Debugger, java::gradle::GradleRuntime},
    search_engine::SearchEngine,
    tests::java::java_test::JavaTests,
};

use super::{Runner, config::RunnerConfig};

pub fn get_java_runner<SE: SearchEngine + 'static, CM: Cache + Clone + 'static>(
    test_framework: &str,
    runtime: &str,
    config: RunnerConfig<SE>,
    cache_manager: CM,
) -> Result<Box<dyn Runner>, FztError> {
    if let Some(debugger) = config.debugger.as_ref() {
        if !matches!(debugger, Debugger::Java(_)) {
            return Err(FztError::InvalidArgument(
                "Invalid debugger option.".to_string(),
            ));
        }
    }
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();
    match (
        test_framework.to_lowercase().as_str(),
        runtime.to_lowercase().as_str(),
    ) {
        ("junit5", "gradle") => Ok(Box::new(GeneralCacheRunner::new(
            GradleRuntime::default(),
            config,
            JavaTests::new_empty(path_str.to_string()),
            RunnerName::JavaJunit5Runner,
            cache_manager,
            path_str.to_string(),
        ))),
        _ => {
            return Err(FztError::GeneralParsingError(format!(
                "Combination unknown: {test_framework} {runtime}"
            )));
        }
    }
}
