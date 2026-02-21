use chrono::{DateTime, Local, Utc};
use data_encoding::BASE32_NOPAD;
use iced::{
    Color, Length, font,
    widget::{Column, Image, column, container, image::Handle, rich_text, row, span},
};

use bytes::Bytes;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Tweet {
    pub hash: String,
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub author: String,
    pub avatar: Handle,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Follow {
    pub nick: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub text: String,
    pub url: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Metadata {
    pub urls: Vec<String>, // the url(s) of the feed. if the url(s) do not match the url that the user entered in the ViewPage, we should warn the user. we don't redirect them for security reasons
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>, // `type` field, could be "bot" or "rss" (if empty, we assume it's a human managed account)
    pub follows: Vec<Follow>,
    pub following: Option<u64>, // number of people they follow; this isn't really needed since we can just do follows.len()
    pub links: Vec<Link>, // urls on their profile (ex. My Github Page: https://github.com/username)
    pub prev: Vec<String>,
    pub refresh: Option<u64>,
}

// timestamp must be formatted as RFC3339, with the time truncated/expanded to seconds precision
// it also has to be formatted using the Zulu indicator (Z)

pub fn compute_twt_hash(feed_url: &str, timestamp: &str, text: &str) -> String {
    use blake2::{
        Blake2bVar,
        digest::{Update, VariableOutput},
    };

    let payload = format!("{feed_url}\n{timestamp}\n{text}");

    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(payload.as_bytes());
    let mut result = vec![0u8; 32];
    hasher.finalize_variable(&mut result).unwrap();

    let encoded = BASE32_NOPAD.encode(&result).to_lowercase();

    encoded
        .chars()
        .rev()
        .take(7)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

pub fn parse_metadata(input: &str) -> Option<Metadata> {
    let mut metadata = Metadata::default();

    for line in input.lines() {
        let Some(stripped) = line.strip_prefix('#') else {
            continue;
        };
        let Some((key, value)) = stripped.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim();

        match key {
            "url" => metadata.urls.push(value.to_string()),

            "nick" => metadata.nick = Some(value.to_string()),

            "avatar" => metadata.avatar = Some(value.to_string()),

            "description" => metadata.description = Some(value.to_string()),

            "type" => metadata.kind = Some(value.to_string()),

            "follow" => {
                // format: nick url
                if let Some((nick, url)) = value.rsplit_once(' ') {
                    metadata.follows.push(Follow {
                        nick: nick.trim().to_string(),
                        url: url.trim().to_string(),
                    });
                }
            }

            "following" => {
                if let Ok(num) = value.parse::<u64>() {
                    metadata.following = Some(num);
                }
            }

            "link" => {
                // format: text url
                if let Some((text, url)) = value.rsplit_once(' ') {
                    metadata.links.push(Link {
                        text: text.trim().to_string(),
                        url: url.trim().to_string(),
                    });
                }
            }

            "prev" => metadata.prev.push(value.to_string()),

            "refresh" => {
                if let Ok(num) = value.parse::<u64>() {
                    metadata.refresh = Some(num);
                }
            }

            _ => {
                // Ignore unknown keys (future extensions)
            }
        }
    }

    // Return None if nothing was actually parsed
    if metadata == Metadata::default() {
        None
    } else {
        Some(metadata)
    }
}

pub fn parse_tweets(author: &str, url: &str, avatar: Option<Handle>, input: &str) -> Vec<Tweet> {
    let author = author.to_string();

    input
        .lines()
        .filter_map(|line| {
            if line.starts_with('#') {
                return None;
            }

            let (timestamp, content) = line.split_once('\t')?;

            Some(Tweet {
                // we should use the url specified in the feed's metadata instead of using the one provided,
                // as it may be different from the one provided (and one slight difference can lead to a *completely different* hash!)
                // for now though, we can just assume it's the same
                hash: compute_twt_hash(&url, &timestamp, &content),
                timestamp: DateTime::parse_from_rfc3339(timestamp).ok()?.to_utc(),
                author: author.clone(),
                url: url.to_string(),
                avatar: avatar
                    .clone()
                    .unwrap_or(Handle::from_path("assets/default_avatar.png")),
                content: content.to_string(),
            })
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

        let header = rich_text![
            span(&tweet.author).font(bold).link(tweet.url.clone()),
            span(" - "),
            span(formatted_time.to_string()),
            // just for debugging
            span(" "),
            span(tweet.hash.clone())
        ]
        .on_link_click(on_link);

        let mut spans = Vec::new();

        for word in tweet.content.split_whitespace() {
            let is_link = word.starts_with("http://") || word.starts_with("https://");

            if is_link {
                spans.push(
                    span(word)
                        .link(word.to_string())
                        .color(Color::from_rgb(0.4, 0.6, 1.0)),
                );
            } else {
                spans.push(span(word));
            }

            spans.push(span(" "));
        }

        let content = rich_text(spans).on_link_click(on_link);
        let avatar_img = Image::new(tweet.avatar.clone())
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .border_radius(20);

        col = col.push(
            container(row![avatar_img, column![header, content].spacing(4).padding(4)].spacing(6))
                .padding(4)
                .width(Length::Fill),
        );
    }

    col
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

// Used for download_binary
fn get_cache_paths(url: &str) -> (PathBuf, PathBuf) {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let dir = PathBuf::from("cache");
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

    // 304 Not Modified
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
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
    std::fs::write(cache_path, serialized).map_err(|e| e.to_string())?;

    Ok(content)
}
