use std::collections::HashMap;

use chrono::{DateTime, Local, Utc};
use iced::{
    Color, Length, font,
    widget::{Column, column, container, rich_text, span, text},
};

#[derive(Clone)]
pub struct Tweet {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub content: String,
}

pub fn parse_metadata(input: &str) -> Option<HashMap<String, String>> {
    let map: HashMap<String, String> = input
        .lines()
        .filter_map(|line| {
            if line.starts_with('#') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((
                        parts[0].trim_start_matches('#').trim().to_string(),
                        parts[1].trim().to_string(),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if map.is_empty() { None } else { Some(map) }
}

pub fn parse_tweets(author: &str, input: &str) -> Vec<Tweet> {
    input
        .lines()
        .filter_map(|line| {
            if !line.starts_with("#") {
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.len() == 2 {
                    Some(Tweet {
                        timestamp: DateTime::parse_from_rfc3339(parts[0]).ok()?.to_utc(),
                        author: author.to_string(),
                        content: parts[1].to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn build_feed<'a, M, F>(tweets: &'a [Tweet], on_link: F) -> Column<'a, M>
where
    M: 'a,
    F: Fn(String) -> M + Copy + 'a,
{
    let mut col = column!().spacing(8);

    let mut bold = font::Font::with_name("Iosevka Aile");
    bold.weight = font::Weight::Bold;

    for tweet in tweets {
        let formatted_time = tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");

        let header = text(format!("{} - {}", tweet.author, formatted_time)).font(bold);

        let mut spans = Vec::new();

        for word in tweet.content.split_whitespace() {
            let is_link = word.starts_with("http://") || word.starts_with("https://");

            if is_link {
                spans.push(
                    span(word)
                        .link(word.to_string())
                        .underline(true)
                        .color(Color::from_rgb(0.4, 0.6, 1.0)),
                );
            } else {
                spans.push(span(word));
            }

            spans.push(span(" "));
        }

        let content = rich_text(spans).on_link_click(on_link);

        col = col.push(
            container(column![header, content].spacing(4))
                .padding(4)
                .width(Length::Fill),
        );
    }

    col
}

use bytes::Bytes;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

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

// Used for download_file
fn get_cache_path(url: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let mut path = PathBuf::from("cache");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.push(hash);
    path.set_extension("json");
    path
}

fn get_cache_paths(url: &str) -> (PathBuf, PathBuf) {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let mut dir = PathBuf::from("cache");
    let _ = std::fs::create_dir_all(&dir);

    let mut data_path = dir.clone();
    data_path.push(format!("{}.bin", hash));

    let mut meta_path = dir;
    meta_path.push(format!("{}.meta", hash));

    (data_path, meta_path)
}

pub async fn download_binary(url: String) -> Result<Bytes, String> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let (data_path, meta_path) = get_cache_paths(&url);

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

    std::fs::write(&data_path, &data).map_err(|e| e.to_string())?;
    let meta_json = serde_json::to_string(&CacheMetadata {
        etag,
        last_modified,
    })
    .map_err(|e| e.to_string())?;
    std::fs::write(&meta_path, meta_json).map_err(|e| e.to_string())?;

    Ok(data)
}

pub async fn download_file(url: String) -> Result<String, String> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let cache_path = get_cache_path(&url);

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

    // 3. 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return cached_data
            .map(|e| e.content)
            .ok_or_else(|| "Server returned 304 but no local file found".to_string());
    }

    // 4. 200 OK
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
    std::fs::write(cache_path, serialized).map_err(|e| e.to_string())?;

    Ok(content)
}
