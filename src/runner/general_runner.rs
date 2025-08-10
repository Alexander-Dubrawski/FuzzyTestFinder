use std::{collections::HashMap, str::FromStr};

use serde::de::DeserializeOwned;

use crate::{
    cache::manager::{CacheManager, HistoryGranularity},
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    runtime::Runtime,
    search_engine::{Appened, SearchEngine},
    tests::{
        Tests,
        test_provider::{Select, TestProvider},
    },
};

use super::{Preview, history_provider::HistoryProvider};

fn append_selection_to_preview(selection: &HashMap<Select, Vec<String>>) -> String {
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
        let cache_manager = CacheManager::new(project_id);
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

    fn select(
        &mut self,
        select: &Select,
        test_provider: &TestProvider,
        query: &Option<String>,
    ) -> Result<Vec<String>, FztError> {
        let preview = if select == &Select::Directory {
            if self.config.preview.is_some() {
                Some(Preview::Directory)
            } else {
                None
            }
        } else if select == &Select::RunTime {
            None
        } else {
            self.config.preview.clone()
        };
        Ok(self.search_engine.get_tests_to_run(
            test_provider.select_option(select).as_slice(),
            &preview,
            query,
        )?)
    }

    fn select_tests(
        &mut self,
        query: &Option<String>,
        test_provider: &TestProvider,
        granularity: &HistoryGranularity,
        select: &Select,
    ) -> Result<Vec<String>, FztError> {
        Ok(match self.config.mode {
            super::RunnerMode::All => test_provider.all(select),
            super::RunnerMode::Last => test_provider
                .runtime_arguments(select, self.history_provider.last(granularity)?.as_slice()),
            super::RunnerMode::History => test_provider.runtime_arguments(
                select,
                self.history_provider
                    .history(granularity, &self.search_engine, query)?
                    .as_slice(),
            ),
            super::RunnerMode::Select => {
                let selected_items = self.select(select, test_provider, query)?;
                self.history_provider
                    .update_history(granularity, selected_items.as_slice())?;
                test_provider.runtime_arguments(select, selected_items.as_slice())
            }
        })
    }

    fn select_append(
        &mut self,
        query: &Option<String>,
        test_provider: &TestProvider,
    ) -> Result<Vec<String>, FztError> {
        Ok(match self.config.mode {
            super::RunnerMode::All => test_provider.all(&Select::RunTime),
            super::RunnerMode::Last => {
                let mut selection = HashMap::new();
                let last = self.history_provider.last(&HistoryGranularity::Append)?;
                last.into_iter().for_each(|test| {
                    let mut iter = test.splitn(2, ' ');
                    // TODO: Error handling
                    let select = Select::from_str(iter.next().unwrap()).unwrap();
                    let selected_items =
                        iter.next().unwrap().strip_prefix(' ').unwrap().to_string();
                    selection
                        .entry(select)
                        .or_insert(vec![])
                        .push(selected_items);
                });
                selection
                    .iter()
                    .flat_map(|(select, selected_items)| {
                        test_provider.runtime_arguments(select, selected_items.as_slice())
                    })
                    .collect()
            }
            super::RunnerMode::History => {
                let mut selection = HashMap::new();
                let history = self.history_provider.history(
                    &HistoryGranularity::Append,
                    &self.search_engine,
                    query,
                )?;
                history.into_iter().for_each(|test| {
                    let mut iter = test.splitn(2, ' ');
                    // TODO: Error handling
                    let select = Select::from_str(iter.next().unwrap()).unwrap();
                    let selected_items =
                        iter.next().unwrap().strip_prefix(' ').unwrap().to_string();
                    selection
                        .entry(select)
                        .or_insert(vec![])
                        .push(selected_items);
                });
                selection
                    .iter()
                    .flat_map(|(select, selected_items)| {
                        test_provider.runtime_arguments(select, selected_items.as_slice())
                    })
                    .collect()
            }
            super::RunnerMode::Select => {
                let mut selection = HashMap::new();
                loop {
                    match self
                        .search_engine
                        .appened(append_selection_to_preview(&selection).as_str())?
                    {
                        Appened::Test => {
                            let mut selected_items =
                                self.select(&Select::Test, test_provider, query)?;
                            selection
                                .entry(Select::Test)
                                .or_insert(vec![])
                                .append(&mut selected_items);
                        }
                        Appened::File => {
                            let mut selected_items =
                                self.select(&Select::File, test_provider, query)?;
                            selection
                                .entry(Select::File)
                                .or_insert(vec![])
                                .append(&mut selected_items);
                        }
                        Appened::Directory => {
                            let mut selected_items =
                                self.select(&Select::Directory, test_provider, query)?;
                            selection
                                .entry(Select::Directory)
                                .or_insert(vec![])
                                .append(&mut selected_items);
                        }
                        Appened::RunTime => {
                            let mut selected_items =
                                self.select(&Select::RunTime, test_provider, query)?;
                            selection
                                .entry(Select::RunTime)
                                .or_insert(vec![])
                                .append(&mut selected_items);
                        }
                        Appened::Done => break,
                    }
                }
                for (select, selected_items) in selection.iter() {
                    self.history_provider.update_history(
                        &HistoryGranularity::Append,
                        selected_items
                            .iter()
                            .map(|test| format!("{:<20} {}", select, test))
                            .collect::<Vec<String>>()
                            .as_slice(),
                    )?;
                }
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

        let test_provider = TestProvider::new(&self.tests);

        let tests_to_run: Vec<String> = match self.config.filter_mode {
            super::FilterMode::Test => self.select_tests(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::Test,
                &Select::Test,
            )?,
            super::FilterMode::File => self.select_tests(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::File,
                &Select::File,
            )?,
            super::FilterMode::Directory => self.select_tests(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::Directory,
                &Select::Directory,
            )?,
            super::FilterMode::RunTime => self.select_tests(
                &self.config.query.clone(),
                &test_provider,
                &HistoryGranularity::RunTime,
                &Select::RunTime,
            )?,
            super::FilterMode::Append => {
                self.select_append(&self.config.query.clone(), &test_provider)?
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
