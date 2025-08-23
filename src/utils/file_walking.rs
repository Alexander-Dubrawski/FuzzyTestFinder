use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    hash::Hash,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::{DirEntry, WalkDir};

use crate::errors::FztError;

pub fn filter_out_deleted_files<T>(root_dir: &str, tests: &mut HashMap<String, T>) -> bool {
    let mut tests_to_remove = vec![];
    for path in tests.keys() {
        let local_path = Path::new(root_dir).join(path);
        if !std::path::absolute(local_path)
            .expect("Should be valid path")
            .exists()
        {
            tests_to_remove.push(path.clone());
        }
    }
    let updated = tests_to_remove.len() > 0;
    tests_to_remove.into_iter().for_each(|test_path| {
        tests.remove(&test_path);
    });
    updated
}

fn is_hidden(entry: &DirEntry) -> bool {
    let hidden = entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false);
    hidden
}

pub fn collect_tests<T: Eq + Hash>(
    root_folder: &str,
    timestamp: &mut u128,
    tests: &mut HashMap<String, HashSet<T>>,
    only_check_for_change: bool,
    file_extention: &str,
    regex_pattern: Option<Regex>,
    collect_tests_from_file: impl Fn(&Path) -> Result<HashSet<T>, FztError>,
) -> Result<bool, FztError> {
    let mut updated = false;
    for entry in WalkDir::new(root_folder)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let metadata = std::fs::metadata(entry.path())?;
            if entry.path().extension().is_none() {
                continue;
            }

            if entry
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .expect("Is file type")
                != file_extention
            {
                continue;
            }

            if let Some(pattern) = regex_pattern.as_ref() {
                if !pattern.is_match(
                    entry
                        .path()
                        .file_name()
                        .expect("Is file type")
                        .to_str()
                        .expect("Is file type"),
                ) {
                    continue;
                }
            }

            let full_path = entry.path().as_os_str().to_str().expect("Is file type");
            let relative_path = full_path
                .strip_prefix(root_folder)
                .map(|path| path.strip_prefix("/"))
                .flatten()
                .ok_or(FztError::GeneralParsingError(format!(
                    "File path could not be parsed: {}",
                    full_path
                )))?;

            if let Ok(modified) = metadata.modified() {
                if modified.duration_since(UNIX_EPOCH)?.as_millis() > *timestamp {
                    let new_tests = collect_tests_from_file(entry.path())?;
                    if !tests.contains_key(relative_path) {
                        updated = true;
                        tests.insert(relative_path.to_string(), new_tests);
                        continue;
                    }
                    if new_tests != tests[relative_path] {
                        if only_check_for_change {
                            return Ok(true);
                        }
                        updated = true;
                        let entry = tests.get_mut(relative_path).expect("contains key");
                        *entry = new_tests;
                    }
                }
            }
            if let Ok(created) = metadata.created() {
                if created.duration_since(UNIX_EPOCH)?.as_millis() > *timestamp {
                    let new_tests = collect_tests_from_file(entry.path())?;
                    if !new_tests.is_empty() {
                        if only_check_for_change {
                            return Ok(true);
                        }
                        tests.insert(relative_path.to_string(), new_tests);
                        updated = true;
                    }
                }
            }
        }
    }
    if updated {
        *timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    }
    Ok(updated)
}
