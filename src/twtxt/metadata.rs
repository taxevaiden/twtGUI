//! Module providing a struct for twtxt metadata.

use serde::{Deserialize, Serialize};

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
