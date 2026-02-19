use std::collections::HashMap;

use bytes::Bytes;
use iced::{
    Element, Length, Task,
    widget::{
        button, column,
        image::{self, Handle},
        row, scrollable, text, text_input,
    },
};

use crate::utils::{
    Tweet, build_feed, download_binary, download_file, parse_metadata, parse_tweets,
};
use crate::{app::RedirectInfo, config::AppConfig};

pub struct ViewPage {
    composer: String,
    avatar_bytes: Bytes,
    fetched: String,
    tweets: Vec<Tweet>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerChanged(String),
    ViewPressed,
    FeedDownloadFinished(Result<String, String>),
    AvatarDownloadFinished(Result<Bytes, String>),
    LinkClicked(String),
}

impl ViewPage {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            composer: config.settings.twturl.clone(),
            avatar_bytes: Bytes::new(),
            fetched: String::new(),
            tweets: Vec::new(),
            metadata: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
                Task::none()
            }

            Message::ViewPressed => {
                self.tweets.clear();
                self.avatar_bytes = Bytes::new();
                self.fetched.clear();

                Task::perform(
                    download_file(self.composer.clone()),
                    Message::FeedDownloadFinished,
                )
            }

            Message::FeedDownloadFinished(Ok(data)) => {
                self.fetched = data;
                let data = &self.fetched;

                self.metadata = parse_metadata(data);

                if let Some(avatar) = self.metadata.as_ref().and_then(|m| m.get("avatar")) {
                    // Download avatar first
                    return Task::perform(
                        download_binary(avatar.to_string()),
                        Message::AvatarDownloadFinished,
                    );
                }

                self.build_tweets();
                Task::none()
            }

            Message::FeedDownloadFinished(Err(e)) => {
                self.fetched = format!("Error: {}", e);
                println!("{}", e);
                Task::none()
            }

            Message::AvatarDownloadFinished(Ok(data)) => {
                self.avatar_bytes = data;
                self.build_tweets();
                Task::none()
            }

            Message::AvatarDownloadFinished(Err(_)) => {
                self.build_tweets();
                Task::none()
            }

            Message::LinkClicked(url) => {
                if url.contains("twtxt") && url.ends_with(".txt") {
                    self.process_redirect_info(RedirectInfo {
                        page: crate::app::Page::View,
                        content: url.clone(),
                    })
                } else {
                    // Open the URL in the default browser
                    if let Err(err) = webbrowser::open(&url) {
                        println!("Error opening URL: {}", err);
                    }
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let image = image::Image::new(Handle::from_bytes(self.avatar_bytes.clone()));
        let timeline = build_feed(&self.tweets, Message::LinkClicked);
        let mut col = column!().spacing(8);

        if let Some(metadata) = &self.metadata {
            for (name, value) in metadata {
                col = col.push(text(format!("{}: {}", name, value)));
            }
        }

        let scroll = scrollable(column![col, image, timeline]).height(iced::Length::Fill);

        let composer = row![
            text_input("https://example.com/twtxt.txt", &self.composer)
                .on_input(Message::ComposerChanged)
                .padding(8),
            button("View")
                .on_press(Message::ViewPressed)
                .padding([8, 16]),
        ]
        .spacing(8);

        column![composer, scroll]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8)
            .into()
    }

    pub fn process_redirect_info(&mut self, info: crate::app::RedirectInfo) -> Task<Message> {
        match info.page {
            crate::app::Page::View => {
                self.composer = info.content;
                Task::done(Message::ViewPressed)
            }
            _ => Task::none(),
        }
    }

    fn build_tweets(&mut self) {
        let data = &self.fetched;

        let nick = self
            .metadata
            .as_ref()
            .and_then(|m| m.get("nick"))
            .cloned()
            .unwrap_or_else(|| {
                url::Url::parse(&self.composer)
                    .ok()
                    .and_then(|url| url.host_str().map(str::to_string))
                    .unwrap_or_else(|| "unknown".to_string())
            });

        let handle = Handle::from_bytes(self.avatar_bytes.clone());

        self.tweets = parse_tweets(&nick, handle, data);
        self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }
}
