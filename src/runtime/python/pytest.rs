use std::{io::{BufRead, BufReader}, process::{Command, Stdio}};

use crate::{
    errors::FztError,
    runtime::{utils::run_and_capture, Debugger, PythonDebugger, Runtime},
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
        let mut command = Command::new("python");
        command.arg("-m");
        command.arg("pytest");
        command.arg("--color");
        command.arg("yes");
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
            println!("INFO: Verbose mode enabled, does not capture failed tests");
            let program = command.get_program().to_str().unwrap();
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
            command.status()?;
            Ok(String::new())
        } else {
            let output = run_and_capture(command)?;
            Ok(output)
        }
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
