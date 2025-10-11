use crate::runtime::utils::partition_tests;
use crate::utils::process::{DefaultFormatter, Formatter, run_and_capture_print};
use crate::{FztError, utils::process::CaptureOutput};
use crossbeam_channel::{Receiver as CrossbeamReceiver, unbounded};
use std::sync::mpsc::Receiver as StdReceiver;
use std::{collections::HashMap, process::Command};

use super::{Debugger, RuntimeFormatter};

const NUMBER_THREADS: usize = 16;

struct Output {
    pub output: CaptureOutput,
    pub test: String,
    pub covered: Vec<String>,
}

pub struct Engine<F: RuntimeFormatter + Formatter + Clone + Sync + Send> {
    base_command_args: Vec<String>,
    runtime_command_args: Vec<String>,
    runtime_command_args_seperator: String,
    formatter: F,
    tests: Vec<String>,
    number_threads: usize,
}

impl<F: RuntimeFormatter + Clone + Formatter + Clone + Sync + Send> Engine<F> {
    pub fn new(
        base_command_args: Vec<String>,
        runtime_command_args: Vec<String>,
        runtime_command_args_seperator: &str,
        tests: Vec<String>,
        formatter: F,
        number_threads: Option<usize>,
    ) -> Self {
        let number_threads = if let Some(number_threads) = number_threads {
            number_threads
        } else {
            std::env::var("FZT_NUMBER_THREADS")
                .ok()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(NUMBER_THREADS)
        };
        Self {
            base_command_args,
            runtime_command_args,
            runtime_command_args_seperator: runtime_command_args_seperator.to_string(),
            tests,
            formatter,
            number_threads,
        }
    }

    fn construct_command(&self) -> Command {
        let mut command = Command::new(&self.base_command_args[0]);
        if self.base_command_args.len() > 1 {
            command.args(&self.base_command_args[1..]);
        }
        command
    }

    fn append_runtime_args(&self, command: &mut Command) {
        if self.runtime_command_args.len() > 0 {
            command.arg(self.runtime_command_args_seperator.as_str());
            command.args(&self.runtime_command_args[..]);
        }
    }

    fn run_tests_single(
        &self,
        tests: &[String],
        formatter: &mut F,
        receiver: CrossbeamReceiver<String>,
        verbose: bool,
    ) -> Result<Vec<Output>, FztError> {
        let mut output = vec![];
        for test in tests {
            let mut command = self.construct_command();
            command.arg(test);
            self.append_runtime_args(&mut command);

            if verbose {
                let program = command.get_program().to_str().unwrap();
                let args: Vec<String> = command
                    .get_args()
                    .map(|arg| arg.to_str().unwrap().to_string())
                    .collect();
                println!("\n{} {}\n", program, args.as_slice().join(" "));
                let captured =
                    run_and_capture_print(command, &mut DefaultFormatter, Some(receiver.clone()))?;
                output.push(Output {
                    output: captured,
                    test: test.clone(),
                    covered: vec![],
                });
            } else {
                let captured = run_and_capture_print(command, formatter, Some(receiver.clone()))?;
                let covered = formatter.coverage();
                formatter.reset_coverage();
                output.push(Output {
                    output: captured,
                    test: test.clone(),
                    covered,
                });
            }
        }
        Ok(output)
    }

    pub fn execute_single_batch(
        &self,
        debugger: &Option<Debugger>,
        receiver: Option<StdReceiver<String>>,
        verbose: bool,
    ) -> Result<Option<String>, FztError> {
        let mut command = self.construct_command();
        self.tests.iter().for_each(|test| {
            command.arg(test);
        });
        self.append_runtime_args(&mut command);
        if verbose {
            let program = command.get_program().to_str().unwrap();
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
        }

        // TODO: Check
        // || runtime_ags.contains(&String::from("--pdb"))
        if debugger.is_some() {
            command.status()?;
            Ok(None)
        } else {
            let output = run_and_capture_print(command, &mut DefaultFormatter, receiver)?;
            if output.stopped {
                Ok(None)
            } else {
                Ok(Some(output.stdout))
            }
        }
    }

    pub fn execute_per_item(
        &self,
        coverage: &mut Option<HashMap<String, Vec<String>>>,
        receiver: Option<StdReceiver<String>>,
        verbose: bool,
    ) -> Result<Option<String>, FztError> {
        let partitions = partition_tests(&self.tests, self.number_threads);
        let mut formatters = vec![self.formatter.clone(); partitions.len()];
        let mut outputs: Vec<Result<Vec<Output>, FztError>> =
            (0..partitions.len()).map(|_| Ok(vec![])).collect();

        println!("\nRunning {} tests", self.tests.len());

        let (cross_tx, cross_rx) = unbounded();
        if let Some(rx) = receiver {
            // Bridge thread: listen on std receiver, broadcast on crossbeam
            std::thread::spawn({
                let cross_tx = cross_tx.clone();
                move || {
                    if let Ok(msg) = rx.recv() {
                        let _ = cross_tx.send(msg); // Broadcast to all worker threads
                    }
                }
            });
        }
        std::thread::scope(|s| {
            for ((formatter, output), partition) in formatters
                .iter_mut()
                .zip(outputs.iter_mut())
                .zip(partitions.iter())
            {
                s.spawn(|| {
                    *output = self.run_tests_single(
                        partition.as_slice(),
                        formatter,
                        cross_rx.clone(),
                        verbose,
                    );
                });
            }
        });

        let mut final_formatter = self.formatter.clone();
        let mut final_output = String::new();

        for (formatter, output_result) in formatters.into_iter().zip(outputs.into_iter()) {
            let outputs = output_result?;
            if outputs
                .iter()
                .any(|capture_output| capture_output.output.stopped)
            {
                return Ok(None);
            }
            final_formatter.add(formatter);
            final_output.push_str("\n");
            outputs.iter().for_each(|capture_output| {
                if let Some(cov) = coverage {
                    capture_output.covered.iter().for_each(|path| {
                        cov.entry(path.to_string())
                            .and_modify(|tests| tests.push(capture_output.test.clone()))
                            .or_insert(vec![capture_output.test.clone()]);
                    });
                }
                final_output.push_str(&capture_output.output.stdout.as_str());
            });
        }
        if !verbose {
            final_formatter.finish();
        }
        Ok(Some(final_output))
    }
}
