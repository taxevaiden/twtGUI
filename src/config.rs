use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
        let path = Path::new("config.ini");

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap()).ok();
            fs::write(path, "shegoncallmebabyboo").unwrap();
        }

        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()
            .expect("Failed to load config");

        settings.try_deserialize().expect("Invalid config format")
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut ini = ini::Ini::new();

        // Settings section
        ini.with_section(Some("settings"))
            .set("nick", &self.settings.nick)
            .set("twtxt", &self.settings.twtxt)
            .set("twturl", &self.settings.twturl);

        // Following section
        if let Some(following) = &self.following {
            let mut section = ini.with_section(Some("following"));
            for (name, url) in following {
                section.set(name, url);
            }
        }

        ini.write_to_file("config.ini")?;
        Ok(())
    }
}
