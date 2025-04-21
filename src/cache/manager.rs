use home::home_dir;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::errors::FztError;

pub struct CacheManager {
    cache_file: PathBuf,
}

impl CacheManager {
    pub fn new(project_id: String) -> Self {
        let mut cache_location = home_dir().expect("Could not find home directory");
        cache_location.push(".fzt");
        let cache_file = cache_location.join(format!("{}.json", project_id));
        Self { cache_file }
    }

    pub fn new_from_path(cache_file: PathBuf) -> Self {
        Self { cache_file }
    }

    pub fn get_entry(&self) -> Result<Option<BufReader<File>>, FztError> {
        if !Path::new(&self.cache_file).exists() {
            Ok(None)
        } else {
            let entry = File::open(&self.cache_file)?;
            Ok(Some(BufReader::new(entry)))
        }
    }

    pub fn add_entry(&self, entry: &str) -> Result<(), FztError> {
        let file = File::create(&self.cache_file)?;
        let mut writer = BufWriter::new(file);
        writer.write(entry.as_bytes())?;
        println!("Cache filled.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::NamedTempFile;

    #[test]
    fn get_non_existing_entry() {
        let path = PathBuf::from("/ifhoeowhfoew/oihsoehwofihwoih.json");
        let manager = CacheManager::new_from_path(path);
        let result = manager.get_entry().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_existing_entry() {
        let file = NamedTempFile::new().unwrap();
        let path = PathBuf::from(file.path());
        let manager = CacheManager::new_from_path(path);
        let mut reader = manager.get_entry().unwrap().unwrap();
        let mut file_content = String::new();
        reader.read_to_string(&mut file_content).unwrap();
        assert_eq!(file_content, String::from(""));
    }

    #[test]
    fn add_new_entry() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Old").unwrap();
        let path = PathBuf::from(file.path());
        let manager = CacheManager::new_from_path(path.clone());
        manager.add_entry("New").unwrap();

        let entry = File::open(path).unwrap();
        let mut reader = BufReader::new(entry);
        let mut file_content = String::new();
        reader.read_to_string(&mut file_content).unwrap();
        assert_eq!(file_content, String::from("New"));
    }
}
