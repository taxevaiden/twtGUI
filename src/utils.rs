use chrono::{DateTime, Local, Utc};
use data_encoding::BASE32_NOPAD;
use iced::{
    Color, Length, font,
    widget::{Column, Image, column, container, image::Handle, rich_text, row, space, span},
};
use regex;

use bytes::Bytes;
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub hash: String,
    pub reply_to: Option<String>, // reply is a tweet hash, defined by something like (#abc1234) at the beginning of the tweet
    pub mentions: Vec<OptLink>,
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub author: String,
    pub content: String,

    #[serde(skip, default = "default_avatar")]
    pub avatar: Handle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptLink {
    pub text: Option<String>,
    pub url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub urls: Vec<String>, // the url(s) of the feed. if the url(s) do not match the url that the user entered in the ViewPage, we should warn the user. we don't redirect them for security reasons
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>, // `type` field, could be "bot" or "rss" (if empty, we assume it's a human managed account)
    pub follows: Vec<Link>,
    pub following: Option<u64>, // number of people they follow; this isn't really needed since we can just do follows.len()
    pub links: Vec<Link>, // urls on their profile (ex. My Github Page: https://github.com/username)
    pub prev: Vec<String>,
    pub refresh: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedBundle {
    pub tweets: Vec<Tweet>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParsedCache {
    content_hash: String,
    bundle: FeedBundle,
}

fn default_avatar() -> Handle {
    Handle::from_path("assets/default_avatar.png")
}

fn hash_sha256_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
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

pub fn parse_twt_contents(raw_content: &str) -> (Option<String>, Vec<OptLink>, String) {
    let mention_re = regex::Regex::new(r"@<(?P<first>[^\s>]+)(?:\s+(?P<second>[^>]+))?>").unwrap();
    let subject_re = regex::Regex::new(r"^\(#(?P<hash>[^)]+)\)").unwrap();

    let mut reply_to = None;
    let mut mentions = Vec::new();
    let mut display_content = String::new();
    let mut current_pos = 0;
    let mut subject_found = false;

    // prefix pass (hashes, first couple mentions)
    while current_pos < raw_content.len() {
        let remaining = &raw_content[current_pos..];
        let trimmed = remaining.trim_start();
        current_pos += remaining.len() - trimmed.len();

        let fragment = &raw_content[current_pos..];
        if fragment.is_empty() {
            break;
        }

        // try matching a mention at the current start
        if let Some(cap) = mention_re.captures(fragment) {
            let first = cap.name("first").map(|m| m.as_str()).unwrap();
            let second = cap.name("second").map(|m| m.as_str());

            mentions.push(OptLink {
                text: second.map(|_| first.trim().to_string()),
                url: second.unwrap_or(first).trim().to_string(),
            });

            if second.is_some() {
                display_content.push_str(&format!("@{} ", first));
            } else {
                display_content.push_str(&format!("{} ", first));
            }

            current_pos += cap.get(0).unwrap().end();
            continue;
        }

        // try matching hash at the current start
        if !subject_found {
            if let Some(cap) = subject_re.captures(fragment) {
                reply_to = Some(cap.name("hash").unwrap().as_str().to_string());
                subject_found = true;
                current_pos += cap.get(0).unwrap().end();
                continue;
            }
        }

        break;
    }

    let body = &raw_content[current_pos..];
    let mut last_end = 0;

    // second pass (all mentions throughout body)
    for cap in mention_re.captures_iter(body) {
        let whole_match = cap.get(0).unwrap();
        display_content.push_str(&body[last_end..whole_match.start()]);

        let first = cap.name("first").map(|m| m.as_str()).unwrap();
        let second = cap.name("second").map(|m| m.as_str());

        mentions.push(OptLink {
            text: second.map(|_| first.trim().to_string()),
            url: second.unwrap_or(first).trim().to_string(),
        });

        if second.is_some() {
            display_content.push_str(&format!("@{}", first));
        } else {
            display_content.push_str(first);
        }

        last_end = whole_match.end();
    }
    display_content.push_str(&body[last_end..]);

    (reply_to, mentions, display_content.trim().to_string())
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
                    metadata.follows.push(Link {
                        text: nick.trim().to_string(),
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

            _ => {}
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
    let author_name = author.to_string();

    input
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (timestamp_str, raw_content) = line.split_once('\t')?;
            let (reply_to, mentions, display_content) = parse_twt_contents(raw_content);

            Some(Tweet {
                hash: compute_twt_hash(url, timestamp_str, raw_content),
                reply_to,
                mentions,
                timestamp: DateTime::parse_from_rfc3339(timestamp_str)
                    .ok()?
                    .with_timezone(&Utc),
                author: author_name.clone(),
                url: url.to_string(),
                avatar: avatar
                    .clone()
                    .unwrap_or_else(|| Handle::from_path("assets/default_avatar.png")),
                // Use the cleaned version for the UI
                content: display_content.trim().to_string(),
            })
        })
        .collect()
}

pub fn build_feed<'a, M, F>(tweets: &'a [Tweet], on_link: F) -> Column<'a, M>
where
    M: 'a,
    F: Fn(String) -> M + Copy + 'a,
{
    use std::collections::HashMap;

    let mut col = column!().spacing(8);

    let mut bold = font::Font::with_name("Iosevka Aile");
    bold.weight = font::Weight::Bold;

    // Build lookup map
    let mut map: HashMap<&str, &Tweet> = HashMap::new();
    for tweet in tweets {
        map.insert(&tweet.hash, tweet);
    }

    for tweet in tweets {
        let formatted_time = tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");

        let header = rich_text![
            span(&tweet.author).font(bold).link(tweet.url.clone()),
            span(" - "),
            span(formatted_time.to_string()),
            span(" "),
            span(tweet.hash.clone())
        ]
        .on_link_click(on_link);

        let mut spans = Vec::new();

        for word in tweet.content.split_whitespace() {
            let is_link = word.starts_with("http://") || word.starts_with("https://");
            let is_mention = word.starts_with("@");

            if is_link {
                spans.push(
                    span(word)
                        .link(word.to_string())
                        .color(Color::from_rgb(0.4, 0.6, 1.0)),
                );
                spans.push(span(" "));

                continue;
            }

            if is_mention {
                let mention_str = word.trim_start_matches('@');
                for mention in &tweet.mentions {
                    if let Some(word) = mention.text.clone() {
                        if word == mention_str {
                            spans.push(
                                span(format!("@{}", word))
                                    .link(mention.url.clone())
                                    .color(Color::from_rgb(0.4, 0.6, 1.0)),
                            );
                            spans.push(span(" "));
                        }
                    } else {
                        if mention.url.clone() == mention_str {
                            spans.push(
                                span(format!("@{}", mention.url.clone()))
                                    .link(mention.url.clone())
                                    .color(Color::from_rgb(0.4, 0.6, 1.0)),
                            );
                            spans.push(span(" "));
                        }
                    }
                }

                continue;
            }

            spans.push(span(word));
            spans.push(span(" "));
        }

        let content = rich_text(spans).on_link_click(on_link);

        let avatar_img = Image::new(tweet.avatar.clone())
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .border_radius(20);

        if let Some(reply) = tweet.reply_to.as_ref() {
            if let Some(reply_twt) = map.get(reply.as_str()) {
                let reply_author = reply_twt.author.clone();
                let reply_content = reply_twt.content.clone();
                col = col.push(
                    column![
                        row![
                            space().width(64),
                            rich_text![
                                span("Reply to "),
                                span(reply_author).font(bold).link(reply_twt.url.clone()),
                                span(": "),
                                span(reply_content)
                            ]
                            .on_link_click(on_link)
                        ],
                        row![avatar_img, column![header, content].spacing(4).padding(4)].spacing(6),
                    ]
                    .padding(4)
                    .width(Length::Fill),
                );
            } else {
                col = col.push(
                    container(
                        row![avatar_img, column![header, content].spacing(4).padding(4)].spacing(6),
                    )
                    .padding(4)
                    .width(Length::Fill),
                );
            }
        } else {
            col = col.push(
                container(
                    row![avatar_img, column![header, content].spacing(4).padding(4)].spacing(6),
                )
                .padding(4)
                .width(Length::Fill),
            );
        }
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
fn get_txt_cache_path(url: &str) -> PathBuf {
    let hash = hash_sha256_str(url);

    let mut path = PathBuf::from("cache");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path.push(hash);
    path.set_extension("json");
    path
}

// Used for download_binary
fn get_bin_cache_paths(url: &str) -> (PathBuf, PathBuf) {
    let hash = hash_sha256_str(url);

    let dir = PathBuf::from("cache");
    let _ = std::fs::create_dir_all(&dir);

    let mut data_path = dir.clone();
    data_path.push(format!("{}.bin", hash));

    let mut meta_path = dir;
    meta_path.push(format!("{}.meta", hash));

    (data_path, meta_path)
}

fn get_parsed_cache_path(url: &str) -> PathBuf {
    let hash = hash_sha256_str(url);

    let mut path = PathBuf::from("cache");
    let _ = std::fs::create_dir_all(&path);
    path.push(format!("{}.parsed.json", hash));
    path
}

pub async fn download_binary(url: String) -> Result<Bytes, String> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let (data_path, meta_path) = get_bin_cache_paths(&url);

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

pub async fn download_twtxt(url: String) -> Result<String, String> {
    println!("Downloading twtxt.txt from {}", url);
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let cache_path = get_txt_cache_path(&url);

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
        println!("304 Not Modified: {}", url);
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

    println!("200 OK: {}", url);

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

// use_nick determines whether the nick provided should be used as the actual display name, or just a fallback if there is no nick in the metadata
// this nick should NOT be in the cache, only the nick provided by the feed's metadata
pub async fn download_and_parse_twtxt(
    nick: String,
    url: String,
    use_nick: bool,
) -> Result<FeedBundle, String> {
    let raw = download_twtxt(url.clone()).await?;
    let raw_hash = hash_sha256_str(&raw);
    let parsed_path = get_parsed_cache_path(&url);

    if let Ok(cached_str) = std::fs::read_to_string(&parsed_path) {
        if let Ok(cache) = serde_json::from_str::<ParsedCache>(&cached_str) {
            if cache.content_hash == raw_hash {
                return Ok(apply_nick_override(cache.bundle, &nick, use_nick));
            }
        }
    }

    let metadata = parse_metadata(&raw);

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

    let tweets = parse_tweets(&canonical_nick, &url, None, &raw);

    let canonical_bundle = FeedBundle { tweets, metadata };

    let cache = ParsedCache {
        content_hash: raw_hash,
        bundle: canonical_bundle.clone(),
    };

    let _ = std::fs::write(parsed_path, serde_json::to_string(&cache).unwrap());

    Ok(apply_nick_override(canonical_bundle, &nick, use_nick))
}

fn apply_nick_override(mut bundle: FeedBundle, nick: &str, use_nick: bool) -> FeedBundle {
    if use_nick {
        for tweet in &mut bundle.tweets {
            tweet.author = nick.to_string();
        }
    }

    bundle
}
