use std::path::PathBuf;

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
