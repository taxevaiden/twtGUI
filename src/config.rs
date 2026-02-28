use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};

use crate::utils::Metadata;

fn config_path() -> Result<PathBuf, Box<dyn Error>> {
    let proj = ProjectDirs::from("com", "taxevaiden", "twtGUI")
        .ok_or("Could not determine config directory")?;

    let dir = proj.config_dir();
    fs::create_dir_all(dir)?;

    Ok(dir.join("config.toml"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub metadata: Metadata,
    pub paths: AppFilePaths,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppFilePaths {
    pub twtxt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            metadata: Metadata {
                urls: Vec::new(),
                nick: None,
                avatar: None,
                description: None,
                kind: None,
                follows: Vec::new(),
                following: None,
                links: Vec::new(),
                prev: Vec::new(),
                refresh: None,
            },
            paths: AppFilePaths {
                twtxt: String::new(),
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path = config_path()?;

        if !path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let contents = fs::read_to_string(&path)?;

        if contents.trim().is_empty() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = config_path()?;

        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;

        Ok(())
    }
}
