//! Threading utilities for twtxt feeds.
//!
//! This module focuses on building a tree of tweet replies for rendering threaded conversations.

use std::collections::HashMap;

use crate::utils::{Tweet, TweetNode};

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
