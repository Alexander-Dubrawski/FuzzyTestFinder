use crate::{runner::config::RunnerConfig, search_engine::fzf::FzfSearchEngine};

pub mod cli_parser;
mod default;

#[derive(Debug, Clone)]
pub struct Config {
    pub runner_config: RunnerConfig<FzfSearchEngine>,
    pub default: bool,
    pub watch: bool,
}
