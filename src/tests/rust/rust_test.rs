use std::{collections::{HashMap, HashSet}, fs, path::Path};
use proc_macro2::Span;
use syn::{visit::Visit, Attribute, File, ItemFn};

use crate::{errors::FztError, utils::file_walking::collect_tests};

// TODO: Add Hashset to collect tests
struct TestFinder<'a> {
    file_path: &'a Path,
}

impl<'ast, 'a> Visit<'ast> for TestFinder<'a> {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        if has_test_attr(&i.attrs) {
            let span: Span = i.sig.ident.span();
            let start = span.start(); // needs span-locations feature
            println!(
                "Test found: {} in file {:?} at line {}, column {}",
                i.sig.ident,
                self.file_path,
                start.line,
                start.column,
            );
        }
    }
}

fn has_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("test"))
}


fn collect_tests_from_file(path: &Path) -> Result<HashSet<String>, FztError> {
    let source = std::fs::read_to_string(path)?;
    let syntax = syn::parse_file(&source)?;
    let mut visitor = TestFinder { file_path: path };
    visitor.visit_file(&syntax);
    todo!()
}


pub fn update_tests(
    root_folder: &str,
    timestamp: &mut u128,
    tests: &mut HashMap<String, HashSet<String>>,
    only_check_for_change: bool,
) -> Result<bool, FztError> {
    collect_tests(
        root_folder,
        timestamp,
        tests,
        only_check_for_change,
        "rs",
        None,
        collect_tests_from_file,
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::collect_tests_from_file;

    #[test]
    fn foo() {
        //collect_tests_from_file(&Path::new("src/tests/rust/test_data/b/test_three.rs"));
    }
}