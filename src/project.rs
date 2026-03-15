use std::path::{Path, PathBuf};

/// Compute a deterministic DB path for a project directory.
pub fn db_path_for(project_dir: &Path) -> PathBuf {
    let canonical = project_dir
        .canonicalize()
        .unwrap_or_else(|_| project_dir.to_path_buf());
    let hash = simple_hash(canonical.to_string_lossy().as_bytes());
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".claude")
        .join("file-suggest")
        .join(format!("{hash}.db"))
}

/// Simple non-crypto hash (FNV-1a 64-bit, hex-encoded, first 16 chars).
fn simple_hash(data: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Ensure the DB directory exists.
pub fn ensure_db_dir() {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let dir = home.join(".claude").join("file-suggest");
    let _ = std::fs::create_dir_all(dir);
}
