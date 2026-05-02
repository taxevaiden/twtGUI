//! Download and caching utilities used by the application.
//!
//! This module provides simple caching for HTTP requests (ETag / Last-Modified) and
//! a convenience wrapper for downloading + parsing twtxt feeds.

use crate::utils::FeedBundle;
use bytes::Bytes;
use iced::widget::markdown;
use opengraph::{self, Object};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use serde::{Deserialize, Serialize};

use tracing::{debug, info};

use crate::utils::ParsedCache;
use crate::utils::hash::hash_sha256_str;
use crate::utils::paths::{get_bin_cache_paths, get_parsed_cache_path, get_txt_cache_path};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("BUILD_VERSION"));

use std::sync::OnceLock;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_client() -> reqwest::Client {
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()
                .expect("Failed to build client")
        })
        .clone()
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheMetadata {
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheEntry {
    content: String,
    metadata: CacheMetadata,
}

/// Downloads a binary file and caches it on disk using HTTP caching headers.
///
/// This can be used to download anything, however it's primarily intended for images
/// (avatars and inline media) used in tweets.
///
/// Returns the cached bytes if the server responds with `304 Not Modified`.
pub async fn download_binary(url: String) -> Result<Bytes, String> {
    let client = get_client();

    debug!("Downloading file from {}", url);

    let (data_path, meta_path) = get_bin_cache_paths(&url)?;

    let metadata: Option<CacheMetadata> = std::fs::read_to_string(&meta_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    let mut request = client.get(&url);
    if let Some(ref meta) = metadata {
        if let Some(ref etag) = meta.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(ref last_mod) = meta.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_mod);
        }
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    // 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        info!(
            "304 Not Modified: {}\n\tData: {}\n\tMetadata: {}",
            url,
            data_path.display(),
            meta_path.display()
        );
        let raw_bytes = std::fs::read(&data_path).map_err(|e| e.to_string())?;
        return Ok(Bytes::from(raw_bytes));
    }

    // 200 OK
    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let data = response.bytes().await.map_err(|e| e.to_string())?;

    let meta_json = serde_json::to_string(&CacheMetadata {
        etag,
        last_modified,
    })
    .map_err(|e| e.to_string())?;

    info!(
        "200 OK: {}\n\tWriting {} bytes to data {}\n\tWriting {} bytes to metadata {}",
        url,
        data.len(),
        data_path.display(),
        meta_json.len(),
        meta_path.display()
    );

    std::fs::write(&data_path, &data).map_err(|e| e.to_string())?;
    std::fs::write(&meta_path, meta_json).map_err(|e| e.to_string())?;

    Ok(data)
}

/// Downloads a page/file as plain text and caches it locally.
///
/// Uses HTTP `ETag`/`Last-Modified` headers to avoid re-downloading unchanged files.
pub async fn download_text(url: String) -> Result<String, String> {
    let client = get_client();

    debug!("Downloading text from {}", url);

    let cache_path = get_txt_cache_path(&url)?;

    let cached_data: Option<CacheEntry> = std::fs::read_to_string(&cache_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok());

    let mut request = client.get(&url);
    if let Some(ref entry) = cached_data {
        if let Some(ref etag) = entry.metadata.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(ref last_mod) = entry.metadata.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_mod);
        }
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    // 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        info!("304 Not Modified: {}\n\t{}", url, cache_path.display());
        return cached_data
            .map(|e| e.content)
            .ok_or_else(|| "Server returned 304 but no local file found".to_string());
    }

    // 200 OK
    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let content = response.text().await.map_err(|e| e.to_string())?;

    let new_entry = CacheEntry {
        content: content.clone(),
        metadata: CacheMetadata {
            etag,
            last_modified,
        },
    };

    let serialized = serde_json::to_string(&new_entry).map_err(|e| e.to_string())?;
    info!(
        "200 OK: {}\n\tWriting {} bytes to {}",
        url,
        serialized.len(),
        cache_path.display()
    );
    std::fs::write(cache_path, serialized).map_err(|e| e.to_string())?;

    Ok(content)
}

pub async fn download_opengraph(url: String) -> Result<Object, String> {
    let page = download_text(url.clone()).await?;
    let obj = opengraph::extract(&mut page.to_string().as_bytes(), Default::default())
        .map_err(|e| e.to_string())?;
    Ok(obj)
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

    let metadata = crate::utils::parse_metadata(&raw);

    let canonical_nick = metadata
        .as_ref()
        .and_then(|m| m.nick.as_ref())
        .cloned()
        .unwrap_or_else(|| {
            url::Url::parse(&url)
                .ok()
                .and_then(|u| u.host_str().map(str::to_string))
                .unwrap_or_else(|| nick.clone())
        });

    let tweets = crate::utils::parse_tweets(&canonical_nick, &url, hash_url.as_deref(), &raw);

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
