use bytes::Bytes;
use chrono::Utc;
use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, image::Handle, row, text, text_input},
};
use std::fs::OpenOptions;
use std::io::Write;

use crate::components::feed::{self, VirtualTimeline};
use crate::config::AppConfig;
use crate::utils::{
    FeedBundle, Tweet, compute_twt_hash, download_and_parse_twtxt, download_binary, parse_metadata,
    parse_tweets, parse_twt_contents,
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
    FeedLoaded {
        nick: String,
        url: String,
        result: Result<FeedBundle, String>,
    },
    AvatarLoaded {
        url: String, // The URL of the feed these tweets belong to
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

            Message::Refresh => {
                self.tweets.clear();
                self.feed.reset(0);

                let mut tasks = Vec::new();

                // handle local file
                let path = std::path::Path::new(&config.settings.twtxt);
                if let Ok(content) = std::fs::read_to_string(path) {
                    let bundle = FeedBundle {
                        metadata: parse_metadata(&content),
                        tweets: parse_tweets(
                            &config.settings.nick,
                            &config.settings.twturl,
                            None,
                            &content,
                        ),
                    };
                    tasks.push(Task::done(Message::FeedLoaded {
                        nick: config.settings.nick.clone(),
                        url: config.settings.twturl.clone(),
                        result: Ok(bundle),
                    }));
                }

                // handle following
                if let Some(following) = config.following.as_ref() {
                    for (nick, url) in following {
                        self.pending_downloads += 1;
                        // THis is terrible but ill fix later lmao
                        let follow_nick = nick.clone();
                        let follow_url = url.clone();
                        tasks.push(Task::perform(
                            download_and_parse_twtxt(follow_nick.clone(), follow_url.clone()),
                            move |result| Message::FeedLoaded {
                                nick: follow_nick.clone(),
                                url: follow_url.clone(),
                                result,
                            },
                        ));
                    }
                }
                Task::batch(tasks)
            }

            Message::FeedLoaded { nick, url, result } => {
                if let Ok(bundle) = result {
                    self.tweets.extend(bundle.tweets);
                    self.sort_and_refresh();

                    // trigger avatar download if available
                    if let Some(meta) = bundle.metadata {
                        if let Some(avatar_url) = meta.avatar {
                            return Task::perform(download_binary(avatar_url), move |res| {
                                Message::AvatarLoaded {
                                    url: url.clone(),
                                    result: res,
                                }
                            });
                        }
                    }
                }
                self.decrement_pending()
            }

            Message::AvatarLoaded { url, result } => {
                if let Ok(bytes) = result {
                    let new_handle = Handle::from_bytes(bytes);

                    // "patch" existing tweets that match this feed URL
                    for tweet in self.tweets.iter_mut() {
                        if tweet.url == url {
                            tweet.avatar = new_handle.clone();
                            if tweet.url == config.settings.twturl {
                                self.local_avatar = Some(new_handle.clone());
                            }
                        }
                    }
                }
                self.decrement_pending()
            }

            Message::Feed(feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(msg) => self.feed.update(msg, self.tweets.len()).map(Message::Feed),

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
        }
    }

    fn sort_and_refresh(&mut self) {
        self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.feed.reset(self.tweets.len());
    }

    fn decrement_pending(&mut self) -> Task<Message> {
        if self.pending_downloads > 0 {
            self.pending_downloads -= 1;
        }
        Task::none()
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
