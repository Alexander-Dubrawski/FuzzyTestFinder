use std::env;

use crate::{
    cache::helper::project_hash,
    errors::FztError,
    runner::{Runner, RunnerConfig, RunnerName, general_runner::GeneralCacheRunner},
    runtime::java::gradle::GradleRuntime,
    search_engine::SearchEngine,
    tests::java::java_test::JavaTests,
};

pub fn get_java_runner<SE: SearchEngine + 'static>(
    test_framework: &str,
    runtime: &str,
    config: RunnerConfig,
    search_engine: SE,
) -> Result<Box<dyn Runner>, FztError> {
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();
    match (
        test_framework.to_lowercase().as_str(),
        runtime.to_lowercase().as_str(),
    ) {
        ("junit5", "gradle") => Ok(Box::new(GeneralCacheRunner::new(
            search_engine,
            GradleRuntime::default(),
            config,
            JavaTests::new_empty(path_str.to_string()),
            format!("{}-java-junit5", project_hash()?),
            RunnerName::JavaJunit5Runner,
        ))),
        _ => {
            return Err(FztError::GeneralParsingError(format!(
                "Combination unknown: {test_framework} {runtime}"
            )));
        }
    }
}
