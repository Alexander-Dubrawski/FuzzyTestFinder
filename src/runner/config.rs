use serde::{Deserialize, Serialize};

use crate::{
    cache::{helper::project_hash, manager::CacheManager},
    errors::FztError,
    runtime::Debugger,
    search_engine::SearchEngine,
};

use super::{Runner, java::get_java_runner, python::get_python_runner, rust::get_rust_runner};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RunnerMode {
    All,
    Last,
    History,
    Select,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Preview {
    File,
    Test,
    Directory,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FilterMode {
    Test,
    File,
    Directory,
    RunTime,
    Append,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Language {
    Python {
        parser: String,
        runtime: String,
    },
    Java {
        test_framework: String,
        runtime: String,
    },
    Rust,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunnerConfig<SE: SearchEngine + 'static> {
    pub clear_cache: bool,
    pub verbose: bool,
    pub clear_history: bool,
    pub runtime_args: Vec<String>,
    pub mode: RunnerMode,
    pub preview: Option<Preview>,
    pub filter_mode: FilterMode,
    pub query: Option<String>,
    pub debugger: Option<Debugger>,
    pub run_failed: bool,
    pub language: Language,
    pub search_engine: SE,
}

impl<SE: SearchEngine> RunnerConfig<SE> {
    pub fn new(
        clear_cache: bool,
        verbose: bool,
        clear_history: bool,
        runtime_args: Vec<String>,
        mode: RunnerMode,
        preview: Option<Preview>,
        filter_mode: FilterMode,
        query: Option<String>,
        debugger: Option<Debugger>,
        run_failed: bool,
        language: Language,
        search_engine: SE,
    ) -> Self {
        Self {
            clear_cache,
            verbose,
            clear_history,
            runtime_args,
            mode,
            preview,
            filter_mode,
            query,
            debugger,
            run_failed,
            language,
            search_engine,
        }
    }

    fn build_cache_manager(&self, project_id: &str) -> CacheManager {
        if self.run_failed {
            CacheManager::new_failed_tests(project_id)
        } else {
            CacheManager::new(project_id)
        }
    }

    pub fn into_runner(self) -> Result<Box<dyn Runner>, FztError> {
        let project_hash = project_hash()?;
        match self.language.clone() {
            Language::Python { parser, runtime } => {
                let project_id = if parser == "rustpython" {
                    format!("{}-rust-python", project_hash)
                } else {
                    format!("{}-pytest", project_hash)
                };
                let cache_manager = self.build_cache_manager(project_id.as_str());
                get_python_runner(parser.as_str(), runtime.as_str(), self, cache_manager)
            }
            Language::Java {
                test_framework,
                runtime,
            } => {
                let cache_manager =
                    self.build_cache_manager(format!("{}-java-junit5", project_hash).as_str());
                get_java_runner(
                    test_framework.as_str(),
                    runtime.as_str(),
                    self,
                    cache_manager,
                )
            }
            Language::Rust => {
                let cache_manager =
                    self.build_cache_manager(format!("{}-rust-cargo", project_hash).as_str());
                get_rust_runner(self, cache_manager)
            }
        }
    }
}
