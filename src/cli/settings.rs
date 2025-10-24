use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;

use home::home_dir;

use crate::FztError;

const DEFAULT_CONFIG: &str = r#"# ==========================
# The config file consists of simple key-value pairs,
# separated by equals signs.

# Spacing around the equals sign does not matter.
# All of these are identical:
# key=value
# key= value
# key =value
# key = value

# Available settings
# preview=file | f | test | t | directory | d | select | s | auto | a | none
# mode=directory | file | test | runtime | append | s | select

# Default settings (uncomment and modify as needed)
# preview=auto
# mode=test
"#;

pub fn update_settings() -> Result<(), FztError> {
    let mut settings_location = home_dir().expect("Could not find home directory");
    settings_location.push(".fzt");
    settings_location.push("config");

    if !settings_location.exists() {
        let mut file = File::create(&settings_location)?;
        file.write_all(DEFAULT_CONFIG.as_bytes())?;
    }

    open_in_editor(&settings_location)?;
    Ok(())
}

/// Open a file in the user's preferred editor
fn open_in_editor(path: &PathBuf) -> Result<(), FztError> {
    // Try to get the editor from environment variables
    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| {
            // Fallback editors based on platform
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else if cfg!(target_os = "macos") {
                "open".to_string()
            } else {
                // Unix/Linux - try common editors
                "vim".to_string()
            }
        });

    let status = Command::new(&editor).arg(path).status().map_err(|e| {
        FztError::RuntimeError(format!("Failed to open editor '{}': {}", editor, e))
    })?;

    if !status.success() {
        return Err(FztError::RuntimeError(format!(
            "Editor '{}' exited with non-zero status",
            editor
        )));
    }

    Ok(())
}

/// Parse the config file and return a HashMap of key-value pairs
fn parse_config(config_path: PathBuf) -> Result<HashMap<String, String>, FztError> {
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let mut config = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Split on '=' sign
        if let Some(equals_pos) = trimmed.find('=') {
            let key = trimmed[..equals_pos].trim().to_string();
            let value = trimmed[equals_pos + 1..].trim().to_string();

            // Only add non-empty keys
            if !key.is_empty() {
                config.insert(key, value);
            }
        }
    }

    Ok(config)
}

/// Load config from the default location
pub fn load_config() -> Result<HashMap<String, String>, FztError> {
    let mut settings_location = home_dir().expect("Could not find home directory");
    settings_location.push(".fzt");
    settings_location.push("config");

    if !settings_location.exists() {
        return Ok(HashMap::new());
    }

    parse_config(settings_location)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "preview=file").unwrap();
        writeln!(temp_file, "mode = directory").unwrap();
        writeln!(temp_file, "  key = value  ").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "# Another comment").unwrap();
        writeln!(temp_file, "test=123").unwrap();

        let config = parse_config(temp_file.path().to_path_buf()).unwrap();

        assert_eq!(config.get("preview"), Some(&"file".to_string()));
        assert_eq!(config.get("mode"), Some(&"directory".to_string()));
        assert_eq!(config.get("key"), Some(&"value".to_string()));
        assert_eq!(config.get("test"), Some(&"123".to_string()));
        assert_eq!(config.len(), 4);
    }

    #[test]
    fn test_empty_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Only comments").unwrap();
        writeln!(temp_file, "").unwrap();

        let config = parse_config(temp_file.path().to_path_buf()).unwrap();
        assert!(config.is_empty());
    }
}
