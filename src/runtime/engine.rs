use crate::runtime::utils::partition_tests;
use crate::utils::process::{DefaultFormatter, FailedTest, OutputFormatter, run_and_capture_print};
use crate::{FztError, utils::process::CaptureOutput};
use crossbeam_channel::{Receiver as CrossbeamReceiver, unbounded};
use std::sync::mpsc::Receiver as StdReceiver;
use std::{collections::HashMap, process::Command};

use super::RuntimeOutput;

const NUMBER_THREADS: usize = 16;

pub struct EngineOutput<F: OutputFormatter + Clone + Sync + Send + Default> {
    test_outputs: Vec<TestOutput<F>>,
}

impl<F: OutputFormatter + Clone + Sync + Send + Default> EngineOutput<F> {
    pub fn new(test_outputs: Vec<TestOutput<F>>) -> Self {
        Self { test_outputs }
    }

    pub fn success(&self, test_failure_exit_code: i32) -> bool {
        !self.test_outputs.iter().any(|test_output| {
            test_output.output.status.is_some_and(|status| {
                !status.success()
                    && status
                        .code()
                        .is_some_and(|code| code != test_failure_exit_code)
            })
        })
    }

    pub fn get_error_status_test_output(&self, test_failure_exit_code: i32) -> Vec<CaptureOutput> {
        self.test_outputs
            .iter()
            .filter(|test_output| {
                test_output.output.status.is_some_and(|status| {
                    !status.success()
                        && status
                            .code()
                            .is_some_and(|code| code != test_failure_exit_code)
                })
            })
            .map(|test_output| test_output.output.clone())
            .collect()
    }

    pub fn stopped(&self) -> bool {
        self.test_outputs
            .iter()
            .any(|test_output| test_output.output.stopped)
    }

    pub fn failed_tests(&self) -> Vec<FailedTest> {
        self.test_outputs
            .iter()
            .flat_map(|test_output| test_output.formatter.failed_tests())
            .collect()
    }

    pub fn coverage(&self) -> HashMap<String, Vec<String>> {
        let mut coverage: HashMap<String, Vec<String>> = HashMap::new();
        self.test_outputs
            .iter()
            .map(|test_output| (test_output.test.as_str(), test_output.formatter.coverage()))
            .for_each(|(test, coverred_tests)| {
                coverred_tests.iter().for_each(|path| {
                    coverage
                        .entry(path.to_string())
                        .and_modify(|tests| tests.push(String::from(test)))
                        .or_insert(vec![String::from(test)]);
                });
            });
        coverage
    }

    pub fn merge_formatters(&self) -> F {
        let mut final_formatter = F::default();
        self.test_outputs
            .iter()
            .map(|test_output| &test_output.formatter)
            .for_each(|formatter| {
                final_formatter.add(formatter);
            });
        final_formatter
    }

    pub fn merge_stdout(&self) -> String {
        let mut merged_stdout = String::new();
        for output in self.test_outputs.iter() {
            merged_stdout.push_str(&output.output.stdout);
            merged_stdout.push_str("\n");
        }
        merged_stdout
    }

    pub fn get_test_outputs(&self) -> &[TestOutput<F>] {
        &self.test_outputs.as_slice()
    }
}

pub struct TestOutput<F: OutputFormatter + Clone + Sync + Send> {
    pub output: CaptureOutput,
    pub test: String,
    pub formatter: F,
}

#[derive(Debug, Clone)]
pub struct TestItem<F: OutputFormatter + Clone + Sync + Send> {
    pub test_name: String,
    pub formatter: F,
    pub additional_base_args: Vec<String>,
    pub additional_runtime_args: Vec<String>,
}

pub struct Engine {
    base_command_args: Vec<String>,
    runtime_command_args: Vec<String>,
    runtime_command_args_separator: String,
    number_threads: usize,
    command_envs: HashMap<String, String>,
}

impl Engine {
    pub fn new(runtime_command_args_separator: &str, number_threads: Option<usize>) -> Self {
        let number_threads = if let Some(number_threads) = number_threads {
            number_threads
        } else {
            std::env::var("FZT_NUMBER_THREADS")
                .ok()
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(NUMBER_THREADS)
        };
        Self {
            base_command_args: vec![],
            runtime_command_args: vec![],
            runtime_command_args_separator: runtime_command_args_separator.to_string(),
            number_threads,
            command_envs: HashMap::new(),
        }
    }

    pub fn base_args(&mut self, args: &[&str]) -> &mut Self {
        self.base_command_args
            .extend(args.iter().map(|s| s.to_string()));
        self
    }

