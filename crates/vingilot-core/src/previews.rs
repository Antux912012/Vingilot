use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Normalizes a URL for hashing and storage (strips fragment and trims).
pub fn normalize_url_for_preview(url: &str) -> String {
    let trimmed = url.trim();
    if let Some((base, _)) = trimmed.split_once('#') {
        base.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generates a unique 16-character SHA-256 hash filename for a URL preview image.
pub fn preview_filename_for_url(url: &str) -> String {
    let normalized = normalize_url_for_preview(url);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let hash = hasher.finalize();
    let hex = format!("{:x}", hash);
    format!("{}.png", &hex[..16])
}

/// Returns the default XDG data directory for Vingilot.
pub fn default_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir).join("vingilot");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("vingilot");
        }
    }
    PathBuf::from("/tmp/vingilot")
}

/// Returns the default XDG cache directory for Vingilot.
pub fn default_cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CACHE_HOME") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir).join("vingilot");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home).join(".cache").join("vingilot");
        }
    }
    PathBuf::from("/tmp/vingilot-cache")
}

/// Returns the default directory where preview thumbnail PNGs are stored.
pub fn default_previews_dir() -> PathBuf {
    default_cache_dir().join("previews")
}

/// Returns the default database file path.
pub fn default_db_path() -> PathBuf {
    default_data_dir().join("vingilot.db")
}

/// Clears all preview PNG files in the specified directory.
pub fn clear_preview_cache(previews_dir: &Path) -> std::io::Result<usize> {
    if !previews_dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in std::fs::read_dir(previews_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "png").unwrap_or(false) {
            let _ = std::fs::remove_file(path);
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_filename_deterministic() {
        let fn1 = preview_filename_for_url("https://rust-lang.org/learn");
        let fn2 = preview_filename_for_url("https://rust-lang.org/learn#section1");
        assert_eq!(fn1, fn2);
        assert!(fn1.ends_with(".png"));
    }
}
