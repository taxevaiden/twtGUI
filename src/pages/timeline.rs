use bytes::Bytes;
use chrono::Utc;
use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, image::Handle, row, text, text_input},
};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::components::feed::{self, VirtualTimeline};
use crate::config::AppConfig;
use crate::utils::{
    Tweet, compute_twt_hash, download_binary, download_twtxt, parse_metadata, parse_tweets,
    parse_twt_contents,
};

pub struct TimelinePage {
    composer: String,
    tweets: Vec<Tweet>,
    local_avatar: Option<Handle>,
    pending_downloads: usize,
    feed: VirtualTimeline,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerChanged(String),
    PostPressed,
    Refresh,
    DownloadFinished {
        nick: String,
        url: String,
        result: Result<String, String>,
    },
    AvatarDownloadFinished {
        nick: String,
        url: String,
        content: String,
        result: Result<Bytes, String>,
    },
    RedirectToPage(crate::app::RedirectInfo),
    Feed(feed::Message),
}

impl TimelinePage {
    pub fn new() -> Self {
        Self {
            composer: String::new(),
            tweets: Vec::new(),
            local_avatar: None,
            pending_downloads: 0,
            feed: VirtualTimeline::new(0),
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

            Message::DownloadFinished { nick, url, result } => match result {
                Ok(content) => {
                    self.pending_downloads -= 1;

                    if let Some(metadata) = parse_metadata(&content) {
                        if let Some(avatar_url) = metadata.avatar {
                            self.pending_downloads += 1;
                            return Task::perform(download_binary(avatar_url.to_string()), {
                                let nick = nick.clone();
                                let content = content.clone();
                                let url = url.clone();
                                move |result| Message::AvatarDownloadFinished {
                                    nick,
                                    url,
                                    content,
                                    result,
                                }
                            });
                        }
                    }

                    // No avatar → just parse normally
                    let fetched = parse_tweets(&nick, &url, None, &content);
                    self.tweets.extend(fetched);
                    if self.pending_downloads == 0 {
                        self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        self.feed.reset(self.tweets.len());
                    }
                    Task::none()
                }
                Err(err) => {
                    self.pending_downloads -= 1;
                    println!("Error downloading: {}", err);
                    Task::none()
                }
            },

            Message::AvatarDownloadFinished {
                nick,
                content,
                url,
                result,
            } => {
                self.pending_downloads -= 1;
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

                let fetched = parse_tweets(&nick, &url, Some(handle), &content);
                self.tweets.extend(fetched);
                if self.pending_downloads == 0 {
                    self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    self.feed.reset(self.tweets.len());
                }

                Task::none()
            }

            Message::Feed(feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(msg) => self.feed.update(msg, self.tweets.len()).map(Message::Feed),

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
        }
    }

    fn refresh_timeline(&mut self, config: &AppConfig) -> Task<Message> {
        self.tweets.clear();
        self.feed.reset(0);
        self.pending_downloads = 0;

        let mut tasks = Vec::new();

        let path = Path::new(&config.settings.twtxt);

        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(metadata) = parse_metadata(&content) {
                if let Some(avatar_url) = metadata.avatar {
                    // Download avatar first, then parse tweets
                    tasks.push(Task::perform(download_binary(avatar_url), {
                        let content = content.clone();
                        let nick = config.settings.nick.clone();
                        let url = config.settings.twturl.clone();
                        move |result| Message::AvatarDownloadFinished {
                            nick,
                            url,
                            content,
                            result,
                        }
                    }));
                    self.pending_downloads += 1;
                } else {
                    // No avatar → parse immediately
                    let fetched = parse_tweets(
                        &config.settings.nick,
                        &config.settings.twturl,
                        None,
                        &content,
                    );

                    self.tweets.extend(fetched);
                }
            } else {
                // No avatar → parse immediately
                let fetched = parse_tweets(
                    &config.settings.nick,
                    &config.settings.twturl,
                    None,
                    &content,
                );

                self.tweets.extend(fetched);
            }
        }

        // Spawn tasks to download following twtxts
        if let Some(following) = config.following.as_ref() {
            for (key, value) in following {
                tasks.push(Task::perform(download_twtxt(value.to_string()), {
                    let key = key.clone();
                    let value = value.clone();
                    move |result| Message::DownloadFinished {
                        nick: key,
                        url: value,
                        result,
                    }
                }));
                self.pending_downloads += 1;
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

        let (reply_to, mentions, display_content) = parse_twt_contents(&self.composer);

        self.tweets.insert(
            0,
            Tweet {
                hash: compute_twt_hash(
                    &config.settings.nick,
                    &now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    &self.composer,
                ),
                reply_to,
                mentions,
                timestamp: now,
                author: config.settings.nick.clone(),
                url: config.settings.twturl.clone(),
                avatar,
                content: display_content,
            },
        );

        let mut file = OpenOptions::new()
            .append(true)
            .open(&config.settings.twtxt)
            .unwrap();

        writeln!(
            file,
            "{}\t{}",
            now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            self.composer
        )
        .ok();

        self.composer.clear();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let scroll = self.feed.view(&self.tweets).map(Message::Feed);

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
        .on_press_maybe(if self.pending_downloads == 0 {
            Some(Message::Refresh)
        } else {
            None
        })
        .width(Length::Fill)
        .padding([8, 16]);

        column![composer, scroll, refresh_button]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8)
            .into()
    }
}
