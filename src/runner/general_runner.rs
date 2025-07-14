use std::{collections::HashMap, path::PathBuf};

use serde::de::DeserializeOwned;

use crate::{
    cache::manager::{CacheManager, HistoryGranularity},
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    runtime::Runtime,
    search_engine::SearchEngine,
    tests::{Test, Tests},
};

use super::Preview;

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

    fn filter_mode_test(&mut self, query: &Option<String>) -> Result<Vec<String>, FztError> {
        let tests_runtime_args: HashMap<String, String> = HashMap::from_iter(
            self.tests
                .tests()
                .iter()
                .map(|test| (test.name(), test.runtime_argument())),
        );
        Ok(match self.config.mode {
            super::RunnerMode::All => self
                .tests
                .tests()
                .iter()
                .map(|test| test.runtime_argument())
                .collect(),
            super::RunnerMode::Last => {
                let selected_tests = self
                    .cache_manager
                    .recent_history_command(HistoryGranularity::Test)?;
                selected_tests
                    .into_iter()
                    .map(|name| tests_runtime_args[&name].clone())
                    .collect()
            }
            super::RunnerMode::History => {
                let history = self.cache_manager.history(HistoryGranularity::Test)?;
                let selected_tests = self
                    .search_engine
                    .get_from_history(history.as_slice(), query)?;
                if selected_tests.len() > 0 {
                    self.cache_manager
                        .update_history(selected_tests.iter().as_ref(), HistoryGranularity::Test)?;
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
                let selected_tests = self.search_engine.get_tests_to_run(
                    names.as_slice(),
                    &self.config.preview,
                    query,
                )?;
                self.cache_manager
                    .update_history(selected_tests.iter().as_ref(), HistoryGranularity::Test)?;
                let selected_test_runtime: Vec<String> = selected_tests
                    .into_iter()
                    .map(|name| tests_runtime_args[&name].clone())
                    .collect();
                self.cache_manager.update_history(
                    selected_test_runtime.iter().as_ref(),
                    HistoryGranularity::RunTime,
                )?;
                selected_test_runtime
            }
        })
    }

    fn filter_mode_runtime_argument(
        &mut self,
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError> {
        Ok(match self.config.mode {
            super::RunnerMode::All => self
                .tests
                .tests()
                .iter()
                .map(|test| test.runtime_argument())
                .collect(),
            super::RunnerMode::Last => {
                let selected_tests = self
                    .cache_manager
                    .recent_history_command(HistoryGranularity::RunTime)?;
                selected_tests
            }
            super::RunnerMode::History => {
                let history = self.cache_manager.history(HistoryGranularity::RunTime)?;
                let selected_tests = self
                    .search_engine
                    .get_from_history(history.as_slice(), query)?;
                if selected_tests.len() > 0 {
                    self.cache_manager.update_history(
                        selected_tests.iter().as_ref(),
                        HistoryGranularity::RunTime,
                    )?;
                }
                selected_tests
            }
            super::RunnerMode::Select => {
                let runtime_args_test_names: HashMap<String, String> = HashMap::from_iter(
                    self.tests
                        .tests()
                        .iter()
                        .map(|test| (test.runtime_argument(), test.name())),
                );
                let runtime_args: Vec<String> = self
                    .tests
                    .tests()
                    .iter()
                    .map(|test| test.runtime_argument())
                    .collect();
                let selected_test_runtime = self.search_engine.get_tests_to_run(
                    runtime_args
                        .iter()
                        .map(|arg| arg.as_str())
                        .collect::<Vec<&str>>()
                        .as_slice(),
                    &self.config.preview,
                    query,
                )?;
                self.cache_manager.update_history(
                    selected_test_runtime.iter().as_ref(),
                    HistoryGranularity::RunTime,
                )?;
                let selected_test_names: Vec<String> = selected_test_runtime
                    .iter()
                    .map(|name| runtime_args_test_names[name].clone())
                    .collect();
                self.cache_manager.update_history(
                    selected_test_names.iter().as_ref(),
                    HistoryGranularity::Test,
                )?;
                selected_test_runtime
            }
        })
    }

    fn filter_mode_file(&mut self, query: &Option<String>) -> Result<Vec<String>, FztError> {
        let mut tests_runtime_args: HashMap<String, Vec<String>> = HashMap::new();
        for test in self.tests.tests().iter() {
            let file_path = test.file_path();
            if let Some(args) = tests_runtime_args.get_mut(&file_path) {
                args.push(test.runtime_argument());
            } else {
                tests_runtime_args.insert(file_path, vec![test.runtime_argument()]);
            }
        }
        Ok(match self.config.mode {
            super::RunnerMode::All => self
                .tests
                .tests()
                .iter()
                .map(|test| test.runtime_argument())
                .collect(),
            super::RunnerMode::Last => {
                let selected_files = self
                    .cache_manager
                    .recent_history_command(HistoryGranularity::File)?;
                selected_files
                    .into_iter()
                    .flat_map(|file_path| tests_runtime_args[&file_path].clone())
                    .collect()
            }
            super::RunnerMode::History => {
                let history = self.cache_manager.history(HistoryGranularity::File)?;
                let selected_files = self
                    .search_engine
                    .get_from_history(history.as_slice(), query)?;
                if selected_files.len() > 0 {
                    self.cache_manager
                        .update_history(selected_files.iter().as_ref(), HistoryGranularity::File)?;
                }
                selected_files
                    .into_iter()
                    .flat_map(|file_path| tests_runtime_args[&file_path].clone())
                    .collect()
            }
            super::RunnerMode::Select => {
                let file_paths: Vec<&str> = tests_runtime_args
                    .keys()
                    .map(|file_path| file_path.as_str())
                    .collect();
                let selected_files = self.search_engine.get_tests_to_run(
                    file_paths.as_slice(),
                    &self.config.preview,
                    query,
                )?;
                self.cache_manager
                    .update_history(selected_files.iter().as_ref(), HistoryGranularity::File)?;
                selected_files
                    .into_iter()
                    .flat_map(|file_name| tests_runtime_args[&file_name].clone())
                    .collect()
            }
        })
    }

    fn filter_mode_directory(&mut self, query: &Option<String>) -> Result<Vec<String>, FztError> {
        let mut tests_runtime_args: HashMap<String, Vec<String>> = HashMap::new();
        for test in self.tests.tests().iter() {
            let file_path = test.file_path();
            let parent = PathBuf::from(file_path)
                .parent()
                .map(|path| path.to_str().expect("Expect valid path"))
                .unwrap_or("root")
                .to_string();
            if let Some(args) = tests_runtime_args.get_mut(&parent) {
                args.push(test.runtime_argument());
            } else {
                tests_runtime_args.insert(parent.to_string(), vec![test.runtime_argument()]);
            }
        }
        Ok(match self.config.mode {
            super::RunnerMode::All => self
                .tests
                .tests()
                .iter()
                .map(|test| test.runtime_argument())
                .collect(),
            super::RunnerMode::Last => {
                let selected_dictionaries = self
                    .cache_manager
                    .recent_history_command(HistoryGranularity::Directory)?;
                selected_dictionaries
                    .into_iter()
                    .flat_map(|file_path| tests_runtime_args[&file_path].clone())
                    .collect()
            }
            super::RunnerMode::History => {
                let history = self.cache_manager.history(HistoryGranularity::Directory)?;
                let selected_dictionaries = self
                    .search_engine
                    .get_from_history(history.as_slice(), query)?;
                if selected_dictionaries.len() > 0 {
                    self.cache_manager.update_history(
                        selected_dictionaries.iter().as_ref(),
                        HistoryGranularity::Directory,
                    )?;
                }
                selected_dictionaries
                    .into_iter()
                    .flat_map(|file_path| tests_runtime_args[&file_path].clone())
                    .collect()
            }
            super::RunnerMode::Select => {
                let file_paths: Vec<&str> = tests_runtime_args
                    .keys()
                    .map(|file_path| file_path.as_str())
                    .collect();
                // If preview is set always use Directory
                let preview = if self.config.preview.is_some() {
                    Some(Preview::Directory)
                } else {
                    None
                };
                let selected_dictionaries =
                    self.search_engine
                        .get_tests_to_run(file_paths.as_slice(), &preview, query)?;
                self.cache_manager.update_history(
                    selected_dictionaries.iter().as_ref(),
                    HistoryGranularity::Directory,
                )?;
                selected_dictionaries
                    .into_iter()
                    .flat_map(|file_name| tests_runtime_args[&file_name].clone())
                    .collect()
            }
        })
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
        let tests_to_run: Vec<String> = match self.config.filter_mode {
            super::FilterMode::Test => self.filter_mode_test(&self.config.query.clone())?,
            super::FilterMode::File => self.filter_mode_file(&self.config.query.clone())?,
            super::FilterMode::Directory => {
                self.filter_mode_directory(&self.config.query.clone())?
            }
            super::FilterMode::RunTime => {
                self.filter_mode_runtime_argument(&self.config.query.clone())?
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
