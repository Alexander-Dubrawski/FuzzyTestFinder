use std::{collections::HashMap, sync::mpsc::Receiver};

use itertools::Itertools;

use crate::{
    errors::FztError,
    runtime::{Debugger, PythonDebugger, Runtime, engine::Engine},
    utils::process::DefaultFormatter,
};

const PYTEST_FAILURE_EXIT_CODE: i32 = 1;

#[derive(Default)]
pub struct PytestRuntime {}

impl Runtime for PytestRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
        receiver: Option<Receiver<String>>,
        _coverage: &mut Option<HashMap<String, Vec<String>>>,
    ) -> Result<Option<String>, FztError> {
        let mut engine = Engine::new("--", DefaultFormatter, None, PYTEST_FAILURE_EXIT_CODE);
        if debugger.is_some() || runtime_ags.contains(&String::from("--pdb")) {
            engine.base_args(&["python", "-m", "pytest", "-s"]);
        } else {
            // unbuffer merges stdout and stderr
            engine.base_args(&["unbuffer", "python", "-m", "pytest"]);
        }
        engine.runtime_args(runtime_ags);

        tests
            .iter()
            .sorted_by_key(|name| {
                let file = name
                    .splitn(2, "::")
                    .next()
                    .expect(format!("{name} is an invalid test name").as_str());
                file
            })
            .for_each(|test| {
                engine.test(test);
            });

        if let Some(debugger_selection) = debugger {
            match debugger_selection {
                Debugger::Python(PythonDebugger::Pdb) => {
                    engine.env("PYTHONBREAKPOINT", "pdb.set_trace");
                }
                Debugger::Python(PythonDebugger::Ipdb) => {
                    engine.env("PYTHONBREAKPOINT", "ipdb.set_trace");
                }
                Debugger::Python(PythonDebugger::IPython) => {
                    engine.env("PYTHONBREAKPOINT", "IPython.terminal.debugger.set_trace");
                }
                Debugger::Python(PythonDebugger::Pudb) => {
                    engine.env("PYTHONBREAKPOINT", "pudb.set_trace");
                }
                Debugger::Python(PythonDebugger::WebPdb) => {
                    println!("web-pdb, visit http://localhost:5555 to debug");
                    engine.env("PYTHONBREAKPOINT", "web_pdb.set_trace");
                }
                _ => {
                    unreachable!(
                        "Non-Python debugger passed to PytestRuntime. This should be unreachable due to CLI validation."
                    );
                }
            }
        } else {
            engine.env("PYTHONBREAKPOINT", "0");
        }

        engine.execute_single_batch(
            debugger.is_some() || runtime_ags.contains(&String::from("pdb")),
            receiver,
            verbose,
        )
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
