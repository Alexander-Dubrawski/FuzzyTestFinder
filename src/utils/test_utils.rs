#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::io;
#[cfg(test)]
use std::path::{Path, PathBuf};

#[cfg(test)]
use tempfile::TempDir;
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
pub fn copy_dict(src: &Path) -> Result<(TempDir, PathBuf), io::Error> {
    let temp_dir = tempdir()?;
    let temp_data_path = temp_dir.path().join("data").to_path_buf();
    copy_dir_recursive(src, &temp_data_path)?;
    // Resolve the real, absolute path by following all symlinks.
    // This ensures we get the physical path on disk (e.g., with /private on macOS),
    // which is required by subprocesses that don't handle symlinked paths correctly.
    let path = fs::canonicalize(temp_data_path)?;
    Ok((temp_dir, path))
}
