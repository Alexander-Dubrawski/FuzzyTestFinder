use std::env;

use crate::{
    cache::helper::project_hash,
    errors::FztError,
    runner::{Runner, RunnerConfig, RunnerName, general_runner::GeneralCacheRunner},
    runtime::rust::cargo::CargoRuntime,
    search_engine::SearchEngine,
    tests::rust::rust_test::RustTests,
};

pub fn get_rust_runner<SE: SearchEngine + 'static>(
    config: RunnerConfig,
    search_engine: SE,
) -> Result<Box<dyn Runner>, FztError> {
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();
    Ok(Box::new(GeneralCacheRunner::new(
        search_engine,
        CargoRuntime::default(),
        config,
        RustTests::new_empty(path_str.to_string()),
        format!("{}-rust-cargo", project_hash()?),
        RunnerName::RustCargoRunner,
    )))
}
