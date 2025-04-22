use crate::{
    cache::manager::CacheManager, errors::FztError, parser::python::python_tests::PythonTests,
    search_engine::SearchEngine,
};

pub fn get_tests<SE: SearchEngine>(
    history: bool,
    last: bool,
    cache_manager: &CacheManager,
    search_engine: &SE,
    tests: PythonTests,
) -> Result<Vec<String>, FztError> {
    if last {
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
