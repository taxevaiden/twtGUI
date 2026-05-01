//! Shared utilities used throughout the application.
//!
//! This module provides common types and helpers.

pub mod download;
pub mod hash;
pub mod parsing;
pub mod paths;
pub mod styling;
pub mod threading;

pub use download::{download_and_parse_twtxt, download_binary};
pub use hash::compute_twt_hash;
pub use parsing::{parse_metadata, parse_tweets, parse_twt_contents};
pub use threading::build_threads;

use chrono::{DateTime, Utc};
use iced::widget::markdown;
use serde::{Deserialize, Serialize};

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

/// A simple text/URL pair used for follow lists, links, and prev entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub text: String,
    pub url: String,
}

/// Metadata extracted from the header of a twtxt feed.
///
/// This includes the feed's public-facing information (nick, avatar, description)
/// as well as the follow list, links, and refresh hints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Metadata {
    /// The URL(s) of the feed that produced the tweets.
    ///
    /// Used to detect when a feed has moved or is being redirected.
    pub urls: Vec<String>,

    /// The display name for the feed.
    pub nick: Option<String>,

    /// URL to an avatar image.
    pub avatar: Option<String>,

    /// A short description of the feed.
    pub description: Option<String>,

    /// The kind of feed (e.g. "bot", "rss") if specified.
    pub kind: Option<String>,

    /// The list of feeds this feed follows.
    pub follows: Vec<Link>,

    /// The number of people followed. Kept in sync with `follows.len()`.
    pub following: Option<u64>,

    /// Additional links to display on the profile.
    pub links: Vec<Link>,

    /// Link to an archived feed.
    pub prev: Option<Link>,

    /// Refresh interval hint (in seconds).
    pub refresh: Option<u64>,
}

/// A coherent bundle of feed data (tweets and optional metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedBundle {
    pub tweets: Vec<Tweet>,
    // Metadata could be missing since it's possible a feed could be a twtxt v1 feed
    pub metadata: Option<Metadata>,
}

/// Internal cache format used when keeping a parsed feed around.
///
/// Stores the hash of the raw content so we can skip re-parsing unchanged input.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParsedCache {
    pub content_hash: String,
    pub bundle: FeedBundle,
}
