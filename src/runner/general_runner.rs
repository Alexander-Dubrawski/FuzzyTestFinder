use std::{collections::HashMap, str::FromStr};

use serde::de::DeserializeOwned;

use crate::{
    cache::manager::{CacheManager, HistoryGranularity},
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    runtime::Runtime,
    search_engine::{Append, SearchEngine},
    tests::{
        Tests,
        test_provider::{SelectGranularity, TestProvider},
    },
};

use super::{Preview, history_provider::HistoryProvider};

fn append_selection_to_preview(selection: &HashMap<SelectGranularity, Vec<String>>) -> String {
    let mut preview = String::new();
    selection.iter().for_each(|(select, selected_items)| {
        preview.push_str(&format!("{}\n", select));
        preview.push_str("-".repeat(select.to_string().len()).as_str());
        preview.push('\n');
        preview.push_str(&selected_items.join("\n"));
        preview.push('\n');
        preview.push('\n');
    });
    preview
}

fn parse_append_history(history: Vec<String>) -> HashMap<SelectGranularity, Vec<String>> {
    let mut selection = HashMap::new();
    history.into_iter().for_each(|test| {
        let mut parts = test.splitn(2, ' ');
        let first = parts
            .next()
            .expect(format!("THIS IS A BUG. History parts should contain two parts").as_str());
        let selected_items = parts
            .next()
            .expect(format!("THIS IS A BUG. History parts should contain two parts").as_str())
            .to_string();
        let select = SelectGranularity::from_str(first)
            .expect(format!("THIS IS A BUG. {first} should map to SelectGranularity").as_str());
        selection
            .entry(select)
            .or_insert(vec![])
            .push(selected_items);
    });
    selection
}

pub struct GeneralCacheRunner<SE: SearchEngine, RT: Runtime, T: Tests> {
    tests: T,
    cache_manager: CacheManager,
    search_engine: SE,
    runtime: RT,
    config: RunnerConfig,
    runner_name: RunnerName,
    history_provider: HistoryProvider,
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
        let cache_manager = if config.run_failed {
            CacheManager::new_failed_tests(project_id)
        } else {
            CacheManager::new(project_id)
        };
        let history_provider = HistoryProvider::new(cache_manager.clone());

        Self {
            tests,
            cache_manager,
            search_engine,
            runtime,
            config,
            runner_name,
            history_provider,
        }
    }

    fn select_tests(
        &mut self,
        granularity: &SelectGranularity,
        test_provider: &TestProvider,
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError> {
        let preview = if granularity == &SelectGranularity::Directory {
            if self.config.preview.is_some() {
                Some(Preview::Directory)
            } else {
                None
            }
        } else if granularity == &SelectGranularity::RunTime {
            None
        } else {
            self.config.preview.clone()
        };
        Ok(self.search_engine.get_tests_to_run(
            test_provider.select_option(granularity).as_slice(),
            &preview,
            query,
        )?)
    }

    fn get_tests_to_run(
        &mut self,
        query: &Option<String>,
        test_provider: &TestProvider,
        history_granularity: &HistoryGranularity,
        select_granularity: &SelectGranularity,
    ) -> Result<Vec<String>, FztError> {
        Ok(match self.config.mode {
            super::RunnerMode::All => test_provider.all(select_granularity),
            super::RunnerMode::Last => test_provider.runtime_arguments(
                select_granularity,
                self.history_provider.last(history_granularity)?.as_slice(),
            ),
            super::RunnerMode::History => test_provider.runtime_arguments(
                select_granularity,
                self.history_provider
                    .history(history_granularity, &self.search_engine, query)?
                    .as_slice(),
            ),
            super::RunnerMode::Select => {
                let selected_items = self.select_tests(select_granularity, test_provider, query)?;
                self.history_provider
                    .update_history(history_granularity, selected_items.as_slice())?;
                test_provider.runtime_arguments(select_granularity, selected_items.as_slice())
            }
        })
    }

    fn select_append(
        &mut self,
        query: &Option<String>,
        test_provider: &TestProvider,
    ) -> Result<Vec<String>, FztError> {
        Ok(match self.config.mode {
            super::RunnerMode::All => test_provider.all(&SelectGranularity::RunTime),
            super::RunnerMode::Last => {
                parse_append_history(self.history_provider.last(&HistoryGranularity::Append)?)
                    .iter()
                    .flat_map(|(select, selected_items)| {
                        test_provider.runtime_arguments(select, selected_items.as_slice())
                    })
                    .collect()
            }
            super::RunnerMode::History => {
                let history = self.history_provider.history(
                    &HistoryGranularity::Append,
                    &self.search_engine,
                    query,
                )?;
                parse_append_history(history)
                    .iter()
                    .flat_map(|(select, selected_items)| {
                        test_provider.runtime_arguments(select, selected_items.as_slice())
                    })
                    .collect()
            }
            super::RunnerMode::Select => {
                let mut selection = HashMap::new();
                loop {
                    let append = self
                        .search_engine
                        .appened(append_selection_to_preview(&selection).as_str())?;
                    if append == Append::Done {
                        break;
                    }
                    let select_granularity = SelectGranularity::from(append);
                    let mut selected_items =
                        self.select_tests(&select_granularity, test_provider, query)?;
                    selection
                        .entry(select_granularity)
                        .or_insert(vec![])
                        .append(&mut selected_items);
                }

                let history_update: Vec<String> = selection
                    .iter()
                    .flat_map(|(select, selected_items)| {
                        selected_items
                            .iter()
                            .map(|test| format!("{:<20} {}", select, test))
                            .collect::<Vec<String>>()
                    })
                    .collect();
                self.history_provider
                    .update_history(&HistoryGranularity::Append, history_update.as_slice())?;

                selection
                    .iter()
                    .flat_map(|(select, selected_items)| {
                        test_provider.runtime_arguments(select, selected_items.as_slice())
                    })
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

        let test_provider = if self.config.run_failed {
            TestProvider::new_failed(&self.tests)
        } else {
            TestProvider::new(&self.tests)
        };

        let tests_to_run: Vec<String> = match self.config.filter_mode {
            super::FilterMode::Test => self.get_tests_to_run(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::Test,
                &SelectGranularity::Test,
            )?,
            super::FilterMode::File => self.get_tests_to_run(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::File,
                &SelectGranularity::File,
            )?,
            super::FilterMode::Directory => self.get_tests_to_run(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::Directory,
                &SelectGranularity::Directory,
            )?,
            super::FilterMode::RunTime => self.get_tests_to_run(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::RunTime,
                &SelectGranularity::RunTime,
            )?,
            super::FilterMode::Append => {
                self.select_append(&self.config.query.clone(), &test_provider)?
            }
        };
        drop(test_provider);
        if !tests_to_run.is_empty() {
            if let Some(output) = self.runtime.run_tests(
                tests_to_run,
                self.config.verbose,
                &self.config.runtime_args.as_slice(),
                &self.config.debugger,
            )? {
                // We don't want to update the cache if we are running failed tests only
                if !self.config.run_failed && self.tests.update_failed(output.as_str()) {
                    self.cache_manager
                        .add_entry(self.tests.to_json()?.as_str())?;
                }
            }
            Ok(())
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
