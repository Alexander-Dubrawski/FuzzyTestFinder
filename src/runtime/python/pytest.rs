use std::{collections::HashMap, sync::mpsc::Receiver};

use itertools::Itertools;

use crate::{
    errors::FztError,
    runtime::{
        Debugger, PythonDebugger, Runtime, RuntimeOutput, engine::Engine,
        python::formatter::PytestFormatter,
    },
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
        run_coverage: bool,
    ) -> Result<RuntimeOutput, FztError> {
        let base_args = if debugger.is_some() || runtime_ags.contains(&String::from("--pdb")) {
            vec!["python", "-m", "pytest", "-s"]
        } else {
            vec!["unbuffer", "python", "-m", "pytest"]
        };

        let ordered_tests: Vec<String> = tests
            .iter()
            .sorted_by_key(|name| {
                let file = name
                    .splitn(2, "::")
                    .next()
                    .expect(format!("{name} is an invalid test name").as_str());
                file
            })
            .map(|file| file.to_string())
            .collect();

        let mut envs = HashMap::new();

        if let Some(debugger_selection) = debugger {
            match debugger_selection {
                Debugger::Python(PythonDebugger::Pdb) => {
                    envs.insert("PYTHONBREAKPOINT", "pdb.set_trace");
                }
                Debugger::Python(PythonDebugger::Ipdb) => {
                    envs.insert("PYTHONBREAKPOINT", "ipdb.set_trace");
                }
                Debugger::Python(PythonDebugger::IPython) => {
                    envs.insert("PYTHONBREAKPOINT", "IPython.terminal.debugger.set_trace");
                }
                Debugger::Python(PythonDebugger::Pudb) => {
                    envs.insert("PYTHONBREAKPOINT", "pudb.set_trace");
                }
                Debugger::Python(PythonDebugger::WebPdb) => {
                    println!("web-pdb, visit http://localhost:5555 to debug");
                    envs.insert("PYTHONBREAKPOINT", "web_pdb.set_trace");
                }
                _ => {
                    unreachable!(
                        "Non-Python debugger passed to PytestRuntime. This should be unreachable due to CLI validation."
                    );
                }
            }
        } else {
            envs.insert("PYTHONBREAKPOINT", "0");
        }

        if run_coverage {
            if debugger.is_some() || runtime_ags.contains(&String::from("--pdb")) {
                return Err(FztError::UserError(
                    "Coverage cannot be run with a debugger attached.".to_string(),
                ));
            }
            let mut engine =
                Engine::new("", PytestFormatter::new(), None, PYTEST_FAILURE_EXIT_CODE);
            engine.base_args(base_args.as_slice());
            engine.tests(ordered_tests.as_slice());
            engine.runtime_args(runtime_ags);
            engine.runtime_args(&[
                "--cov=myapp".to_string(),
                "--cov-report=term-missing:skip-covered".to_string(),
            ]);
            engine.execute_per_item_parallel(true, receiver, verbose)
        } else {
            let mut engine = Engine::new("--", DefaultFormatter, None, PYTEST_FAILURE_EXIT_CODE);
            engine.base_args(base_args.as_slice());
            engine.tests(ordered_tests.as_slice());
            engine.runtime_args(runtime_ags);
            engine.envs(&envs);
            engine.execute_single_batch_sequential(
                debugger.is_some() || runtime_ags.contains(&String::from("--pdb")),
                receiver,
                verbose,
            )
        }
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
