use chrono::{Local, Utc};
use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, row, scrollable, text, text_input},
};
use std::io::Write;
use std::path::Path;
use std::fs::OpenOptions;

use crate::config::AppConfig;
use crate::utils::{Tweet, parse_twtxt};

pub struct TimelinePage {
    config: AppConfig,
    composer: String,
    tweets: Vec<Tweet>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerChanged(String),
    PostPressed,
    Refresh,
}

impl TimelinePage {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: config.clone(),
            composer: String::new(),
            tweets: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
            }

            Message::PostPressed => {
                self.send_tweet();
            }

            Message::Refresh => {
                self.refresh_timeline();
            }
        }
    }

    fn refresh_timeline(&mut self) {
        self.tweets.clear();

        let path = Path::new(&self.config.settings.twtxt);
        // let file = File::open(&path).unwrap();
        // let reader = io::BufReader::new(file);

        // for line in reader.lines().flatten() {
        //     let parts: Vec<&str> = line.splitn(2, '\t').collect();
        //     if parts.len() == 2 {
        //         if let Ok(ts) = DateTime::parse_from_rfc3339(parts[0]) {
        //             self.tweets.push(Tweet {
        //                 timestamp: ts.with_timezone(&Utc),
        //                 author: self.config.settings.nick.clone(),
        //                 content: parts[1].to_string(),
        //             });
        //         }
        //     }
        // }
        //
        if let Ok(content) = std::fs::read_to_string(path) {
            self.tweets =
                parse_twtxt(self.config.settings.nick.clone().as_str(), content.as_str()).clone();

            self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
    }

    fn send_tweet(&mut self) {
        if self.composer.trim().is_empty() {
            return;
        }

        let now = Utc::now();

        self.tweets.insert(
            0,
            Tweet {
                timestamp: now,
                author: self.config.settings.nick.clone(),
                content: self.composer.clone(),
            },
        );

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.config.settings.twtxt)
            .unwrap();

        writeln!(file, "{}\t{}", now.to_rfc3339(), self.composer).ok();

        self.composer.clear();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let timeline = self.tweets.iter().fold(column!().spacing(2), |col, tweet| {
            let formatted = format!(
                "{} {}: {}",
                tweet
                    .timestamp
                    .with_timezone(&Local)
                    .format("%m/%d/%Y%l:%M %p"),
                tweet.author,
                tweet.content
            );

            col.push(container(text(formatted)).padding(4).width(Length::Fill))
        });

        let scroll = scrollable(timeline).height(iced::Length::Fill);

        let composer = row![
            text_input("What's on your mind?", &self.composer)
                .on_input(Message::ComposerChanged)
                .padding(8),
            button("Post")
                .on_press(Message::PostPressed)
                .padding([8, 16]),
        ]
        .spacing(8);

        let refresh_button = button(
            text("Refresh")
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .on_press(Message::Refresh)
        .width(Length::Fill)
        .padding([8, 16]);

        column![composer, scroll, refresh_button]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8)
            .into()
    }
}
