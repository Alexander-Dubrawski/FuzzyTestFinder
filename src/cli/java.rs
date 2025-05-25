use std::env;

use crate::{
    errors::FztError,
    runner::{Runner, RunnerConfig, java::java::JavaJunit5Runner},
    runtime::java::gradle::GradleRuntime,
    search_engine::SearchEngine,
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
        ("junit5", "gradle") => Ok(Box::new(JavaJunit5Runner::new(
            path_str.to_string(),
            search_engine,
            GradleRuntime::default(),
            config,
        ))),
        _ => todo!(),
    }
}
