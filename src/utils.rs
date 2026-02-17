use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Tweet {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub content: String,
}

pub fn parse_twtxt(author: &str, input: &str) -> Vec<Tweet> {
    input
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() == 2 {
                Some(Tweet {
                    timestamp: DateTime::parse_from_rfc3339(parts[0]).ok()?.to_utc(),
                    author: author.to_string(),
                    content: parts[1].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}
