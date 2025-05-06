use crate::{
    cache::{helper::project_hash, manager::CacheManager},
    errors::FztError,
    parser::{
        Tests,
        python::{pytest::PyTestParser, python_tests::PythonTests},
    },
    runner::Runner,
    runtime::Runtime,
    search_engine::SearchEngine,
};

use super::history::get_tests;

pub struct PytestRunner<SE: SearchEngine, RT: Runtime> {
    parser: PyTestParser,
    cache_manager: CacheManager,
    search_engine: SE,
    runtime: RT,

    root_dir: String,
}

impl<SE: SearchEngine, RT: Runtime> PytestRunner<SE, RT> {
    pub fn new(root_dir: String, search_engine: SE, runtime: RT) -> Self {
        let project_id = format!("{}-pytest", project_hash(root_dir.clone()));

        let parser = PyTestParser::new(root_dir.clone());
        let cache_manager = CacheManager::new(project_id);

        Self {
            parser,
            cache_manager,
            search_engine,
            runtime,
            root_dir,
        }
    }
}

impl<SE: SearchEngine, RT: Runtime> Runner for PytestRunner<SE, RT> {
    fn run(&self, history: bool, last: bool) -> Result<(), FztError> {
        let tests = match self.cache_manager.get_entry()? {
            Some(reader) => {
                let mut tests: PythonTests = serde_json::from_reader(reader)?;
                if self.parser.parse_tests(&mut tests)? {
                    self.cache_manager.add_entry(tests.to_json()?.as_str())?
                }
                tests
            }
            None => {
                let mut tests = PythonTests::new_empty(self.root_dir.clone());
                self.parser.parse_tests(&mut tests)?;
                self.cache_manager.add_entry(tests.to_json()?.as_str())?;
                tests
            }
        };
        let selected_tests = get_tests(
            history,
            last,
            &self.cache_manager,
            &self.search_engine,
            tests,
        )?;
        if !selected_tests.is_empty() {
            self.runtime.run_tests(selected_tests)
        } else {
            Ok(())
        }
    }

    fn clear_cache(&self) -> Result<(), FztError> {
        self.cache_manager.clear_cache()
    }
}
