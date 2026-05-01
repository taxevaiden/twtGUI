//! Twtxt domain helpers and feed utilities.
//!
//! This module keeps twtxt-specific parsing, local feed loading, and tweet
//! composition logic separate from the UI pages.

use crate::config::AppConfig;
use crate::utils::hash::hash_sha256_str;
use crate::utils::{
    FeedBundle, ParsedCache, Tweet, compute_twt_hash, download_and_parse_twtxt, parse_metadata,
    parse_tweets, parse_twt_contents,
};
use chrono::{DateTime, Utc};
use iced::widget::markdown;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process::{Child, Command};

/// Loads the user's local `twtxt.txt` feed from disk.
///
/// Returns `None` when the local path is missing or the file cannot be read.
pub fn load_local_twtxt_feed(config: &AppConfig) -> Option<(String, String, ParsedCache)> {
    let path = std::path::Path::new(&config.paths.twtxt);
    let content = std::fs::read_to_string(path).ok()?;

    let nick = config.metadata.nick.clone().unwrap_or_default();
    let url = config.metadata.urls.first().cloned().unwrap_or_default();

    let bundle = FeedBundle {
        metadata: parse_metadata(&content),
        tweets: parse_tweets(&nick, &url, None, &content),
    };

    let content_hash = hash_sha256_str(&content);

    Some((
        nick,
        url,
        ParsedCache {
            bundle,
            content_hash,
        },
    ))
}

/// Builds a new tweet from the composer text and persists it to the local feed.
///
/// This helper is intentionally separated from the view layer so that the UI
/// page only has to manage state changes and not the twtxt file logic.
pub fn compose_twtxt_tweet(
    composer_text: &str,
    config: &AppConfig,
    local_hash: Option<String>,
) -> Option<Tweet> {
    let trimmed = composer_text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let nick = config.metadata.nick.clone()?;
    let now = Utc::now();
    let (reply_to, display_content) = parse_twt_contents(trimmed);
    let url = config.metadata.urls.first().cloned().unwrap_or_default();
    let timestamp_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let written = trimmed.replace('\n', "\\u2028");

    let feed_hash = local_hash.or_else(|| {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&config.paths.twtxt)
            .ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        Some(hash_sha256_str(&contents))
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
        md_items: markdown::parse(&display_content).collect(),
    };

    if let Some(path) = &config.paths.pre_tweet_script {
        run_script(path, &[]).ok();
    }

    if let Some(path) = &config.paths.tweet_script {
        run_script(path, &[&written]).ok();
    } else if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.paths.twtxt)
    {
        let _ = writeln!(file, "{}\t{}", timestamp_str, written);
    }

    if let Some(path) = &config.paths.post_tweet_script {
        run_script(path, &[]).ok();
    }

    Some(tweet)
}

/// Downloads a twtxt feed and parses it.
///
/// This helper is a thin wrapper around the shared twtxt downloader and parser.
pub async fn download_and_parse_feed(
    nick: String,
    url: String,
    hash_url: Option<String>,
    use_nick: bool,
) -> Result<ParsedCache, String> {
    download_and_parse_twtxt(nick, url, hash_url, use_nick).await
}

fn run_script(script: &str, args: &[&str]) -> std::io::Result<Child> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", script]).args(args).spawn()
    } else {
        Command::new("sh").arg(script).args(args).spawn()
    }
}
