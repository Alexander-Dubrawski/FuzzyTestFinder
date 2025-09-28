use std::{
    process::Command,
    sync::mpsc::{Receiver, Sender},
};

use itertools::Itertools;

use crate::{
    errors::FztError,
    runtime::{Debugger, DefaultFormatter, PythonDebugger, Runtime, utils::run_and_capture_print},
};

#[derive(Default)]
pub struct PytestRuntime {}

fn build_command(tests: &[String], runtime_ags: &[String], debugger: &Option<Debugger>) -> Command {
    let mut command = if debugger.is_some() || runtime_ags.contains(&String::from("--pdb")) {
        Command::new("python")
    } else {
        let mut command = Command::new("unbuffer");
        command.arg("python");
        command
    };
    command.arg("-m");
    command.arg("pytest");
    if debugger.is_some() {
        command.arg("-s");
    }
    runtime_ags.iter().for_each(|arg| {
        command.arg(arg);
    });
    tests
        .iter()
        .sorted_by_key(|name| {
            let file = name
                .splitn(2, "::")
                .next()
                .expect(format!("{name} is an invalid test name").as_str());
            file.to_string()
        })
        .for_each(|test| {
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
    command
}

impl Runtime for PytestRuntime {
    fn run_tests(
        &self,
        tests: Vec<String>,
        verbose: bool,
        runtime_ags: &[String],
        debugger: &Option<Debugger>,
        channels: Option<(Sender<String>, Receiver<String>)>,
    ) -> Result<Option<String>, FztError> {
        let command = build_command(tests.as_slice(), runtime_ags, &None);
        let mut debug_command = build_command(tests.as_slice(), runtime_ags, debugger);

        if verbose {
            let program = debug_command.get_program().to_str().unwrap();
            let args: Vec<String> = debug_command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
        }
        if debugger.is_some() || runtime_ags.contains(&String::from("--pdb")) {
            debug_command.status()?;
            Ok(None)
        } else {
            let reciver = if let Some((_, reciver)) = channels {
                Some(reciver)
            } else {
                None
            };
            let output = run_and_capture_print(command, &mut DefaultFormatter, reciver)?;
            if output.stopped {
                Ok(None)
            } else {
                Ok(Some(output.message))
            }
        }
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
