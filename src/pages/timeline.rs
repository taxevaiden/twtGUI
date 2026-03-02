use bytes::Bytes;
use chrono::Utc;
use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, image::Handle, row, text, text_editor},
};
use std::fs::OpenOptions;
use std::io::Write;

use crate::components::threaded_feed::{self, LazyThreadedFeed};
use crate::config::AppConfig;
use crate::utils::{
    FeedBundle, Tweet, TweetNode, build_threads, compute_twt_hash, download_and_parse_twtxt,
    download_binary, parse_metadata, parse_tweets, parse_twt_contents,
};

pub struct TimelinePage {
    show_composer: bool,
    composer: text_editor::Content,
    tweets: Vec<Tweet>,
    thread_tree: Vec<TweetNode>,
    local_avatar: Option<Handle>,
    pending_downloads: usize,
    feed: LazyThreadedFeed,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerEdit(text_editor::Action),
    ToggleComposer,
    CancelCompose,
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
    Feed(threaded_feed::Message),
}

impl TimelinePage {
    pub fn new() -> Self {
        Self {
            show_composer: false,
            composer: text_editor::Content::new(),
            tweets: Vec::new(),
            thread_tree: Vec::new(),
            local_avatar: None,
            pending_downloads: 0,
            feed: LazyThreadedFeed::new(0),
        }
    }

    pub fn update(&mut self, message: Message, config: &AppConfig) -> Task<Message> {
        match message {
            Message::ComposerEdit(action) => {
                self.composer.perform(action);
                Task::none()
            }

            Message::ToggleComposer => {
                self.show_composer = !self.show_composer;
                Task::none()
            }

            Message::CancelCompose => {
                self.show_composer = false;
                self.composer = text_editor::Content::new();
                Task::none()
            }

            Message::PostPressed => {
                self.show_composer = false;
                self.send_tweet(config);
                Task::none()
            }

            Message::Refresh => {
                self.tweets.clear();
                self.thread_tree.clear();
                self.feed.reset(0);

                let mut tasks = Vec::new();

                // handle local file
                let path = std::path::Path::new(&config.paths.twtxt);

                if let Ok(content) = std::fs::read_to_string(path) {
                    let nick = config.metadata.nick.clone().unwrap_or_default();

                    let url = config.metadata.urls.first().cloned().unwrap_or_default();

                    let bundle = FeedBundle {
                        metadata: parse_metadata(&content),
                        tweets: parse_tweets(&nick, &url, None, &content),
                    };

                    tasks.push(Task::done(Message::FeedLoaded {
                        nick,
                        url,
                        result: Ok(bundle),
                    }));
                }

                // handle following
                for link in &config.metadata.follows {
                    self.pending_downloads += 1;

                    let follow_nick = link.text.clone();
                    let follow_url = link.url.clone();

                    tasks.push(Task::perform(
                        download_and_parse_twtxt(follow_nick.clone(), follow_url.clone(), true),
                        move |result| Message::FeedLoaded {
                            nick: follow_nick.clone(),
                            url: follow_url.clone(),
                            result,
                        },
                    ));
                }

                Task::batch(tasks)
            }

            Message::FeedLoaded { nick, url, result } => {
                let mut extra_task = Task::none();

                match result {
                    Ok(bundle) => {
                        println!("Feed successfully loaded for {} @ {}", nick, url);
                        self.tweets.extend(bundle.tweets);

                        // trigger avatar download if available
                        if let Some(meta) = bundle.metadata {
                            if let Some(avatar_url) = meta.avatar {
                                self.pending_downloads += 1;
                                extra_task =
                                    Task::perform(download_binary(avatar_url), move |res| {
                                        Message::AvatarLoaded {
                                            url: url.clone(),
                                            result: res,
                                        }
                                    });
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error loading feed: {}", e);
                    }
                }

                let decrement_task = self.decrement_pending();

                Task::batch(vec![decrement_task, extra_task])
            }

            Message::AvatarLoaded { url, result } => {
                match result {
                    Ok(bytes) => {
                        println!("Avatar successfully loaded for {}", url);
                        let new_handle = Handle::from_bytes(bytes);

                        // "patch" existing tweets that match this feed URL
                        for tweet in self.tweets.iter_mut() {
                            if tweet.url == url {
                                tweet.avatar = new_handle.clone();
                                if tweet.url == config.metadata.urls[0] {
                                    self.local_avatar = Some(new_handle.clone());
                                }
                            }
                        }
                    }

                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                self.decrement_pending()
            }

            Message::Feed(threaded_feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(msg) => self.feed.update(msg, self.tweets.len()).map(Message::Feed),

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
        }
    }

    fn sort_and_refresh(&mut self) {
        self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.thread_tree = build_threads(&self.tweets);
        self.feed.reset(self.tweets.len());
    }

    fn decrement_pending(&mut self) -> Task<Message> {
        if self.pending_downloads > 0 {
            self.pending_downloads -= 1;
        }

        if self.pending_downloads == 0 {
            self.sort_and_refresh();
        }
        Task::none()
    }

    fn send_tweet(&mut self, config: &AppConfig) {
        let composer_text = self.composer.text();

        if composer_text.trim().is_empty() {
            return;
        }

        let now = Utc::now();
        let avatar = self
            .local_avatar
            .clone()
            .unwrap_or_else(|| Handle::from_bytes(Bytes::new()));

        let (reply_to, mentions, display_content) = parse_twt_contents(&composer_text);

        let nick = match &config.metadata.nick {
            Some(n) => n.clone(),
            None => return, // no nick set, abort
        };

        let url = config.metadata.urls.first().cloned().unwrap_or_default();
        let timestamp_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let new_tweet = Tweet {
            hash: compute_twt_hash(&url, &timestamp_str, &composer_text),
            reply_to,
            mentions,
            timestamp: now,
            author: nick.clone(),
            url,
            avatar,
            content: display_content,
        };

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.paths.twtxt)
        {
            let written = composer_text.replace("\n", "\\u2028");
            let _ = writeln!(file, "{}\t{}", timestamp_str, written);
        }

        self.tweets.insert(0, new_tweet);

        self.sort_and_refresh();

        self.composer = text_editor::Content::new();
    }

    pub fn view(&self) -> Element<'_, Message> {
        let compose_button = button(
            text("Compose Tweet")
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .on_press(Message::ToggleComposer)
        .width(Length::Fill)
        .padding([8, 16]);

        let mut col = column!().spacing(8);

        col = col.push(compose_button);

        if self.show_composer {
            col = col.push(
                column![
                    text_editor(&self.composer)
                        .placeholder("What's on your mind?")
                        .on_action(Message::ComposerEdit)
                        .height(Length::Fill)
                        .padding(8),
                    row![
                        button(text("Post").align_x(Alignment::Center).width(Length::Fill))
                            .on_press(Message::PostPressed)
                            .width(Length::Fill)
                            .padding([8, 16]),
                        button(
                            text("Cancel")
                                .align_x(Alignment::Center)
                                .width(Length::Fill)
                        )
                        .on_press(Message::CancelCompose)
                        .width(Length::Fill)
                        .padding([8, 16]),
                    ]
                    .width(Length::Fill)
                    .spacing(8)
                ]
                .spacing(8),
            );
        } else {
            let scroll = self
                .feed
                .view(&self.thread_tree, &self.tweets)
                .map(Message::Feed);

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

            col = col.push(scroll);
            col = col.push(refresh_button);
        }

        col.width(Length::Fill).height(Length::Fill).into()
    }
}
