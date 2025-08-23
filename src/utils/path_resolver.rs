use crate::errors::FztError;

pub fn get_relative_path(parent: &str, full_path: &str) -> Result<String, FztError> {
    let relative_path = full_path
        .strip_prefix(parent)
        .map(|path| path.strip_prefix("/"))
        .flatten()
        .ok_or(FztError::GeneralParsingError(format!(
            "File path could not be parsed: {}",
            full_path
        )))?;

    Ok(relative_path.to_string())
}
