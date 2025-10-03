use FzT::{
    cache::{helper::project_hash, manager::LocalCacheManager},
    cli::cli_parser::parse_cli,
    errors::FztError,
    watcher::local::watch,
};

fn main() -> Result<(), FztError> {
    let config = parse_cli()?;
    let default = config.default;
    let watch_change = config.watch;
    let mut runner = config.runner_config.clone().into_runner()?;
    if default {
        LocalCacheManager::save_meta(project_hash()?.as_str(), runner.meta_data()?.as_str())?;
    }
    if watch_change {
        watch(config.runner_config)
    } else {
        runner.run(None)
    }
}
