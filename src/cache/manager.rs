use home::home_dir;
use walkdir::WalkDir;

use super::types::CacheEntry;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn check_change(root: &str, since: u128) -> bool {
    for entry in WalkDir::new(root) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let metadata = std::fs::metadata(entry.path()).unwrap();
            if let Ok(modified) = metadata.modified() {
                if modified.duration_since(UNIX_EPOCH).unwrap().as_millis() > since {
                    println!("Modified: {:?}", entry.path());
                    return true; // Found a modified file
                }
            }
            if let Ok(created) = metadata.created() {
                if created.duration_since(UNIX_EPOCH).unwrap().as_millis() > since {
                    println!("New file: {:?}", entry.path());
                    return true;
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
    if check_change(cache_entry.root_folder.as_str(), cache_entry.timestamp) {
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
