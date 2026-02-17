use chrono::{DateTime, Local, Utc};
use iced::{
    Length, font,
    widget::{Column, column, container, text},
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
    let mut col = column!().spacing(8);
    let mut bold = font::Font::with_name("Iosevka Aile");
    bold.weight = font::Weight::Bold;

    for tweet in tweets {
        let formatted_time = tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");
        let formatted_header = text(format!("{} - {}", tweet.author, formatted_time)).font(bold);
        let formatted_content = text(tweet.content.clone());

        col = col.push(
            container(column![formatted_header, formatted_content].spacing(4))
                .padding(4)
                .width(Length::Fill),
        );
    }

    col
}

pub async fn download_file(url: String) -> Result<String, String> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}
