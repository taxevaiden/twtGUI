//! Hash utilities for twtxt feeds.
//!
//! This module provides a utility function for computing tweet hashes used by the twtxt v2 specification.

use chrono::{DateTime, TimeZone, Utc};

use data_encoding::BASE32_NOPAD;

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
