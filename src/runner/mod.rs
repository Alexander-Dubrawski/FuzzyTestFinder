use crate::{cli::Config, errors::FztError};

pub mod python;

pub trait Runner {
    fn run(&self) -> Result<(), FztError>;
}

pub struct RunnerConfig {
    pub clear_cache: bool,
    pub history: bool,
    pub last: bool,
    pub verbose: bool,
    pub debug: bool,
    pub clear_history: bool,
    pub runtime_args: Vec<String>,
}

impl From<Config> for RunnerConfig {
    fn from(value: Config) -> Self {
        Self {
            clear_cache: value.clear_cache,
            history: value.history,
            last: value.last,
            verbose: value.verbose,
            debug: value.debug,
            clear_history: value.clear_history,
            runtime_args: value.runtime_args,
        }
    }
}
