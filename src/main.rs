use std::env;

use FzT::{python::pytest::run, runner::python::pytest::PytestRunner};

// TODO:
// Add support for --query and direct it to fzf
// Add cache function / one cache per project
// Add window preview mode seeing the code
// Add cache clear option
fn main() {
    let pytest = false;
    let path = env::current_dir().unwrap();
    let path_str = path.to_string_lossy();
    if pytest {
        let runner = PytestRunner::new(path_str.to_string());
        runner.run();
    } else {
        let runner = RustPytonRunner::new(path_str.to_string());
        runner.run();
    }
}