    pub fn base_arg(&mut self, arg: &str) -> &mut Self {
        self.base_command_args.push(arg.to_string());
        self
    }

    pub fn base_args_string(&mut self, args: &[String]) -> &mut Self {
        self.base_command_args.extend(args.iter().cloned());
        self
    }

    pub fn runtime_args(&mut self, args: &[String]) -> &mut Self {
        self.runtime_command_args.extend(args.iter().cloned());
        self
    }

    pub fn envs(&mut self, pairs: &HashMap<&str, &str>) -> &mut Self {
        self.command_envs
            .extend(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        self
    }

    fn construct_command(&self, addional_args: &[String]) -> Command {
        let mut command = Command::new(&self.base_command_args[0]);
        if self.base_command_args.len() > 1 {
            command.args(&self.base_command_args[1..]);
        }
        command.args(addional_args);
        command
    }

    fn append_runtime_args(&self, command: &mut Command, addional_args: &[String]) {
        if !self.runtime_command_args.is_empty() {
            command.arg(self.runtime_command_args_separator.as_str());
            command.args(&self.runtime_command_args[..]);
            command.args(addional_args);
        }
    }

    fn run_tests_single<F: OutputFormatter + Clone + Sync + Send>(
        &self,
        test_items: Vec<TestItem<F>>,
        receiver: CrossbeamReceiver<String>,
        verbose: bool,
    ) -> Result<Vec<TestOutput<F>>, FztError> {
        let mut output = vec![];
        for mut item in test_items.into_iter() {
            let mut command = self.construct_command(&item.additional_base_args.as_slice());
            command.arg(item.test_name.clone());
            self.append_runtime_args(&mut command, &item.additional_runtime_args.as_slice());

            if verbose {
                let program = command.get_program().to_str().unwrap();
                let args: Vec<String> = command
                    .get_args()
                    .map(|arg| arg.to_str().unwrap().to_string())
                    .collect();
                println!("\n{} {}\n", program, args.as_slice().join(" "));
            } else {
                let captured =
                    run_and_capture_print(command, &mut item.formatter, Some(receiver.clone()))?;
                output.push(TestOutput {
                    output: captured,
                    test: item.test_name,
                    formatter: item.formatter,
                });
            }
        }
        Ok(output)
    }

    pub fn execute_single_batch_sequential<F: OutputFormatter + Clone + Sync + Send>(
        &self,
        debug_mode: bool,
        receiver: Option<StdReceiver<String>>,
        tests: Vec<String>,
        formatter: &mut F,
        verbose: bool,
    ) -> Result<RuntimeOutput, FztError> {
        let mut command = self.construct_command(&[]);
        tests.iter().for_each(|tests| {
            command.arg(tests);
        });
        self.append_runtime_args(&mut command, &[]);
        if verbose {
            let program = command.get_program().to_str().unwrap();
            let args: Vec<String> = command
                .get_args()
                .map(|arg| arg.to_str().unwrap().to_string())
                .collect();
            println!("\n{} {}\n", program, args.as_slice().join(" "));
        }

        if debug_mode {
            command.status()?;
            Ok(RuntimeOutput::new_empty())
        } else {
            let output = run_and_capture_print(command, formatter, receiver)?;
            if output.stopped {
                Ok(RuntimeOutput::new_empty())
            } else {
                Ok(RuntimeOutput {
                    failed_tests: vec![],
                    output: Some(output.stdout),
                    coverage: HashMap::new(),
                })
            }
        }
    }

    // TODO: Return Vec
    pub fn execute_per_item_parallel<F: OutputFormatter + Clone + Sync + Send + Default>(
        &self,
        receiver: Option<StdReceiver<String>>,
        test_items: Vec<TestItem<F>>,
        verbose: bool,
    ) -> Result<EngineOutput<F>, FztError> {
        println!("\nRunning {} tests", test_items.len());

        let partitions = partition_tests(test_items, self.number_threads);
        let mut local_outputs: Vec<Result<Vec<TestOutput<F>>, FztError>> =
            (0..partitions.len()).map(|_| Ok(vec![])).collect();

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
            for (output, partition) in local_outputs.iter_mut().zip(partitions.into_iter()) {
                s.spawn(|| {
                    *output = self.run_tests_single(partition, cross_rx.clone(), verbose);
                });
            }
        });

        let final_output: Vec<TestOutput<F>> = local_outputs
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(EngineOutput::new(final_output))
    }
}
