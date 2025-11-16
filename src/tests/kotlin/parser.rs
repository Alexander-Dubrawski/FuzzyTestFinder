use std::{collections::{HashMap, HashSet}, path::Path};
use tree_sitter::{Node, Parser};
use tree_sitter_kotlin::language as kotlin_language;

use crate::{FztError, runtime::FailedTest};

use super::kotlin_test::KotlinTest;

// Helper: Capitalize first char (as Kotlin uses for generated class, e.g., SampleKt)
fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn extract_tests(
    node: Node,
    src: &str,
    package: &Option<String>,
    outer_class: Option<String>,
    file_stem: &str,
    found: &mut HashSet<KotlinTest>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "class_declaration" | "object_declaration" => {
                let mut class_name = None;
                let mut child_cursor = child.walk();
                for c in child.children(&mut child_cursor) {
                    if c.kind() == "type_identifier" || c.kind() == "simple_identifier" {
                        class_name = Some(src[c.byte_range()].to_string());
                        break;
                    }
                }
                // Check for nested functions and classes
                extract_tests(
                    child,
                    src,
                    package,
                    class_name.or(outer_class.clone()),
                    file_stem,
                    found,
                );
            }
            "function_declaration" => {
                let mut is_test = false;
                let mut method_name = None;
                let mut modifiers_cursor = child.walk();
                for n in child.children(&mut modifiers_cursor) {
                    if n.kind() == "modifiers" {
                        let mut mods_cursor = n.walk();
                        for mod_child in n.children(&mut mods_cursor) {
                            if mod_child.kind() == "annotation" {
                                let annotation_txt = src[mod_child.byte_range()].to_string();
                                if annotation_txt.contains("@Test") {
                                    is_test = true;
                                    break;
                                }
                            }
                        }
                    }
                    if n.kind() == "simple_identifier" {
                        method_name = Some(src[n.byte_range()].to_string());
                    }
                }
                if is_test {
                    let method = method_name.unwrap_or("<unknown>".to_string());
                    // Determine class_path for Gradle/junit runner
                    let class_path = if outer_class.is_none() {
                        match package {
                            Some(pkg) if !pkg.is_empty() => {
                                format!("{}.{}Kt", pkg, capitalize_first(file_stem))
                            }
                            _ => format!("{}Kt", capitalize_first(file_stem)),
                        }
                    } else {
                        match (package, &outer_class) {
                            (Some(pkg), Some(cls)) => format!("{pkg}.{cls}"),
                            (Some(pkg), None) => pkg.clone(),
                            (None, Some(cls)) => cls.clone(),
                            (None, None) => method.clone(),
                        }
                    };
                    found.insert(KotlinTest {
                        class_path,
                        method_name: method,
                    });
                }
            }
            _ => {
                extract_tests(child, src, package, outer_class.clone(), file_stem, found);
            }
        }
    }
}

fn extract_package(node: Node, src: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "package_header" {
            let mut child_cursor = child.walk();
            for grandchild in child.children(&mut child_cursor) {
                if grandchild.kind() == "identifier" {
                    return Some(src[grandchild.byte_range()].to_string());
                }
            }
        }
    }
    None
}

pub fn collect_tests_from_file(path: &Path) -> Result<HashSet<KotlinTest>, FztError> {
    let source_code = std::fs::read_to_string(path)?;
    let mut parser = Parser::new();
    parser
        .set_language(&kotlin_language())
        .expect("Error loading Kotlin grammar");
    let tree = parser
        .parse(&source_code, None)
        .expect("Error parsing Kotlin source");
    let root = tree.root_node();
    let package = extract_package(root, &source_code);
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("<unknown>");
    let mut found = HashSet::new();
    extract_tests(root, &source_code, &package, None, file_stem, &mut found);
    Ok(found)
}

// TODO: Could be generic between java and kotlin
pub fn parse_failed_tests(
    failed_test_output: &[FailedTest],
    current_tests: &HashMap<String, HashSet<KotlinTest>>,
) -> HashMap<String, HashSet<KotlinTest>> {
    let mut kotlin_tests = HashSet::new();

    failed_test_output.iter().for_each(|failed_test| {
        if let Some((class_path, method_name)) = failed_test.name.rsplit_once('.') {
            // We also push methods that are actually not part of the actual test,
            // but filtering them out is done later.
            kotlin_tests.insert(KotlinTest {
                class_path: class_path.to_string(),
                method_name: method_name.to_string(),
            });
        }
    });

    current_tests
        .iter()
        .fold(HashMap::new(), |mut acc, (file_path, tests)| {
            kotlin_tests.iter().for_each(|java_test| {
                if tests.contains(java_test) {
                    acc.entry(file_path.clone())
                        .or_insert(HashSet::new())
                        .insert(java_test.clone());
                }
            });
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn collects_kotlin_tests() {
        // Make a temp file
        let mut temp_path = PathBuf::from("Sample.kt");
        let code = r#"
        package com.example.demo

        import org.junit.Test

        class MyTestClass {
            @Test
            fun myTestA() { }

            fun helper() { }

            @Test
            fun myTestB() { }
        }

        object MyTestObject {
            @Test
            fun objectTest() { }
        }

        @Test
        fun topLevelTest() { }
        "#;
        fs::write(&temp_path, code).unwrap();

        let tests = collect_tests_from_file(&temp_path).unwrap();
        let tests_set: std::collections::HashSet<KotlinTest> = tests.into_iter().collect();

        // Check for expected test names.
        let expected = vec![
            KotlinTest {
                class_path: "com.example.demo.MyTestClass".to_string(),
                method_name: "myTestA".to_string(),
            },
            KotlinTest {
                class_path: "com.example.demo.MyTestClass".to_string(),
                method_name: "myTestB".to_string(),
            },
            KotlinTest {
                class_path: "com.example.demo.MyTestObject".to_string(),
                method_name: "objectTest".to_string(),
            },
            KotlinTest {
                class_path: "com.example.demo.SampleKt".to_string(),
                method_name: "topLevelTest".to_string(),
            }, // top-level
        ]
        .into_iter()
        .collect();

        assert_eq!(tests_set, expected);

        fs::remove_file(&temp_path).unwrap();
    }
}
