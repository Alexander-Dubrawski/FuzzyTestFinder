use crate::parser::Parser;

use super::types::CacheEntry;
use home::home_dir;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

pub fn get_entry(project_id: &str, parser: impl Parser) -> CacheEntry {
    let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
    cache_location.push(".fzt");
    let file_path = cache_location.join(format!("{}.json", project_id));
    if !Path::new(&file_path).exists() {
        let entry = parser.parse_test();
        add_entry(project_id, &entry);
        return entry;
    }
    let entry = File::open(file_path).unwrap();
    let reader = BufReader::new(entry);
    let mut cache_entry: CacheEntry = serde_json::from_reader(reader).unwrap();
    if !parser.update_tests(&mut cache_entry) {
        println!("Cache Hit.");
    } else {
        println!("Cache Miss.");
    }
    cache_entry
}

fn add_entry(project_id: &str, entry: &CacheEntry) {
    let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
    cache_location.push(".fzt");
    let file_path = cache_location.join(format!("{}.json", project_id));
    let file = File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, entry).unwrap();
    println!("Cache filled.");
}
