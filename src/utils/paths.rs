//! Module for handling file paths used in the twtGUI application.

use std::path::PathBuf;

use crate::utils::hash::hash_sha256_str;
use directories::ProjectDirs;

/// Returns the root directory used for caching downloaded content.
///
/// Ensures the directory exists and returns an error string if it cannot be created.
pub fn cache_root() -> Result<PathBuf, String> {
    let proj = ProjectDirs::from("com", "taxevaiden", "twtGUI")
        .ok_or("Could not determine project directories")?;

    let dir = proj.cache_dir();

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    Ok(dir.to_path_buf())
}

pub fn log_root() -> Result<PathBuf, String> {
    let proj = ProjectDirs::from("com", "taxevaiden", "twtGUI")
        .ok_or("Could not determine project directories")?;

    let dir = proj.data_dir();

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    Ok(dir.to_path_buf())
}

/// Returns the cache path for storing the raw twtxt content for a given URL.
pub fn get_txt_cache_path(url: &str) -> Result<PathBuf, String> {
    let hash = hash_sha256_str(url);
    let mut path = cache_root()?;
    path.push(format!("{hash}.json"));
    Ok(path)
}

/// Returns the cache file paths used for binary downloads (data + metadata).
pub fn get_bin_cache_paths(url: &str) -> Result<(PathBuf, PathBuf), String> {
    let hash = hash_sha256_str(url);
    let dir = cache_root()?;

    let mut data_path = dir.clone();
    data_path.push(format!("{hash}.bin"));

    let mut meta_path = dir;
    meta_path.push(format!("{hash}.meta"));

    Ok((data_path, meta_path))
}

/// Returns the cache path for a parsed twtxt bundle (used to avoid re-parsing when unchanged).
pub fn get_parsed_cache_path(url: &str) -> Result<PathBuf, String> {
    let hash = hash_sha256_str(url);
    let mut path = cache_root()?;
    path.push(format!("{hash}.parsed.json"));
    Ok(path)
}
