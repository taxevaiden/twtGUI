//! Twtxt utilities for parsing, hashing, and threading tweets from twtxt feeds.

use crate::config::AppConfig;
use crate::twtxt::hash::compute_twt_hash;
use crate::twtxt::metadata::Metadata;
use crate::twtxt::parsing::{parse_metadata, parse_tweets, parse_twt_contents};
use crate::utils::download::{ParsedCache, download_text};
use crate::utils::hash::hash_sha256_str;
use crate::utils::paths::get_parsed_cache_path;
use chrono::{DateTime, Utc};
use iced::widget::markdown;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process::{Child, Command};

use serde::{Deserialize, Serialize};

pub mod hash;
pub mod metadata;
pub mod parsing;
pub mod threading;

/// A parsed tweet from a twtxt feed.
///
/// This type is used throughout the UI to render timelines, threads and views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    /// A computed hash that identifies the tweet uniquely.
    pub hash: String,

    /// The hash of the tweet that this tweet replies to (if any).
    ///
    /// This is parsed from the `(#<hash>)` prefix in the twtxt line.
    pub reply_to: Option<String>,

    /// The display name of the author that was used when parsing the feed.
    pub author: String,

    /// The parsed timestamp of the tweet.
    pub timestamp: DateTime<Utc>,

    /// The feed URL that provided this tweet.
    pub url: String,

    /// The markdown-ready content extracted from the twtxt line.
    pub content: String,

    /// The sha256 hash of the feed that provided this tweet.
    pub feed_hash: String,

    /// The parsed markdown items used by the UI renderer.
    #[serde(skip)]
    pub md_items: Vec<markdown::Item>,
}
/// A node in the thread/tree representation of tweets.
///
/// `index` is the index into the flat tweet list, and `children` are replies.
#[derive(Debug, Clone)]
pub struct TweetNode {
    pub index: usize,
    pub children: Vec<TweetNode>,
}

/// A coherent bundle of feed data (tweets and optional metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedBundle {
    pub tweets: Vec<Tweet>,
    // Metadata could be missing since it's possible a feed could be a twtxt v1 feed
    pub metadata: Option<Metadata>,
}

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

fn run_script(script: &str, args: &[&str]) -> std::io::Result<Child> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", script]).args(args).spawn()
    } else {
        Command::new("sh").arg(script).args(args).spawn()
    }
}

/// Downloads a twtxt feed, parses it into a `ParsedCache`, and caches the parsed result.
///
/// If the feed content has not changed since the last download, the previously parsed
/// bundle is reused.
///
/// `nick` is the display name to use for tweets when the feed metadata does not include one.
/// `use_nick` controls whether the provided nick should override the feed's own nick.
///
/// Note that `nick` is only used as a display name, and does not affect the actual cached content.
///
/// `hash_url` is the URL to use for hashes (feed_hash, twt hash). If `None`, the main feed URL will be used.
pub async fn download_and_parse_twtxt(
    nick: String,
    url: String,
    hash_url: Option<String>,
    use_nick: bool,
) -> Result<ParsedCache, String> {
    let raw = download_text(url.clone()).await?;
    let raw_hash = hash_sha256_str(&raw);
    let parsed_path = get_parsed_cache_path(&url)?;

    if let Ok(cached_str) = std::fs::read_to_string(&parsed_path)
        && let Ok(mut cache) = serde_json::from_str::<ParsedCache>(&cached_str)
        && cache.content_hash == raw_hash
    {
        for tweet in &mut cache.bundle.tweets {
            tweet.md_items = markdown::parse(&tweet.content).collect();
        }
        return Ok(apply_nick_override(cache, &nick, use_nick));
    }

    let metadata = parse_metadata(&raw);

    let canonical_nick = metadata
        .as_ref()
        .and_then(|m| m.nick.as_ref())
        .cloned()
        .unwrap_or_else(|| {
            reqwest::Url::parse(&url)
                .ok()
                .and_then(|u| u.host_str().map(str::to_string))
                .unwrap_or_else(|| nick.clone())
        });

    let tweets = parse_tweets(&canonical_nick, &url, hash_url.as_deref(), &raw);

    let mut cache = ParsedCache {
        content_hash: raw_hash,
        bundle: FeedBundle { tweets, metadata },
    };

    let serialized = serde_json::to_string(&cache).map_err(|e| e.to_string())?;
    let _ = std::fs::write(parsed_path, serialized);

    for tweet in &mut cache.bundle.tweets {
        tweet.md_items = markdown::parse(&tweet.content).collect();
    }

    Ok(apply_nick_override(cache, &nick, use_nick))
}

/// Optionally overrides the author name for all tweets in the bundle.
fn apply_nick_override(mut parsed: ParsedCache, nick: &str, use_nick: bool) -> ParsedCache {
    if use_nick {
        for tweet in &mut parsed.bundle.tweets {
            tweet.author = nick.to_string();
        }
    }

    parsed
}
