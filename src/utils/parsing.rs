//! Parsing utilities for twtxt feeds.
//!
//! This module focuses on parsing the raw twtxt text into structured objects.

use crate::utils::{Link, Metadata, Tweet, TweetNode};
use chrono::{DateTime, TimeZone, Utc};
use data_encoding::BASE32_NOPAD;
use iced::widget::{image::Handle, markdown};
use regex::Regex;
use std::collections::HashMap;

/// Computes the canonical tweet hash used by the twtxt protocol.
///
/// The hash is computed from the feed URL, timestamp, and tweet text.
///
/// The timestamp must be RFC3339 formatted with a Zulu indicator (`Z`), and must be truncated to the second.
///
/// Any slight change will significantly change the resulting hash.
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

    // https://twtxt.dev/exts/twt-hash-v2.html
    // Tweets after 2026-07-01T00:00:00Z use the v2 hash format.
    // Before this date, the v1 hash format is used.
    let ts = timestamp.parse::<DateTime<Utc>>().unwrap();
    let epoch = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();

    let use_v2_hash = ts >= epoch;

    // v2 uses the first 12 letters, whereas v1 uses the last 7 letters.
    if use_v2_hash {
        encoded.chars().take(12).collect::<String>()
    } else {
        encoded
            .chars()
            .rev()
            .take(7)
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    }
}

/// Parses a single tweet line, extracting a reply hash and converting mentions to markdown.
///
/// Returns a tuple of `(reply_to_hash, markdown_content)`.
pub fn parse_twt_contents(raw_content: &str) -> (Option<String>, String) {
    let subject_re = Regex::new(r"^\(#(?P<hash>[a-z0-9]{7})\)\s*").unwrap();
    let mention_re = Regex::new(r"@<(?P<nick>[^\s>]+)(?:\s+(?P<url>[^>]+))?>").unwrap();

    let mut reply_to = None;
    let mut content = raw_content;

    if let Some(cap) = subject_re.captures(content) {
        reply_to = Some(cap["hash"].to_string());
        let end = cap.get(0).unwrap().end();
        content = &content[end..];
    }

    let mut last_end = 0;
    let mut markdown_content = String::new();

    for cap in mention_re.captures_iter(content) {
        let m = cap.get(0).unwrap();

        markdown_content.push_str(&content[last_end..m.start()]);

        let nick = cap.name("nick").unwrap().as_str();
        let url = cap.name("url").map(|m| m.as_str()).unwrap_or(nick);

        markdown_content.push_str(&format!("[@{}]({})", nick, url));

        last_end = m.end();
    }

    markdown_content.push_str(&content[last_end..]);

    let final_markdown = markdown_content
        .replace("\\u2028", "  \n")
        .trim()
        .to_string();

    (reply_to, final_markdown)
}

/// Parses twtxt metadata headers (`# key = value`) from the given input.
///
/// Returns `None` if no valid metadata fields were found.
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

            "prev" => {
                // format: text url
                if let Some((text, url)) = value.rsplit_once(' ') {
                    metadata.prev.push(Link {
                        text: text.trim().to_string(),
                        url: url.trim().to_string(),
                    });
                }
            }

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

/// Parses a twtxt feed into a list of `Tweet` objects.
///
/// `author` is the display name to assign to each tweet, and `url` is the
/// canonical feed URL used for hash computation.
pub fn parse_tweets(author: &str, url: &str, avatar: Option<Handle>, input: &str) -> Vec<Tweet> {
    let author_name = author.to_string();

    input
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (timestamp_str, raw_content) = line.split_once('\t')?;
            let (reply_to, display_content) = parse_twt_contents(raw_content);
            let items = markdown::parse(&display_content).collect();

            Some(Tweet {
                hash: compute_twt_hash(url, timestamp_str, raw_content),
                reply_to,
                timestamp: DateTime::parse_from_rfc3339(timestamp_str)
                    .ok()?
                    .with_timezone(&Utc),
                author: author_name.clone(),
                url: url.to_string(),
                avatar: avatar
                    .clone()
                    .unwrap_or_else(|| Handle::from_path("assets/default_avatar.png")),
                content: display_content,
                md_items: items,
            })
        })
        .collect()
}

/// Builds a tree of tweet replies for rendering threaded conversations.
///
/// Returns a list of `TweetNode`.
pub fn build_threads(tweets: &[Tweet]) -> Vec<TweetNode> {
    let mut children_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();

    let all_hashes: std::collections::HashSet<&str> =
        tweets.iter().map(|t| t.hash.as_str()).collect();

    for (index, tweet) in tweets.iter().enumerate() {
        let is_reply = tweet
            .reply_to
            .as_ref()
            .map(|parent| all_hashes.contains(parent.as_str()))
            .unwrap_or(false);

        if !is_reply {
            roots.push(index);
        } else {
            let parent_hash = tweet.reply_to.as_ref().unwrap();
            children_map
                .entry(parent_hash.clone())
                .or_default()
                .push(index);
        }
    }

    roots
        .into_iter()
        .map(|root_index| construct_node(root_index, tweets, &children_map))
        .collect()
}

/// Recursively builds a `TweetNode` and its descendants.
fn construct_node(
    index: usize,
    tweets: &[Tweet],
    children_map: &HashMap<String, Vec<usize>>,
) -> TweetNode {
    let children = children_map
        .get(&tweets[index].hash)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|child_index| construct_node(child_index, tweets, children_map))
        .collect();

    TweetNode { index, children }
}
