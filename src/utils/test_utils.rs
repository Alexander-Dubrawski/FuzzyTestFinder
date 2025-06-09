use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(test)]
use tempfile::tempdir;

#[cfg(test)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn copy_dict(src: &Path) -> Result<String, io::Error> {
    let temp_dir = tempdir()?;
    let temp_data_path = temp_dir.path().join("data");
    copy_dir_recursive(src, &temp_data_path)?;
    Ok(temp_data_path
        .to_str()
        .expect("Path should exist")
        .to_string())
}
