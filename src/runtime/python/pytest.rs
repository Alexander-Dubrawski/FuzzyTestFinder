use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crate::{
    errors::FztError,
    runtime::{Debugger, PythonDebugger, Runtime, utils::run_and_capture},
};

#[derive(Default)]
pub struct PytestRuntime {}

impl Runtime for PytestRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
    ) -> Result<String, FztError> {
        let mut command = Command::new("unbuffer");
        command.arg("python");
        command.arg("-m");
        command.arg("pytest");
        if debugger.is_some() {
            command.arg("-s");
        }
        runtime_ags.iter().for_each(|arg| {
            command.arg(arg);
        });
        tests.into_iter().for_each(|test| {
            command.arg(test);
        });

        if let Some(debugger_selection) = debugger {
            match debugger_selection {
                Debugger::Python(PythonDebugger::Pdb) => {
                    command.env("PYTHONBREAKPOINT", "pdb.set_trace");
                }
                Debugger::Python(PythonDebugger::Ipdb) => {
                    command.env("PYTHONBREAKPOINT", "ipdb.set_trace");
                }
                Debugger::Python(PythonDebugger::IPython) => {
                    command.env("PYTHONBREAKPOINT", "IPython.terminal.debugger.set_trace");
                }
                Debugger::Python(PythonDebugger::Pudb) => {
                    command.env("PYTHONBREAKPOINT", "pudb.set_trace");
                }
                Debugger::Python(PythonDebugger::WebPdb) => {
                    println!("web-pdb, visit http://localhost:5555 to debug");
                    command.env("PYTHONBREAKPOINT", "web_pdb.set_trace");
                }
                _ => {
                    unreachable!(
                        "Non-Python debugger passed to PytestRuntime. This should be unreachable due to CLI validation."
                    );
                }
            }
        } else {
            command.env("PYTHONBREAKPOINT", "0");
        }

        if verbose {
            let program = command.get_program().to_str().unwrap();
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
        }
        let output = run_and_capture(command)?;
        Ok(output)
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
