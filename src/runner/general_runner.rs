use serde::de::DeserializeOwned;

use crate::{
    cache::manager::CacheManager,
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    runtime::Runtime,
    search_engine::SearchEngine,
    tests::{Test, Tests},
};

pub fn get_tests<SE: SearchEngine, T: Tests>(
    history: bool,
    last: bool,
    cache_manager: &CacheManager,
    search_engine: &SE,
    tests: &T,
    all: bool,
) -> Result<Vec<String>, FztError> {
    if all {
        Ok(tests
            .tests()
            .iter()
            .map(|test| test.runtime_argument())
            .collect())
    } else if last {
        cache_manager.recent_history_command()
    } else if history {
        let history = cache_manager.history()?;
        let selected_tests = search_engine.get_from_history(history)?;
        if selected_tests.len() > 0 {
            cache_manager.update_history(selected_tests.iter().as_ref())?;
        }
        Ok(selected_tests)
    } else {
        let selected_tests = search_engine.get_tests_to_run(tests)?;
        cache_manager.update_history(selected_tests.iter().as_ref())?;
        Ok(selected_tests)
    }
}

pub struct GeneralCacheRunner<SE: SearchEngine, RT: Runtime, T: Tests> {
    tests: T,
    cache_manager: CacheManager,
    search_engine: SE,
    runtime: RT,
    config: RunnerConfig,
    runner_name: RunnerName,
}

impl<SE: SearchEngine, RT: Runtime, T: Tests> GeneralCacheRunner<SE, RT, T> {
    pub fn new(
        search_engine: SE,
        runtime: RT,
        config: RunnerConfig,
        tests: T,
        project_id: String,
        runner_name: RunnerName,
    ) -> Self {
        let cache_manager = CacheManager::new(project_id);

        Self {
            tests,
            cache_manager,
            search_engine,
            runtime,
            config,
            runner_name,
        }
    }
}

impl<SE: SearchEngine, RT: Runtime, T: Tests + DeserializeOwned> Runner
    for GeneralCacheRunner<SE, RT, T>
{
    fn run(&mut self) -> Result<(), FztError> {
        if self.config.clear_cache || self.config.clear_history {
            if self.config.clear_cache {
                self.cache_manager.clear_cache()?;
            }
            if self.config.clear_history {
                self.cache_manager.clear_history()?;
            }
            return Ok(());
        }
        if let Some(reader) = self.cache_manager.get_entry()? {
            self.tests = serde_json::from_reader(reader)?;
            if self.tests.update()? {
                self.cache_manager
                    .add_entry(self.tests.to_json()?.as_str())?;
            }
        }
        let selected_tests = get_tests(
            self.config.history,
            self.config.last,
            &self.cache_manager,
            &self.search_engine,
            &self.tests,
            self.config.all,
        )?;
        // TODO: Improve Filter for tests
        let test_items: Vec<String> = self
            .tests
            .tests()
            .into_iter()
            .filter(|test| {
                let search_name = test.name();
                selected_tests.contains(&search_name)
            })
            .map(|test| test.runtime_argument())
            .collect();
        if !test_items.is_empty() {
            self.runtime.run_tests(
                test_items,
                self.config.verbose,
                &self.config.runtime_args.as_slice(),
            )
        } else {
            Ok(())
        }
    }

    fn meta_data(&self) -> Result<String, FztError> {
        let meta_data = MetaData {
            runner_name: self.runner_name.clone(),
            search_engine: self.search_engine.name(),
            runtime: self.runtime.name(),
        };
        let json = serde_json::to_string(&meta_data)?;
        Ok(json)
    }
}
