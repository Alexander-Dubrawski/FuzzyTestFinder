use std::collections::HashMap;

use super::rust_test::RustTest;

static FAILED_TEST_PREFIX: usize = "test ".len();
static FAILED_TEST_SUFFIX: usize = " ... FAILED".len();

pub fn parse_failed_tests(
    output: &str,
    current_tests: &HashMap<String, Vec<RustTest>>,
) -> HashMap<String, Vec<RustTest>> {
    let mut rust_tests = vec![];
    output.lines().for_each(|line| {
        if line.starts_with("test") && line.ends_with("FAILED") {
            let parts: Vec<&str> = line[FAILED_TEST_PREFIX..line.len() - FAILED_TEST_SUFFIX]
                .split("::")
                .collect();
            if parts.len() >= 2 {
                let module_path: Vec<String> = parts[..parts.len() - 1]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let method_name = parts[parts.len() - 1].to_string();
                rust_tests.push(RustTest {
                    module_path,
                    method_name,
                });
            }
        }
    });

    current_tests
        .iter()
        .fold(HashMap::new(), |mut acc, (file_path, tests)| {
            rust_tests.iter().for_each(|rust_test| {
                if tests.contains(rust_test) {
                    acc.entry(file_path.clone())
                        .or_insert(vec![])
                        .push(rust_test.clone());
                }
            });
            acc
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::rust::{helper::parse_failed_tests, rust_test::RustTest};

    #[test]
    fn collect_failed_tests() {
        let output = "
running 1 test
test tests::python::helper::tests::collect_tests ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.01s

     Running unittests src/main.rs (target/debug/deps/FzT-1105c16a9c36c56e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

warning: unused variable: `runtime_output`
  --> src/tests/java/java_test.rs:93:33
   |
93 |     fn update_failed(&mut self, runtime_output: &str) -> bool {
   |                                 ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_runtime_output`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `runtime_output`
   --> src/tests/rust/rust_test.rs:185:33
    |
185 |     fn update_failed(&mut self, runtime_output: &str) -> bool {
    |                                 ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_runtime_output`

warning: crate `FzT` should have a snake case name
  |
  = help: convert the identifier to snake case: `fz_t`
  = note: `#[warn(non_snake_case)]` on by default

warning: `FzT` (lib) generated 3 warnings
warning: `FzT` (lib test) generated 2 warnings (2 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/FzT-ae27e584e72f55cd)

running 1 test
test tests::java::java_test::tests::collect_meta ... FAILED

warning: unused variable: `runtime_output`
  --> src/tests/java/java_test.rs:93:33
   |
93 |     fn update_failed(&mut self, runtime_output: &str) -> bool {
   |                                 ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_runtime_output`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `runtime_output`
   --> src/tests/rust/rust_test.rs:185:33
    |
185 |     fn update_failed(&mut self, runtime_output: &str) -> bool {
    |                                 ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_runtime_output`

warning: crate `FzT` should have a snake case name
  |
  = help: convert the identifier to snake case: `fz_t`
  = note: `#[warn(non_snake_case)]` on by default

warning: `FzT` (lib) generated 3 warnings
warning: `FzT` (lib test) generated 2 warnings (2 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/FzT-ae27e584e72f55cd)

running 1 test
test tests::java::java_test::tests::collect_tests ... FAILED

failures:

---- tests::java::java_test::tests::collect_tests stdout ----

thread 'tests::java::java_test::tests::collect_tests' panicked at src/tests/java/java_test.rs:136:9:
assertion failed: false
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::java::java_test::tests::collect_tests

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib
        ";
        let current_tests: HashMap<String, Vec<RustTest>> = HashMap::from([(
            "tests/java/java_test/tests.java".to_string(),
            vec![
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_tests".to_string(),
                },
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_meta".to_string(),
                },
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "some_other_tests".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_meta".to_string(),
                },
            ],
        )]);

        let expected: HashMap<String, Vec<RustTest>> = HashMap::from([(
            "tests/java/java_test/tests.java".to_string(),
            vec![
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_tests".to_string(),
                },
                RustTest {
                    module_path: vec![
                        "tests".to_string(),
                        "java".to_string(),
                        "java_test".to_string(),
                        "tests".to_string(),
                    ],
                    method_name: "collect_meta".to_string(),
                },
            ],
        )]);

        let result = parse_failed_tests(output, &current_tests);

        assert_eq!(result.len(), expected.len());
        for (tests_path, tests) in &expected {
            assert_eq!(
                tests.clone().sort(),
                result.get(tests_path).unwrap().clone().sort()
            );
        }
    }
}
