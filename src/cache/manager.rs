use super::types::{CacheEntry, CacheUpdate};
use home::home_dir;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

pub fn get_entry(project_id: &str, cache_update: impl CacheUpdate) -> Option<CacheEntry> {
    // TODO: refactor default cache location
    let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
    cache_location.push(".fzt");
    let file_path = cache_location.join(format!("{}.json", project_id));
    if !Path::new(&file_path).exists() {
        return None;
    }
    let entry = File::open(file_path).unwrap();
    let reader = BufReader::new(entry);
    let mut cache_entry: CacheEntry = serde_json::from_reader(reader).unwrap();
    // should be lambda
    if cache_update.update(&mut cache_entry) {
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
