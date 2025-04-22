use sha2::{Digest, Sha256};

use crate::{
    cache::manager::CacheManager,
    errors::FztError,
    parser::{
        Tests,
        python::{python_tests::PythonTests, rust_python::RustPytonParser},
    },
    runner::Runner,
    runtime::Runtime,
    search_engine::SearchEngine,
};

pub struct RustPytonRunner<SE: SearchEngine, RT: Runtime> {
    parser: RustPytonParser,
    cache_manager: CacheManager,
    search_engine: SE,
    runtime: RT,
    root_dir: String,
}

impl<SE: SearchEngine, RT: Runtime> RustPytonRunner<SE, RT> {
    pub fn new(root_dir: String, search_engine: SE, runtime: RT) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(root_dir.as_bytes());
        let result = hasher.finalize();
        let project_id = format!("{:x}-rust-python", result);

        let parser = RustPytonParser::default();
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

impl<SE: SearchEngine, RT: Runtime> Runner for RustPytonRunner<SE, RT> {
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
        let selected_tests = self.search_engine.get_tests_to_run(tests)?;
        self.runtime.run_tests(selected_tests)
    }

    fn clear_cache(&self) -> Result<(), FztError> {
        self.cache_manager.clear_cache()
    }
}
