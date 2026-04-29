//! Parsing utilities for twtxt feeds.
//!
//! This module focuses on parsing the raw twtxt text.

use crate::utils::{Link, Metadata, Tweet, compute_twt_hash, hash::hash_sha256_str};
use chrono::{DateTime, Utc};
use iced::widget::markdown;
use regex::Regex;

/// Parses a single tweet line, extracting a reply hash and converting mentions to markdown.
///
/// Returns a tuple of `(reply_to_hash, markdown_content)`.
pub fn parse_twt_contents(raw_content: &str) -> (Option<String>, String) {
    let subject_re = Regex::new(r"^\(#(?P<hash>[a-z0-9]{7,12})\)\s*").unwrap();
    let mention_re = Regex::new(r"@<(?P<nick>[^\s>]+)(?:\s+(?P<url>[^>]+))?>").unwrap();

    let mut reply_to = None;
    let mut content = raw_content;

    // Check for a subject hash prefix (e.g. `(#abc123)`) and extract it as the reply hash
    if let Some(cap) = subject_re.captures(content) {
        reply_to = Some(cap["hash"].to_string());
        let end = cap.get(0).unwrap().end();
        content = &content[end..];
    }

    let mut last_end = 0;
    let mut markdown_content = String::new();

    // We iterate over mentions captured in the content
    // and replace them with hyperlinks in markdown
    for cap in mention_re.captures_iter(content) {
        let m = cap.get(0).unwrap();

        markdown_content.push_str(&content[last_end..m.start()]);

        let nick = cap.name("nick").unwrap().as_str();
        let url = cap.name("url").map(|m| m.as_str()).unwrap_or(nick);

        markdown_content.push_str(&format!("[@{}]({})", nick, url));

        last_end = m.end();
    }

    markdown_content.push_str(&content[last_end..]);

    // Multiline extension
    // \u2028 is stored as its literal characters (e.g. hello\u2028world)
    // so we can just replace it with a newline in markdown
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
pub fn parse_tweets(author: &str, url: &str, input: &str) -> Vec<Tweet> {
    let author_name = author.to_string();
    let feed_hash = hash_sha256_str(input);

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
                feed_hash: feed_hash.clone(),
                author: author_name.clone(),
                url: url.to_string(),
                content: display_content,
                md_items: items,
            })
        })
        .collect()
}
