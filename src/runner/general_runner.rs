use std::collections::HashMap;

use serde::de::DeserializeOwned;

use crate::{
    cache::manager::CacheManager,
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    runtime::Runtime,
    search_engine::SearchEngine,
    tests::{Test, Tests},
};

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
        } else {
            self.tests.update()?;
            self.cache_manager
                .add_entry(self.tests.to_json()?.as_str())?;
        }
        let tests_runtime_args: HashMap<String, String> = HashMap::from_iter(
            self.tests
                .tests()
                .iter()
                .map(|test| (test.name(), test.runtime_argument())),
        );
        let tests_to_run: Vec<String> = match self.config.mode {
            super::RunnerMode::All => self
                .tests
                .tests()
                .iter()
                .map(|test| test.runtime_argument())
                .collect(),
            super::RunnerMode::Last => {
                let selected_tests = self.cache_manager.recent_history_command()?;
                selected_tests
                    .into_iter()
                    .map(|name| tests_runtime_args[&name].clone())
                    .collect()
            }
            super::RunnerMode::History => {
                let history = self.cache_manager.history()?;
                let selected_tests = self.search_engine.get_from_history(history.as_slice())?;
                if selected_tests.len() > 0 {
                    self.cache_manager
                        .update_history(selected_tests.iter().as_ref())?;
                }
                selected_tests
                    .into_iter()
                    .map(|name| tests_runtime_args[&name].clone())
                    .collect()
            }
            super::RunnerMode::Select => {
                let names: Vec<&str> = tests_runtime_args
                    .keys()
                    .map(|name| name.as_str())
                    .collect();
                let selected_tests = self
                    .search_engine
                    .get_tests_to_run(names.as_slice(), &self.config.preview)?;
                self.cache_manager
                    .update_history(selected_tests.iter().as_ref())?;
                selected_tests
                    .into_iter()
                    .map(|name| tests_runtime_args[&name].clone())
                    .collect()
            }
        };
        if !tests_to_run.is_empty() {
            self.runtime.run_tests(
                tests_to_run,
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
