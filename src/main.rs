use std::env;

use FzT::{
    cache::helper::project_hash,
    cli::{PythonParser, cli_parser::parse_cli},
    errors::FztError,
    metadata::handle_metadata,
    runner::{
        Runner,
        python::{pytest::PytestRunner, rust_python::RustPytonRunner},
    },
    runtime::python::pytest::PytestRuntime,
    search_engine::fzf::FzfSearchEngine,
};

fn main() -> Result<(), FztError> {
    let mut config = parse_cli()?;
    let path = env::current_dir()?;
    let path_str = path.to_string_lossy();

    let project_id = project_hash(path_str.to_string());

    handle_metadata(&mut config, project_id)?;

    let search_engine = match config.search_engine {
        Some(search_engine) => match search_engine {
            FzT::cli::SearchEngine::FzF => FzfSearchEngine::default(),
        },
        None => FzfSearchEngine::default(),
    };

    match config.language {
        Some(language) => match language {
            FzT::cli::Language::Python((PythonParser::Pytest, _)) => {
                let runner = PytestRunner::new(
                    path_str.to_string(),
                    search_engine,
                    PytestRuntime::default(),
                );
                if config.clear_cache || config.clear_history {
                    if config.clear_cache {
                        runner.clear_cache()?;
                    }
                    if config.clear_history {
                        runner.clear_history()?;
                    }
                    Ok(())
                } else {
                    runner.run(config.history, config.last, config.verbose, config.debug)
                }
            }
            FzT::cli::Language::Python((PythonParser::RustPython, _)) => {
                let runner = RustPytonRunner::new(
                    path_str.to_string(),
                    search_engine,
                    PytestRuntime::default(),
                );
                if config.clear_cache || config.clear_history {
                    if config.clear_cache {
                        runner.clear_cache()?;
                    }
                    if config.clear_history {
                        runner.clear_history()?;
                    }
                    Ok(())
                } else {
                    runner.run(config.history, config.last, config.verbose, config.debug)
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
            if config.clear_cache || config.clear_history {
                if config.clear_cache {
                    runner.clear_cache()?;
                }
                if config.clear_history {
                    runner.clear_history()?;
                }
                Ok(())
            } else {
                runner.run(config.history, config.last, config.verbose, config.debug)
            }
        }
    }
}
