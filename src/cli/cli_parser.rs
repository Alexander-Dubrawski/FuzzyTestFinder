use clap::{Parser, Subcommand};

use crate::{
    cli::{Language, PythonParser, PythonRuntime, SearchEngine},
    errors::FztError,
};

use super::Config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, default_value_t = String::from("FzF"), value_parser=["FzF"])]
    search_engine: String,

    #[arg(long, default_value_t = false)]
    clear_cache: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Python {
        #[arg(default_value_t = String::from("RustPython"), value_parser=["RustPython", "PyTest"])]
        parser: String,

        #[arg(default_value_t = String::from("Pytest"), value_parser=["PyTest"])]
        runtime: String,
    },
}

pub fn parse_cli() -> Result<Config, FztError> {
    let cli = Cli::parse();

    let search_engine = match cli.search_engine.to_lowercase().as_str() {
        "fzf" => Ok(SearchEngine::FzF),
        _ => Err(FztError::UserError(format!(
            "Unknown search engine: {} Supported are: fzf",
            cli.search_engine.to_lowercase()
        ))),
    }?;

    let language = match &cli.command {
        Some(Commands::Python { parser, runtime }) => {
            let parser = match parser.to_lowercase().as_str() {
                "pytest" => Ok(PythonParser::Pytest),
                "rustpython" => Ok(PythonParser::RustPython),
                _ => Err(FztError::UserError(format!(
                    "Unknown parser: {} Supported are: pytest, rustpython",
                    parser.to_lowercase()
                ))),
            }?;
            let runtime = match runtime.to_lowercase().as_str() {
                "pytest" => Ok(PythonRuntime::Pytest),
                _ => Err(FztError::UserError(format!(
                    "Unknown runtime: {} Supported are: pytest",
                    runtime.to_lowercase()
                ))),
            }?;
            Ok::<Option<Language>, FztError>(Some(Language::Python((parser, runtime))))
        }
        None => Ok(None),
    }?;
    Ok(Config {
        language,
        search_engine,
        clear_cache: cli.clear_cache,
    })
}
