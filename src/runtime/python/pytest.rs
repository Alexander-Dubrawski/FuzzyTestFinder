use std::{collections::HashMap, sync::mpsc::Receiver};

use itertools::Itertools;
use tempfile::TempDir;

use crate::{
    errors::FztError,
    runtime::{
        Debugger, OutputFormatter, PythonDebugger, Runtime, RuntimeOutput,
        engine::{Engine, TestItem},
    },
};

use super::formatter::{pytest::PytestFormatter, pytest_coverage::PytestCovFormatter};

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
            // Drops TempDir when leaving scope
            let temp_dirs: Vec<(TempDir, TempDir)> = (0..tests.len())
                .map(|_| -> Result<(TempDir, TempDir), std::io::Error> {
                    Ok((tempfile::tempdir()?, tempfile::tempdir()?))
                })
                .collect::<Result<Vec<(TempDir, TempDir)>, std::io::Error>>()?;

            let test_items: Vec<TestItem<PytestCovFormatter>> = tests
                .into_iter()
                .zip(temp_dirs.iter())
                .map(|(test, (cov_dir, rep_dir))| {
                    // We do not want multiple threads use the same coverage file
                    let cov_data_path = cov_dir.path().join("coverage").to_path_buf();
                    let cov_path = cov_dir.path().join("coverage.json").to_path_buf();
                    let rep_path = rep_dir.path().join("report.json").to_path_buf();

                    let additional_base_args = vec![
                        "--json-report".to_string(),
                        format!(
                            "--cov-report=json:{}",
                            cov_path
                                .as_os_str()
                                .to_str()
                                .expect("Failed to convert path to string")
                        ),
                        format!(
                            "--json-report-file={}",
                            rep_path
                                .as_os_str()
                                .to_str()
                                .expect("Failed to convert path to string")
                        ),
                    ];

                    let formatter = PytestCovFormatter::new(cov_path, rep_path, test.as_str());
                    TestItem {
                        test_name: test,
                        formatter,
                        additional_base_args,
                        additional_runtime_args: vec![],
                        additional_command_envs: HashMap::from_iter(
                            vec![(
                                "COVERAGE_FILE".to_string(),
                                cov_data_path
                                    .as_os_str()
                                    .to_str()
                                    .expect("Failed to convert path to string")
                                    .to_string(),
                            )]
                            .into_iter(),
                        ),
                    }
                })
                .collect();
            let mut engine = Engine::new(None, None);
            engine.base_args(base_args.as_slice());
            engine.runtime_args(runtime_ags);
            engine.base_args(&["--cov=myapp", "--cov-report=term-missing:skip-covered"]);
            let engine_output = engine.execute_per_item_parallel(receiver, test_items, verbose)?;

            engine_output.merge_formatters().finish();

            if engine_output.stopped() {
                Ok(RuntimeOutput::new_empty())
            } else {
                Ok(RuntimeOutput::from_engine_output(&engine_output))
            }
        } else {
            let mut engine = Engine::new(None, None);
            engine.base_args(base_args.as_slice());
            engine.runtime_args(runtime_ags);
            engine.envs(&envs);
            engine.execute_single_batch_sequential(
                debugger.is_some() || runtime_ags.contains(&String::from("--pdb")),
                receiver,
                ordered_tests,
                &mut PytestFormatter::new(),
                verbose,
            )
        }
    }

    fn name(&self) -> String {
        String::from("pytest")
    }
}
