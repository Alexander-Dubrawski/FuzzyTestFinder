use std::process::Command;

use crate::{
    errors::FztError,
    runtime::{Debugger, PythonDebugger, Runtime},
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
    ) -> Result<(), FztError> {
        let mut command = Command::new("python");
        command.arg("-m");
        command.arg("pytest");
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
                _ => {
                    return Err(FztError::InvalidArgument(
                        "Invalid debugger option. Supported are: Python = [pdb, ipdb, IPython]"
                            .to_string(),
                    ));
                }
            }
        }

        if verbose {
            let program = command.get_program().to_str().unwrap();
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
        }
        command.status()?;
        Ok(())
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
