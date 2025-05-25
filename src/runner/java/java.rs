use crate::{
    cache::{helper::project_hash, manager::CacheManager},
    errors::FztError,
    runner::{MetaData, Runner, RunnerConfig, RunnerName},
    runtime::Runtime,
    search_engine::SearchEngine,
    tests::{
        Test, Tests,
        java::{java_test::JavaTests, parser::JavaParser},
    },
};

// TODO: Make generic
pub fn get_tests<SE: SearchEngine>(
    history: bool,
    last: bool,
    cache_manager: &CacheManager,
    search_engine: &SE,
    tests: JavaTests,
    all: bool,
) -> Result<Vec<String>, FztError> {
    if all {
        Ok(tests
            .tests()
            .into_iter()
            .map(|test| test.search_item_name())
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

pub struct JavaJunit5Runner<SE: SearchEngine, RT: Runtime> {
    parser: JavaParser,
    cache_manager: CacheManager,
    search_engine: SE,
    runtime: RT,
    root_dir: String,
    config: RunnerConfig,
}

impl<SE: SearchEngine, RT: Runtime> JavaJunit5Runner<SE, RT> {
    pub fn new(root_dir: String, search_engine: SE, runtime: RT, config: RunnerConfig) -> Self {
        let project_id = format!("{}-java-junit5", project_hash(root_dir.clone()));
        let parser = JavaParser::new(root_dir.clone());
        let cache_manager = CacheManager::new(project_id);

        Self {
            parser,
            cache_manager,
            search_engine,
            runtime,
            root_dir,
            config,
        }
    }
}

impl<SE: SearchEngine, RT: Runtime> Runner for JavaJunit5Runner<SE, RT> {
    fn run(&self) -> Result<(), FztError> {
        if self.config.clear_cache || self.config.clear_history {
            if self.config.clear_cache {
                self.cache_manager.clear_cache()?;
            }
            if self.config.clear_history {
                self.cache_manager.clear_history()?;
            }
            return Ok(());
        }
        let tests = match self.cache_manager.get_entry()? {
            Some(reader) => {
                let mut tests: JavaTests = serde_json::from_reader(reader)?;
                if self.parser.parse_tests(&mut tests, false)? {
                    self.cache_manager.add_entry(tests.to_json()?.as_str())?
                }
                tests
            }
            None => {
                let mut tests = JavaTests::new_empty(self.root_dir.clone());
                self.parser.parse_tests(&mut tests, false)?;
                self.cache_manager.add_entry(tests.to_json()?.as_str())?;
                tests
            }
        };
        let selected_tests = get_tests(
            self.config.history,
            self.config.last,
            &self.cache_manager,
            &self.search_engine,
            // TODO: ref instead of clone
            tests.clone(),
            self.config.all,
        )?;
        // TODO: Improve Filter for tests
        let test_items: Vec<String> = tests
            .tests()
            .into_iter()
            .filter(|test| {
                let search_name = test.search_item_name();
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
            runner_name: RunnerName::JavaJunit5Runner,
            search_engine: self.search_engine.name(),
            runtime: self.runtime.name(),
        };
        let json = serde_json::to_string(&meta_data)?;
        Ok(json)
    }
}
