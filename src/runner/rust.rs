use std::env;

use crate::{
    cache::helper::project_hash,
    errors::FztError,
    runner::{RunnerName, general_runner::GeneralCacheRunner},
    runtime::{Debugger, rust::cargo::CargoRuntime},
    search_engine::SearchEngine,
    tests::rust::rust_test::RustTests,
};

use super::{Runner, config::RunnerConfig};

pub fn get_rust_runner<SE: SearchEngine + 'static>(
    config: RunnerConfig<SE>,
) -> Result<Box<dyn Runner>, FztError> {
    if let Some(debugger) = config.debugger.as_ref() {
        if !matches!(debugger, Debugger::Rust(_)) {
            return Err(FztError::InvalidArgument(
                "Invalid debugger option.".to_string(),
            ));
        }
    }
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();
    Ok(Box::new(GeneralCacheRunner::new(
        CargoRuntime::default(),
        config,
        RustTests::new_empty(path_str.to_string()),
        format!("{}-rust-cargo", project_hash()?),
        RunnerName::RustCargoRunner,
    )))
}
