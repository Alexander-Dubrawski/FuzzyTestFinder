use home::home_dir;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

pub struct CacheManager {
    project_id: String,
}

impl CacheManager {
    pub fn new(project_id: String) -> Self {
        Self { project_id }
    }

    pub fn get_entry(&self) -> Option<BufReader<File>> {
        let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
        cache_location.push(".fzt");
        let file_path = cache_location.join(format!("{}.json", self.project_id));
        if !Path::new(&file_path).exists() {
            None
        } else {
            let entry = File::open(file_path).unwrap();
            Some(BufReader::new(entry))
        }
        // let mut cache_entry: CacheEntry = serde_json::from_reader(reader).unwrap();
        // if !parser.update_tests(&mut cache_entry) {
        //     println!("Cache Hit.");
        // } else {
        //     println!("Cache Miss.");
        // }
        // cache_entry
    }

    pub fn add_entry(&self, entry: &str) {
        let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
        cache_location.push(".fzt");
        let file_path = cache_location.join(format!("{}.json", self.project_id));
        let file = File::create(file_path).unwrap();
        let mut writer = BufWriter::new(file);
        writer.write(entry.as_bytes());
        println!("Cache filled.");
    }
}
