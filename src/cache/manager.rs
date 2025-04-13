use home::home_dir;
use rustpython_parser::{Mode, lexer::lex, parse_tokens, source_code};
use walkdir::WalkDir;

use super::types::CacheEntry;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn collect_tests_from_file(path: &Path) -> HashSet<String> {
    let source_code = std::fs::read_to_string(path).unwrap();
    let tokens = lex(source_code.as_str(), Mode::Module);
    let ast = parse_tokens(tokens, Mode::Module, "<embedded>").unwrap();
    let mut tests = HashSet::new();
    match ast {
        rustpython_parser::ast::Mod::Module(mod_module) => {
            for stmt in mod_module.body.iter() {
                match stmt {
                    rustpython_parser::ast::Stmt::FunctionDef(stmt_function_def) => {
                        let test_name = stmt_function_def.name.to_string();
                        if test_name.starts_with("test") {
                            tests.insert(stmt_function_def.name.to_string());
                        }
                    }
                    _ => continue,
                }
            }
        }
        _ => todo!(),
    }
    tests
}

fn check_file_for_new_tests(path: &Path, tests: &HashSet<String>) -> bool {
    let collected_tests = collect_tests_from_file(path);
    &collected_tests != tests
}

fn check_change(root: &str, since: u128, tests: &HashMap<String, HashSet<String>>) -> bool {
    for entry in WalkDir::new(root) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let metadata = std::fs::metadata(entry.path()).unwrap();
            if entry.path().extension().is_none() {
                continue;
            }

            if entry.path().extension().and_then(OsStr::to_str).unwrap() != "py" {
                continue;
            }

            // TODO: Check if modified, that if modified, that there are new/removed functions
            // If so Cache entry can be replaced in place
            if let Ok(modified) = metadata.modified() {
                if modified.duration_since(UNIX_EPOCH).unwrap().as_millis() > since {
                    println!("Modified: {:?}", entry.path());
                    // TODO: tests should
                    let full_path = entry.path().as_os_str().to_str().unwrap();
                    let start_dir = Path::new(root).file_name().unwrap().to_str().unwrap();
                    let relative_path =
                        format!("{}{}", start_dir, full_path.strip_prefix(root).unwrap());
                    if check_file_for_new_tests(entry.path(), &tests[&relative_path]) {
                        println!("New tests found");
                        return true;
                    }
                }
            }
            // TODO: Check if created, that the file includes test function
            if let Ok(created) = metadata.created() {
                if created.duration_since(UNIX_EPOCH).unwrap().as_millis() > since {
                    println!("New file: {:?}", entry.path());
                    if !collect_tests_from_file(entry.path()).is_empty() {
                        println!("New tests found");
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn get_entry(project_id: &str) -> Option<CacheEntry> {
    // TODO: refactor default cache location
    let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
    cache_location.push(".fzt");
    let file_path = cache_location.join(format!("{}.json", project_id));
    if !Path::new(&file_path).exists() {
        return None;
    }
    let entry = File::open(file_path).unwrap();
    let reader = BufReader::new(entry);
    let cache_entry: CacheEntry = serde_json::from_reader(reader).unwrap();
    if check_change(
        cache_entry.root_folder.as_str(),
        cache_entry.timestamp,
        &cache_entry.tests,
    ) {
        return None;
    }
    println!("Cache Hit.");
    Some(cache_entry)
}

pub fn add_entry(project_id: String, entry: CacheEntry) {
    let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
    cache_location.push(".fzt");
    let file_path = cache_location.join(format!("{}.json", project_id));
    let file = File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &entry).unwrap();

    println!("Cache filled.");
}
