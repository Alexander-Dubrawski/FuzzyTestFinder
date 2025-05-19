use std::env;

use FzT::{
    cache::helper::project_hash,
    cli::{JavaRuntime, PythonParser, cli_parser::parse_cli},
    errors::FztError,
    metadata::handle_metadata,
    runner::{
        Runner,
        java::java::JavaRunner,
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

    let search_engine = match config.clone().search_engine {
        Some(search_engine) => match search_engine {
            FzT::cli::SearchEngine::FzF => FzfSearchEngine::default(),
        },
        None => FzfSearchEngine::default(),
    };

    match config.clone().language {
        Some(language) => match language {
            FzT::cli::Language::Python((PythonParser::Pytest, _)) => PytestRunner::new(
                path_str.to_string(),
                search_engine,
                PytestRuntime::default(),
                config,
            )
            .run(),
            FzT::cli::Language::Python((PythonParser::RustPython, _)) => RustPytonRunner::new(
                path_str.to_string(),
                search_engine,
                PytestRuntime::default(),
                config,
            )
            .run(),
            FzT::cli::Language::Java((JavaRuntime, JavaRuntime::Gradle)) => JavaRunner::new(
                path_str.to_string(),
                search_engine,
                GradleRuntime::default(),
                config,
            )
            .run(),
        },
        None => {
            // TODO: If more languages supported use auto language detection
            RustPytonRunner::new(
                path_str.to_string(),
                search_engine,
                PytestRuntime::default(),
                config,
            )
            .run()
        }
    }
}
