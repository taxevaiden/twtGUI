use chrono::{DateTime, Local, Utc};
use iced::{
    Length,
    widget::{Column, button, column, container, row, scrollable, text, text_input},
};

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

pub fn build_timeline<'a, M>(tweets: &'a [Tweet]) -> Column<'a, M>
where
    M: 'a,
{
    let mut col = column!().spacing(2);

    for tweet in tweets {
        let formatted = format!(
            "{} {}: {}",
            tweet
                .timestamp
                .with_timezone(&Local)
                .format("%m/%d/%Y %-I:%M %p"),
            tweet.author,
            tweet.content
        );

        col = col.push(container(text(formatted)).padding(4).width(Length::Fill));
    }

    col
}
