use chrono::{Local, Utc};
use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, row, scrollable, text, text_input},
};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::utils::{Tweet, parse_twtxt};
use crate::{config::AppConfig, utils::build_timeline};

pub struct TimelinePage {
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
    pub fn new() -> Self {
        Self {
            composer: String::new(),
            tweets: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message, config: &AppConfig) {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
            }

            Message::PostPressed => {
                self.send_tweet(config);
            }

            Message::Refresh => {
                self.refresh_timeline(config);
            }
        }
    }

    fn refresh_timeline(&mut self, config: &AppConfig) {
        self.tweets.clear();

        let path = Path::new(&config.settings.twtxt);
        if let Ok(content) = std::fs::read_to_string(path) {
            self.tweets = parse_twtxt(&config.settings.nick.as_str(), content.as_str()).clone();

            self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
    }

    fn send_tweet(&mut self, config: &AppConfig) {
        if self.composer.trim().is_empty() {
            return;
        }

        let now = Utc::now();

        self.tweets.insert(
            0,
            Tweet {
                timestamp: now,
                author: config.settings.nick.clone(),
                content: self.composer.clone(),
            },
        );

        let mut file = OpenOptions::new()
            .append(true)
            .open(&config.settings.twtxt)
            .unwrap();

        writeln!(file, "{}\t{}", now.to_rfc3339(), self.composer).ok();

        self.composer.clear();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let timeline = build_timeline(&self.tweets);

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
