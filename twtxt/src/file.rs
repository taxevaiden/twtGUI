//! Module for writing tweets to the local twtxt feed.

use std::{
    fs::OpenOptions,
    io::{Read, Write},
    process::{Child, Command},
};

use chrono::{DateTime, Utc};

use crate::{
    FeedBundle, Tweet,
    metadata::Metadata,
    parsing::{parse_tweets, parse_twt_contents},
    twt_hash::{compute_twt_hash, hash_blake2b_str},
};

pub struct WriteConfig<'a> {
    pub path: &'a str,
    pub pre_script: Option<&'a str>,
    pub post_script: Option<&'a str>,
    pub script: Option<&'a str>,
    pub local_hash: Option<String>,
}

/// Builds a new tweet from the composer text and persists it to the local feed.
pub fn write(composer_text: &str, metadata: &Metadata, config: WriteConfig) -> Option<Tweet> {
    let trimmed = composer_text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let nick = metadata.nick.clone()?;
    let now = Utc::now();
    let (reply_to, display_content) = parse_twt_contents(trimmed);
    let url = metadata.urls.first().cloned().unwrap_or_default();
    let timestamp_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let written = trimmed.replace('\n', "\\u2028");

    let feed_hash = config.local_hash.or_else(|| {
        let mut file = OpenOptions::new().read(true).open(config.path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        Some(hash_blake2b_str(&contents))
    })?;

    let tweet = Tweet {
        hash: compute_twt_hash(&url, &timestamp_str, &written),
        reply_to,
        timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
            .ok()?
            .with_timezone(&Utc),
        author: nick,
        url: url.clone(),
        content: display_content.clone(),
        feed_hash: feed_hash.clone(),
    };

    if let Some(path) = config.pre_script {
        run_script(path, &[]).ok();
    }

    if let Some(path) = config.script {
        run_script(path, &[&written]).ok();
    } else if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config.path)
    {
        let _ = writeln!(file, "{}\t{}", timestamp_str, written);
    }

    if let Some(path) = config.post_script {
        run_script(path, &[]).ok();
    }

    Some(tweet)
}

fn run_script(script: &str, args: &[&str]) -> std::io::Result<Child> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", script]).args(args).spawn()
    } else {
        Command::new("sh").arg(script).args(args).spawn()
    }
}

/// Loads the user's local `twtxt.txt` feed from disk and parses it into a [`FeedBundle`].
///
/// Returns `None` when the local path is missing or the file cannot be read.
pub fn load(path: &str, metadata: &Metadata) -> Option<(String, String, FeedBundle)> {
    let content = std::fs::read_to_string(path).ok()?;

    let nick = metadata.nick.as_deref().unwrap_or_default().to_string();
    let url = metadata
        .urls
        .first()
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let hash = hash_blake2b_str(&content);

    let bundle = FeedBundle {
        metadata: Some(metadata.clone()),
        tweets: parse_tweets(&nick, &url, None, &content),
        hash,
    };

    Some((nick, url, bundle))
}
