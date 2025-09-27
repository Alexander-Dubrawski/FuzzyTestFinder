use crate::{runner::config::RunnerConfig, search_engine::fzf::FzfSearchEngine};

pub mod cli_parser;
mod default;

pub struct Config {
    pub runner_config: RunnerConfig<FzfSearchEngine>,
    pub default: bool,
}
