use home::home_dir;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::errors::FztError;

const HISTORY_SIZE: usize = 200;

pub enum HistoryGranularity {
    Test,
    File,
    Directory,
}

pub struct CacheManager {
    cache_file: PathBuf,
    history_test_granularity: PathBuf,
    history_file_granularity: PathBuf,
    history_directory_granularity: PathBuf,
}

impl CacheManager {
    pub fn new(project_id: String) -> Self {
        let mut cache_location = home_dir().expect("Could not find home directory");
        cache_location.push(".fzt");
        let cache_file = cache_location.join(format!("{}.json", project_id));
        let history_test_granularity =
            cache_location.join(format!("{}-history-test-granularity.json", project_id));
        let history_file_granularity =
            cache_location.join(format!("{}-history-file-granularity.json", project_id));
        let history_directory_granularity =
            cache_location.join(format!("{}-history-directory-granularity.json", project_id));
        Self {
            cache_file,
            history_test_granularity,
            history_file_granularity,
            history_directory_granularity,
        }
    }

    pub fn save_meta(project_id: &str, meta_data: &str) -> Result<(), FztError> {
        let mut meta_location = home_dir().expect("Could not find home directory");
        meta_location.push(".fzt");
        let path = meta_location.join(format!("{}-metadata.json", project_id));
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write(meta_data.as_bytes())?;
        Ok(())
    }

    pub fn get_meta(project_id: &str) -> Result<Option<BufReader<File>>, FztError> {
        let mut meta_location = home_dir().expect("Could not find home directory");
        meta_location.push(".fzt");
        let path = meta_location.join(format!("{}-metadata.json", project_id));

        if !Path::new(&path).exists() {
            Ok(None)
        } else {
            let entry = File::open(&path)?;
            let reader = BufReader::new(entry);
            Ok(Some(reader))
        }
    }

    pub fn new_from_path(
        cache_file: PathBuf,
        history_test_granularity: PathBuf,
        history_file_granularity: PathBuf,
        history_directory_granularity: PathBuf,
    ) -> Self {
        Self {
            cache_file,
            history_test_granularity,
            history_file_granularity,
            history_directory_granularity,
        }
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
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<(), FztError> {
        if Path::new(&self.cache_file).exists() {
            std::fs::remove_file(&self.cache_file)?;
        }
        Ok(())
    }

    pub fn clear_history(&self) -> Result<(), FztError> {
        if Path::new(&self.history_test_granularity).exists() {
            std::fs::remove_file(&self.history_test_granularity)?;
        }
        if Path::new(&self.history_file_granularity).exists() {
            std::fs::remove_file(&self.history_file_granularity)?;
        }
        if Path::new(&self.history_directory_granularity).exists() {
            std::fs::remove_file(&self.history_directory_granularity)?;
        }
        Ok(())
    }

    pub fn update_history(
        &self,
        selection: &[String],
        granularity: HistoryGranularity,
    ) -> Result<(), FztError> {
        if selection.is_empty() {
            return Ok(());
        }

        let histroy_file = match granularity {
            HistoryGranularity::Test => &self.history_test_granularity,
            HistoryGranularity::File => &self.history_file_granularity,
            HistoryGranularity::Directory => &self.history_directory_granularity,
        };

        let mut history = if !Path::new(histroy_file).exists() {
            VecDeque::new()
        } else {
            let file = File::open(histroy_file)?;
            let reader = BufReader::new(file);
            let content: VecDeque<Vec<String>> = serde_json::from_reader(reader)?;
            content
        };
        history.push_front(selection.to_vec());
        let file = File::create(histroy_file)?;
        let mut writer = BufWriter::new(file);
        if history.len() > HISTORY_SIZE {
            history.pop_back();
        }
        serde_json::to_writer(&mut writer, &history)?;
        Ok(())
    }

    pub fn recent_history_command(
        &self,
        granularity: HistoryGranularity,
    ) -> Result<Vec<String>, FztError> {
        let histroy_file = match granularity {
            HistoryGranularity::Test => &self.history_test_granularity,
            HistoryGranularity::File => &self.history_file_granularity,
            HistoryGranularity::Directory => &self.history_directory_granularity,
        };

        if !Path::new(histroy_file).exists() {
            Ok(vec![])
        } else {
            let file = File::open(histroy_file)?;
            let reader = BufReader::new(file);
            let content: Vec<Vec<String>> = serde_json::from_reader(reader)?;
            Ok(content.first().map(|tests| tests.clone()).unwrap_or(vec![]))
        }
    }

    pub fn history(&self, granularity: HistoryGranularity) -> Result<Vec<Vec<String>>, FztError> {
        let histroy_file = match granularity {
            HistoryGranularity::Test => &self.history_test_granularity,
            HistoryGranularity::File => &self.history_file_granularity,
            HistoryGranularity::Directory => &self.history_directory_granularity,
        };

        if !Path::new(histroy_file).exists() {
            Ok(vec![vec![]])
        } else {
            let file = File::open(histroy_file)?;
            let reader = BufReader::new(file);
            let content: Vec<Vec<String>> = serde_json::from_reader(reader)?;
            Ok(content)
        }
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
        let manager = CacheManager::new_from_path(path, PathBuf::from("file.path()"));
        let result = manager.get_entry().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_existing_entry() {
        let file = NamedTempFile::new().unwrap();
        let path = PathBuf::from(file.path());
        let manager = CacheManager::new_from_path(path, PathBuf::from("file.path()"));
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
        let manager = CacheManager::new_from_path(path.clone(), PathBuf::from("file.path()"));
        manager.add_entry("New").unwrap();

        let entry = File::open(path).unwrap();
        let mut reader = BufReader::new(entry);
        let mut file_content = String::new();
        reader.read_to_string(&mut file_content).unwrap();
        assert_eq!(file_content, String::from("New"));
    }
}
