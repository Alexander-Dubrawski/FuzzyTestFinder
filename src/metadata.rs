use crate::{
    cache::{manager::CacheManager, types::MetaData},
    cli::Config,
    errors::FztError,
};

pub fn handle_metadata(config: &mut Config, project_id: String) -> Result<(), FztError> {
    if config.language.is_none() {
        if let Some(meta_data) = CacheManager::get_meta(project_id.as_str())? {
            config.language = meta_data.language;
        }
    }

    if config.search_engine.is_none() {
        if let Some(meta_data) = CacheManager::get_meta(project_id.as_str())? {
            config.search_engine = meta_data.search_engine;
        }
    }

    if config.default {
        CacheManager::save_meta(
            project_id.as_str(),
            MetaData {
                language: config.language.clone(),
                search_engine: config.search_engine.clone(),
            },
        )?;
    }

    Ok(())
}
