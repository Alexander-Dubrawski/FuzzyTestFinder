use std::env;

use crate::{
    cache::Cache,
    errors::FztError,
    runner::{RunnerName, general_runner::GeneralCacheRunner},
    runtime::{Debugger, PythonDebugger, python::pytest::PytestRuntime},
    search_engine::SearchEngine,
    tests::python::{pytest::tests::PytestTests, rust_python::tests::RustPytonTests},
};

use super::{Runner, config::RunnerConfig};

pub fn get_python_runner<SE: SearchEngine + 'static, CM: Cache + Clone + 'static>(
    parser: &str,
    runtime: &str,
    mut config: RunnerConfig<SE>,
    cache_manager: CM,
) -> Result<Box<dyn Runner>, FztError> {
    if let Some(debugger) = config.debugger.as_mut() {
        if debugger == &Debugger::Select {
            let debugger_selection = config
                .search_engine
                .select(&["pdb", "ipdb", "IPython", "pudb", "web-pdb"])?
                .to_lowercase()
                .to_string();
            *debugger = match debugger_selection.as_str() {
                "pdb" => Debugger::Python(PythonDebugger::Pdb),
                "ipdb" => Debugger::Python(PythonDebugger::Ipdb),
                "ipython" => Debugger::Python(PythonDebugger::IPython),
                "pudb" => Debugger::Python(PythonDebugger::Pudb),
                "web-pdb" => Debugger::Python(PythonDebugger::WebPdb),
                _ => {
                    return Err(FztError::InternalError(
                        format!(
                            "Python debugger option `{}` could not be parsed.",
                            debugger_selection.clone()
                        )
                        .to_string(),
                    ));
                }
            };
        }
        if !matches!(debugger, Debugger::Python(_)) {
            return Err(FztError::InvalidArgument(
                "Invalid debugger option. Supported are: Python = [pdb, ipdb, IPython, pudb, web-pdb]"
                    .to_string(),
            ));
        }
    }

    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();
    match (
        parser.to_lowercase().as_str(),
        runtime.to_lowercase().as_str(),
    ) {
        ("rustpython", "pytest") => Ok(Box::new(GeneralCacheRunner::new(
            PytestRuntime::default(),
            config,
            RustPytonTests::new_empty(path_str.to_string()),
            RunnerName::RustPythonRunner,
            cache_manager,
        ))),
        ("pytest", "pytest") => Ok(Box::new(GeneralCacheRunner::new(
            PytestRuntime::default(),
            config,
            PytestTests::new_empty(path_str.to_string()),
            RunnerName::PytestRunner,
            cache_manager,
        ))),
        _ => {
            return Err(FztError::GeneralParsingError(format!(
                "Combination unknown: {parser} {runtime}"
            )));
        }
    }
}
