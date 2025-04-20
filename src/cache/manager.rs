use home::home_dir;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::errors::FztError;

pub struct CacheManager {
    project_id: String,
}

impl CacheManager {
    pub fn new(project_id: String) -> Self {
        Self { project_id }
    }

    pub fn get_entry(&self) -> Result<Option<BufReader<File>>, FztError> {
        let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
        cache_location.push(".fzt");
        let file_path = cache_location.join(format!("{}.json", self.project_id));
        if !Path::new(&file_path).exists() {
            Ok(None)
        } else {
            let entry = File::open(file_path)?;
            Ok(Some(BufReader::new(entry)))
        }
    }

    pub fn add_entry(&self, entry: &str) -> Result<(), FztError> {
        let mut cache_location: PathBuf = home_dir().expect("Could not find home directory");
        cache_location.push(".fzt");
        let file_path = cache_location.join(format!("{}.json", self.project_id));
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        writer.write(entry.as_bytes())?;
        println!("Cache filled.");
        Ok(())
    }
}
