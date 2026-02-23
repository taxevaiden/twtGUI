use serde::{Deserialize, Serialize};

use directories::ProjectDirs;
use std::path::PathBuf;

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let proj = ProjectDirs::from("com", "taxevaiden", "twtGUI")
        .ok_or("Could not determine config directory")?;

    let dir = proj.config_dir();
    std::fs::create_dir_all(dir)?;

    Ok(dir.join("config.ini"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
    pub following: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub nick: String,
    pub twtxt: String,
    pub twturl: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path().expect("Failed to determine config path");

        // Create default config if missing
        if !path.exists() {
            let default = AppConfig {
                settings: Settings {
                    nick: "anon".into(),
                    twtxt: "".into(),
                    twturl: "".into(),
                },
                following: None,
            };

            default.save().expect("Failed to create default config");
            return default;
        }

        let settings = config::Config::builder()
            .add_source(config::File::from(path.clone()))
            .build()
            .expect("Failed to load config");

        settings.try_deserialize().expect("Invalid config format")
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path()?;

        let mut ini = ini::Ini::new();

        ini.with_section(Some("settings"))
            .set("nick", &self.settings.nick)
            .set("twtxt", &self.settings.twtxt)
            .set("twturl", &self.settings.twturl);

        if let Some(following) = &self.following {
            let mut section = ini.with_section(Some("following"));
            for (name, url) in following {
                section.set(name, url);
            }
        }

        ini.write_to_file(path)?;
        Ok(())
    }
}
