use sha2::{Digest, Sha256};

pub fn project_hash(root_dir: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(root_dir.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
