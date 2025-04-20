use std::collections::HashMap;

use sha2::{Digest, Sha256};

use crate::{
    cache::manager::CacheManager,
    errors::FztError,
    parser::{
        Tests,
        python::{pytest::PyTestParser, python_tests::PythonTests},
    },
    runner::Runner,
    runtime::python::pytest::PytestRuntime,
    search_engine::fzf::FzfSearchEngine,
};

pub struct PytestRunner {
    parser: PyTestParser,
    cache_manager: CacheManager,
    search_engine: FzfSearchEngine,
    runtime: PytestRuntime,
    root_dir: String,
}

impl PytestRunner {
    pub fn new(root_dir: String) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(root_dir.as_bytes());
        let result = hasher.finalize();
        let project_id = format!("{:x}-pytest", result);

        let parser = PyTestParser::new(root_dir.clone());
        let cache_manager = CacheManager::new(project_id);
        let search_engine = FzfSearchEngine::default();
        let runtime = PytestRuntime::default();

        Self {
            parser,
            cache_manager,
            search_engine,
            runtime,
            root_dir,
        }
    }
}

impl Runner for PytestRunner {
    fn run(&self) -> Result<(), FztError> {
        let tests = match self.cache_manager.get_entry()? {
            Some(reader) => {
                let mut tests: PythonTests = serde_json::from_reader(reader).unwrap();
                if self.parser.parse_tests(&mut tests)? {
                    self.cache_manager.add_entry(tests.to_json().as_str())?
                }
                tests
            }
            None => {
                let mut tests = PythonTests::new(self.root_dir.clone(), 0, HashMap::new());
                self.parser.parse_tests(&mut tests)?;
                self.cache_manager.add_entry(tests.to_json().as_str())?;
                tests
            }
        };
        let selected_tests = self.search_engine.get_tests_to_run(tests)?;
        self.runtime.run_tests(selected_tests)
    }
}
