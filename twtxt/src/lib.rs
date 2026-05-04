//! Twtxt utilities for parsing, hashing, and threading tweets from twtxt feeds.

use crate::metadata::Metadata;
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

pub mod file;
pub mod metadata;
pub mod parsing;
pub mod threading;
pub mod twt_hash;
pub mod url;

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
    // Blake2b hash of the feed content
    pub hash: String,
}
