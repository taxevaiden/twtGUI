use bytes::Bytes;
use chrono::Utc;
use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, image::Handle, row, scrollable, text, text_input},
};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::utils::{Tweet, download_binary, download_file, parse_metadata, parse_tweets};
use crate::{config::AppConfig, utils::build_feed};

pub struct TimelinePage {
    composer: String,
    tweets: Vec<Tweet>,
    local_avatar: Option<Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerChanged(String),
    PostPressed,
    Refresh,
    DownloadFinished {
        nick: String,
        result: Result<String, String>,
    },
    AvatarDownloadFinished {
        nick: String,
        content: String,
        result: Result<Bytes, String>,
    },
    LinkClicked(String),
    RedirectToPage(crate::app::RedirectInfo),
}

impl TimelinePage {
    pub fn new() -> Self {
        Self {
            composer: String::new(),
            tweets: Vec::new(),
            local_avatar: None,
        }
    }

    pub fn update(&mut self, message: Message, config: &AppConfig) -> Task<Message> {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
                Task::none()
            }

            Message::PostPressed => {
                self.send_tweet(config);
                Task::none()
            }

            Message::Refresh => self.refresh_timeline(config),

            Message::DownloadFinished { nick, result } => match result {
                Ok(content) => {
                    let metadata = parse_metadata(&content);

                    if let Some(avatar_url) = metadata.as_ref().and_then(|m| m.get("avatar")) {
                        return Task::perform(download_binary(avatar_url.to_string()), {
                            let nick = nick.clone();
                            let content = content.clone();
                            move |result| Message::AvatarDownloadFinished {
                                nick,
                                content,
                                result,
                            }
                        });
                    }

                    // No avatar → just parse normally
                    let fetched = parse_tweets(&nick, Handle::from_bytes(Bytes::new()), &content);
                    self.tweets.extend(fetched);
                    self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    Task::none()
                }
                Err(err) => {
                    println!("Error downloading: {}", err);
                    Task::none()
                }
            },

            Message::AvatarDownloadFinished {
                nick,
                content,
                result,
            } => {
                let avatar_bytes = match result {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        println!("Avatar download failed: {}", err);
                        Bytes::new()
                    }
                };

                let handle = Handle::from_bytes(avatar_bytes);

                if nick == config.settings.nick {
                    self.local_avatar = Some(handle.clone());
                }

                let fetched = parse_tweets(&nick, handle, &content);
                self.tweets.extend(fetched);
                self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                Task::none()
            }

            Message::LinkClicked(url) => {
                if url.contains("twtxt") && url.ends_with(".txt") {
                    Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                        page: crate::app::Page::View,
                        content: url.clone(),
                    }))
                } else {
                    // Open the URL in the default browser
                    if let Err(err) = webbrowser::open(&url) {
                        println!("Error opening URL: {}", err);
                    }
                    Task::none()
                }
            }

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
        }
    }

    fn refresh_timeline(&mut self, config: &AppConfig) -> Task<Message> {
        self.tweets.clear();

        let mut tasks = Vec::new();

        let path = Path::new(&config.settings.twtxt);

        if let Ok(content) = std::fs::read_to_string(path) {
            let metadata = parse_metadata(&content);

            if let Some(avatar_url) = metadata.as_ref().and_then(|m| m.get("avatar")) {
                // Download avatar first, then parse tweets
                tasks.push(Task::perform(download_binary(avatar_url.to_string()), {
                    let content = content.clone();
                    let nick = config.settings.nick.clone();
                    move |result| Message::AvatarDownloadFinished {
                        nick,
                        content,
                        result,
                    }
                }));
            } else {
                // No avatar → parse immediately
                let fetched = parse_tweets(
                    &config.settings.nick,
                    Handle::from_bytes(Bytes::new()),
                    &content,
                );

                self.tweets.extend(fetched);
            }
        }

        // Spawn tasks to download following twtxts
        if let Some(following) = config.following.as_ref() {
            for (key, value) in following {
                tasks.push(Task::perform(download_file(value.to_string()), {
                    let key = key.clone();
                    move |result| Message::DownloadFinished { nick: key, result }
                }));
            }
        }

        self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Task::batch(tasks)
    }

    fn send_tweet(&mut self, config: &AppConfig) {
        if self.composer.trim().is_empty() {
            return;
        }

        let now = Utc::now();

        let avatar = self
            .local_avatar
            .clone()
            .unwrap_or_else(|| Handle::from_bytes(Bytes::new()));

        self.tweets.insert(
            0,
            Tweet {
                timestamp: now,
                author: config.settings.nick.clone(),
                avatar,
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
        let timeline = build_feed(&self.tweets, Message::LinkClicked);

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
