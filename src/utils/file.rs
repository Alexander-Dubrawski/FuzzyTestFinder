use std::{fs::metadata, time::UNIX_EPOCH};

pub fn get_file_modification_timestamp(path: &str) -> u128 {
    metadata(path)
        .expect(format!("Path needs to exist: {}", path).as_str())
        .modified()
        .expect("Not supported on this platform")
        .duration_since(UNIX_EPOCH)
        .expect("System clock may have gone backwards")
        .as_millis()
}
