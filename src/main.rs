use FzT::{
    cache::{helper::project_hash, manager::LocalCacheManager},
    cli::cli_parser::parse_cli,
    errors::FztError,
};

fn main() -> Result<(), FztError> {
    let config = parse_cli()?;
    let default = config.default;
    let mut runner = config.runner_config.into_runner()?;
    if default {
        LocalCacheManager::save_meta(project_hash()?.as_str(), runner.meta_data()?.as_str())?;
    }
    runner.run()
}
