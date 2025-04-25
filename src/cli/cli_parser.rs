use clap::{Parser, Subcommand};

use crate::{
    cli::{Language, PythonParser, PythonRuntime, SearchEngine},
    errors::FztError,
};

use super::Config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, value_parser=["FzF"])]
    search_engine: Option<String>,

    #[arg(long, default_value_t = false)]
    clear_cache: bool,

    #[arg(long, default_value_t = false)]
    default: bool,

    #[arg(long, default_value_t = false, short)]
    last: bool,

    #[arg(long, default_value_t = false, short)]
    history: bool,

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
    Rust,
}

pub fn parse_cli() -> Result<Config, FztError> {
    let cli = Cli::parse();

    let search_engine = if let Some(search_engine) = cli.search_engine {
        match search_engine.to_lowercase().as_str() {
            "fzf" => Ok(Some(SearchEngine::FzF)),
            _ => Err(FztError::UserError(format!(
                "Unknown search engine: {} Supported are: fzf",
                search_engine.to_lowercase()
            ))),
        }?
    } else {
        None
    };

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
        Some(Commands::Rust) => Ok::<Option<Language>, FztError>(Some(Language::Rust)),
        None => Ok(None),
    }?;
    Ok(Config {
        language,
        search_engine,
        clear_cache: cli.clear_cache,
        history: cli.history,
        last: cli.last,
        default: cli.default,
    })
}
