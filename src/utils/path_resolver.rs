use crate::errors::FztError;
use std::path::Path;

pub fn get_relative_path(parent: &str, full_path: &str) -> Result<String, FztError> {
    let parent_path = Path::new(parent);
    let full_path_obj = Path::new(full_path);
    let relative_path = full_path_obj
        .strip_prefix(parent_path)
        .map(|path| path.to_str())
        .map_err(|e| {
            FztError::GeneralParsingError(format!(
                "Could not strip prefix {} from {}: {}",
                parent, full_path, e
            ))
        })?
        .ok_or(FztError::GeneralParsingError(format!(
            "File path could not be parsed: {}",
            full_path
        )))?;

    Ok(relative_path.to_string())
}
