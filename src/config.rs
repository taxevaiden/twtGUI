use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
    pub following: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize)]
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
}
