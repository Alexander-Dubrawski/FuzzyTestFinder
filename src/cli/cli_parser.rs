use clap::{Command, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::{
    cache::helper::project_hash,
    errors::FztError,
    runner::{Runner, RunnerConfig, RunnerMode},
    search_engine::fzf::FzfSearchEngine,
};

use super::{
    default::{get_default, set_default},
    java::get_java_runner,
    python::get_python_runner,
    rust::get_rust_runner,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, value_parser=["FzF"])]
    search_engine: Option<String>,

    #[arg(
        long,
        default_value_t = false,
        help = "Clear test build directory cache"
    )]
    clear_cache: bool,

    #[arg(long, short, default_value_t = false)]
    default: bool,

    #[arg(
        long,
        default_value_t = false,
        short,
        help = "Run recently used test command"
    )]
    last: bool,

    #[arg(
        long,
        default_value_t = false,
        short,
        help = "Parse tests commands from history"
    )]
    history: bool,

    #[arg(long, default_value_t = false, short, help = "Clear history")]
    clear_history: bool,

    #[arg(long, default_value_t = false, short)]
    verbose: bool,

    #[arg(
        long,
        default_value_t = false,
        short,
        help = "Run all tests in project"
    )]
    all: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Python {
        #[arg(default_value_t = String::from("RustPython"), value_parser=["RustPython", "PyTest"])]
        parser: String,

        #[arg(default_value_t = String::from("PyTest"), value_parser=["PyTest"])]
        runtime: String,
    },
    Java {
        #[arg(default_value_t = String::from("JUnit5"), value_parser=["JUnit5"])]
        test_framework: String,

        #[arg(default_value_t = String::from("gradle"), value_parser=["gradle"])]
        runtime: String,
    },
    Rust,
}

fn parse_args(cmd: Command) -> (Cli, Vec<String>) {
    let raw_args: Vec<String> = std::env::args().collect();
    let dash_dash_pos = raw_args.iter().position(|arg| arg == "--");

    let (cli_args, runtime_args) = match dash_dash_pos {
        Some(pos) => {
            // Split at -- position
            let cli = raw_args[..pos].to_vec();
            let runtime = if pos + 1 < raw_args.len() {
                raw_args[pos + 1..].to_vec()
            } else {
                Vec::new()
            };
            (cli, runtime)
        }
        None => (raw_args, Vec::new()),
    };

    let matches = cmd.get_matches_from(cli_args);
    let cli = Cli::from_arg_matches(&matches).expect("Failed to parse arguments");

    (cli, runtime_args)
}

fn configure_commands() -> Command {
    let mut cmd = Cli::command();

    cmd = cmd.override_usage("FzT [OPTIONS] [COMMAND] [ARGS]... [-- RUNTIME_ARGS]...");
    cmd = cmd.after_help(
        "Runtime Arguments:\n  \
        Arguments after -- are passed directly to the runtime\n  \
        Example: fzt -v python RustPython PyTest -- --pdb",
    );

    if let Some(python_cmd) = cmd.find_subcommand_mut("python") {
        *python_cmd = python_cmd
            .clone()
            .override_usage("FzT python [ARGS]... [-- RUNTIME_ARGS]...");
        *python_cmd = python_cmd.clone().after_help(
            "Runtime Arguments:\n  \
            Arguments after -- are passed directly to the runtime\n  \
            Example: fzt python RustPython PyTest -- --pdb",
        );
    }

    cmd
}

pub fn parse_cli() -> Result<Box<dyn Runner>, FztError> {
    let cmd = configure_commands();
    let (cli, runtime_args) = parse_args(cmd);

    let mode = if cli.all {
        RunnerMode::All
    } else if cli.last {
        RunnerMode::Last
    } else if cli.history {
        RunnerMode::History
    } else {
        RunnerMode::Select
    };

    let runner_config = RunnerConfig::new(
        cli.clear_cache,
        cli.verbose,
        cli.clear_history,
        runtime_args,
        mode,
    );

    let runner = match &cli.command {
        Some(Commands::Python { parser, runtime }) => {
            get_python_runner(parser, runtime, runner_config, FzfSearchEngine::default())
        }
        Some(Commands::Java {
            test_framework,
            runtime,
        }) => get_java_runner(
            test_framework,
            runtime,
            runner_config,
            FzfSearchEngine::default(),
        ),
        Some(Commands::Rust) => get_rust_runner(runner_config, FzfSearchEngine::default()),
        None => get_default(project_hash()?.as_str(), runner_config),
    }?;
    if cli.default {
        set_default(project_hash()?.as_str(), runner.meta_data()?.as_str())?;
    }
    Ok(runner)
}
