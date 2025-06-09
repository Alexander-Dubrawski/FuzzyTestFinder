use std::env;

use sha2::{Digest, Sha256};

use crate::errors::FztError;

pub fn project_hash() -> Result<String, FztError> {
    let path = env::current_dir()?;
    let root_dir = path.to_string_lossy().to_string();
    let mut hasher = Sha256::new();
    hasher.update(root_dir.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
