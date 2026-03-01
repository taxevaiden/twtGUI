use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::utils::Metadata;
use std::collections::{HashMap, HashSet};

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
            let mut default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let contents = fs::read_to_string(&path)?;

        if contents.trim().is_empty() {
            let mut default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.metadata.following.is_none()
            || self.metadata.following.unwrap_or(0) != self.metadata.follows.len() as u64
        {
            self.metadata.following = Some(self.metadata.follows.len() as u64);
        }

        let path = config_path()?;
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(&path, toml_string)?;

        // sync metadata with twtxt.txt

        if self.paths.twtxt.is_empty() {
            return Ok(());
        }

        let twtxt_path = Path::new(&self.paths.twtxt);

        let mut existing_tweets = Vec::new();
        let mut metadata_lines = Vec::new();
        if twtxt_path.exists() {
            let file = fs::File::open(twtxt_path)?;
            let reader = BufReader::new(file);
            let mut in_metadata = true;
            for line in reader.lines() {
                let line = line?;
                if in_metadata && !line.starts_with('#') && !line.trim().is_empty() {
                    in_metadata = false;
                }
                if in_metadata {
                    metadata_lines.push(line);
                } else {
                    existing_tweets.push(line);
                }
            }
        }

        // build known map
        let mut known_map: HashMap<&str, Vec<String>> = HashMap::new();
        if let Some(nick) = &self.metadata.nick {
            known_map.insert("nick", vec![nick.clone()]);
        }
        if let Some(desc) = &self.metadata.description {
            known_map.insert("description", vec![desc.clone()]);
        }
        if let Some(avatar) = &self.metadata.avatar {
            known_map.insert("avatar", vec![avatar.clone()]);
        }
        if let Some(kind) = &self.metadata.kind {
            known_map.insert("kind", vec![kind.clone()]);
        }
        if !self.metadata.urls.is_empty() {
            known_map.insert("url", vec![self.metadata.urls[0].clone()]);
        }
        if let Some(following_count) = &self.metadata.following {
            known_map.insert("following", vec![following_count.to_string()]);
        }
        if !self.metadata.links.is_empty() {
            known_map.insert(
                "link",
                self.metadata
                    .links
                    .iter()
                    .map(|l| format!("{} {}", l.text, l.url))
                    .collect(),
            );
        }
        if !self.metadata.prev.is_empty() {
            known_map.insert(
                "prev",
                self.metadata
                    .prev
                    .iter()
                    .map(|l| format!("{} {}", l.text, l.url))
                    .collect(),
            );
        }
        if let Some(refresh) = &self.metadata.refresh {
            known_map.insert("refresh", vec![refresh.to_string()]);
        }

        // separate special lines from other metadata
        let mut special_lines: HashMap<&str, Vec<String>> = HashMap::new();
        let mut other_lines = Vec::new();

        for line in &metadata_lines {
            let trimmed = line.trim_start_matches('#').trim();
            if trimmed.starts_with("follow ") {
                special_lines
                    .entry("follow")
                    .or_default()
                    .push(line.clone());
            } else if trimmed.starts_with("link ") {
                special_lines.entry("link").or_default().push(line.clone());
            } else if trimmed.starts_with("prev ") {
                special_lines.entry("prev").or_default().push(line.clone());
            } else {
                other_lines.push(line.clone());
            }
        }

        // helper to sync special lines
        fn sync_special(lines: &mut Vec<String>, new_values: &HashSet<String>) {
            // remove deleted
            lines.retain(|l| new_values.contains(l.trim_start_matches('#').trim()));
            // add new
            for val in new_values {
                if !lines
                    .iter()
                    .any(|l| l.trim_start_matches('#').trim() == val)
                {
                    lines.push(format!("# {}", val));
                }
            }
        }

        // sync all fields that are vecs (follows, links, prevs)
        let follow_set: HashSet<String> = self
            .metadata
            .follows
            .iter()
            .map(|f| format!("follow = {} {}", f.text, f.url))
            .collect();
        sync_special(special_lines.entry("follow").or_default(), &follow_set);

        let link_set: HashSet<String> = self
            .metadata
            .links
            .iter()
            .map(|l| format!("link = {} {}", l.text, l.url))
            .collect();
        sync_special(special_lines.entry("link").or_default(), &link_set);

        let prev_set: HashSet<String> = self
            .metadata
            .prev
            .iter()
            .map(|l| format!("prev = {} {}", l.text, l.url))
            .collect();
        sync_special(special_lines.entry("prev").or_default(), &prev_set);

        let mut used_keys: HashSet<String> = HashSet::new();
        let mut updated_other_lines = Vec::new();

        for line in other_lines {
            let trimmed = line.trim_start_matches('#').trim();
            let key = trimmed.split_whitespace().next().unwrap_or("");

            if let Some(values) = known_map.get(key) {
                if !values.is_empty() && !used_keys.contains(key) {
                    updated_other_lines.push(format!("# {} = {}", key, values[0].clone()));
                    used_keys.insert(key.to_string());
                    continue;
                }
            }

            updated_other_lines.push(line);
        }

        // prepend missing known metadata (except follow/link/prev)
        let keys_order = [
            "nick",
            "description",
            "avatar",
            "kind",
            "url",
            "following",
            "refresh",
        ];

        for &key in &keys_order {
            if let Some(values) = known_map.get(key) {
                if !used_keys.contains(key) {
                    for val in values {
                        updated_other_lines.push(format!("# {} = {}", key, val));
                    }
                }
            }
        }

        let mut final_lines = updated_other_lines;

        if let Some(f) = special_lines.get("follow") {
            final_lines.extend(f.clone());
        }
        if let Some(l) = special_lines.get("link") {
            final_lines.extend(l.clone());
        }
        if let Some(p) = special_lines.get("prev") {
            final_lines.extend(p.clone());
        }

        final_lines.extend(existing_tweets);

        let mut output = final_lines.join("\n");
        output.push('\n');

        fs::write(twtxt_path, output)?;
        Ok(())
    }
}
