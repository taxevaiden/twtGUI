//! Module for computing SHA-256 hashes of strings.

use hex;
use sha2::{Digest, Sha256};

/// Computes a SHA-256 hash of the provided string.
pub fn hash_sha256_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}
