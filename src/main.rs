use std::env;

use FzT::{
    cli::{PythonParser, cli_parser::parse_cli},
    errors::FztError,
    runner::{
        Runner,
        python::{pytest::PytestRunner, rust_python::RustPytonRunner},
    },
    runtime::python::pytest::PytestRuntime,
    search_engine::fzf::FzfSearchEngine,
};

fn main() -> Result<(), FztError> {
    let config = parse_cli()?;
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();

    let search_engine = match config.search_engine {
        FzT::cli::SearchEngine::FzF => FzfSearchEngine::default(),
    };

    match config.language {
        Some(language) => match language {
            FzT::cli::Language::Python((PythonParser::Pytest, _)) => {
                let runner = PytestRunner::new(
                    path_str.to_string(),
                    search_engine,
                    PytestRuntime::default(),
                );
                if config.clear_cache {
                    runner.clear_cache()
                } else {
                    runner.run(config.history, config.last)
                }
            }
            FzT::cli::Language::Python((PythonParser::RustPython, _)) => {
                let runner = RustPytonRunner::new(
                    path_str.to_string(),
                    search_engine,
                    PytestRuntime::default(),
                );
                if config.clear_cache {
                    runner.clear_cache()
                } else {
                    runner.run(config.history, config.last)
                }
            }
        },
        None => {
            // TODO: If more languages supported use auto language detection
            let runner = RustPytonRunner::new(
                path_str.to_string(),
                search_engine,
                PytestRuntime::default(),
            );
            if config.clear_cache {
                runner.clear_cache()
            } else {
                runner.run(config.history, config.last)
            }
        }
    }
}
