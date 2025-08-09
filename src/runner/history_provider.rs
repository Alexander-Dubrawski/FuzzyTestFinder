use crate::{
    cache::manager::{CacheManager, HistoryGranularity},
    errors::FztError,
    search_engine::SearchEngine,
};

pub struct HistoryProvider {
    cache_manager: CacheManager,
}

impl HistoryProvider {
    pub fn new(cache_manager: CacheManager) -> Self {
        Self { cache_manager }
    }

    pub fn history<SE: SearchEngine>(
        &self,
        granularity: &HistoryGranularity,
        search_engine: &SE,
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError> {
        {
            let history = self.cache_manager.history(granularity)?;
            let selection = search_engine.get_from_history(history.as_slice(), query)?;
            if selection.len() > 0 {
                self.cache_manager
                    .update_history(selection.iter().as_ref(), granularity)?;
            }
            Ok(selection)
        }
    }

    pub fn last(&self, granularity: &HistoryGranularity) -> Result<Vec<String>, FztError> {
        Ok(self.cache_manager.recent_history_command(granularity)?)
    }

    pub fn update_history(
        &self,
        granularity: &HistoryGranularity,
        update: &[String],
    ) -> Result<(), FztError> {
        self.cache_manager
            .update_history(update.iter().as_ref(), granularity)?;
        Ok(())
    }
}
