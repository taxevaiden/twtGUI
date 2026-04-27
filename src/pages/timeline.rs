//! A page that displays the user's timeline and allows composing new tweets.

use bytes::Bytes;
use chrono::Utc;
use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, image::Handle, markdown, row, text, text_editor},
};
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{error, info};

use crate::config::AppConfig;
use crate::utils::{
    FeedBundle, Tweet, TweetNode, build_threads, compute_twt_hash, download_and_parse_twtxt,
    download_binary, parse_metadata, parse_tweets, parse_twt_contents,
};
use crate::{
    components::threaded_feed::{self, LazyThreadedFeed},
    utils::run_script,
};

/// The state for the timeline page.
///
/// This page is responsible for showing a combined timeline from the user's
/// own feed and any followed feeds, as well as providing a composer for new tweets.
pub struct TimelinePage {
    show_composer: bool,
    composer: text_editor::Content,
    tweets: Vec<Tweet>,
    thread_tree: Vec<TweetNode>,
    local_avatar: Option<Handle>,
    pending_downloads: usize,
    feed: LazyThreadedFeed,
}

/// Messages used to update the timeline page.
#[derive(Debug, Clone)]
pub enum Message {
    /// The composer text was edited.
    ComposerEdit(text_editor::Action),
    /// Open/close the composer panel.
    ToggleComposer,
    /// Cancel the current composition.
    CancelCompose,
    /// Post the composed tweet.
    PostPressed,
    /// Refresh all feeds.
    Refresh,
    /// A feed finished loading (either local or remote).
    FeedLoaded {
        nick: String,
        url: String,
        result: Box<Result<FeedBundle, String>>,
    },
    /// An avatar image has finished downloading.
    AvatarLoaded {
        url: String, // The URL of the feed these tweets belong to
        result: Box<Result<Bytes, String>>,
    },
    /// Trigger a navigation to another page.
    RedirectToPage(crate::app::RedirectInfo),
    /// Messages forwarded from the threaded feed component.
    Feed(threaded_feed::Message),
}

impl TimelinePage {
    pub fn new() -> (Self, Task<Message>) {
        let (feed, feed_task) = LazyThreadedFeed::new(&[], &[]);
        (
            Self {
                show_composer: false,
                composer: text_editor::Content::new(),
                tweets: Vec::new(),
                thread_tree: Vec::new(),
                local_avatar: None,
                pending_downloads: 0,
                feed,
            },
            feed_task.map(Message::Feed),
        )
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
                self.send_tweet(config)
            }

            Message::Refresh => {
                self.tweets.clear();
                self.thread_tree.clear();
                let reset_task = self.feed.reset(&[], &[]).map(Message::Feed);

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
                        result: Box::new(Ok(bundle)),
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
                            result: Box::new(result),
                        },
                    ));
                }

                tasks.push(reset_task);

                Task::batch(tasks)
            }

            Message::FeedLoaded { nick, url, result } => {
                let Ok(bundle) = *result else {
                    error!("Error loading feed for {} @ {}", nick, url);
                    return self.decrement_pending();
                };

                info!("Feed successfully loaded for {} @ {}", nick, url);
                self.tweets.extend(bundle.tweets);

                let avatar_task = bundle
                    .metadata
                    .and_then(|meta| meta.avatar)
                    .map(|avatar_url| {
                        self.pending_downloads += 1;
                        Task::perform(download_binary(avatar_url), move |res| {
                            Message::AvatarLoaded {
                                url: url.clone(),
                                result: Box::new(res),
                            }
                        })
                    })
                    .unwrap_or_else(Task::none);

                Task::batch([self.decrement_pending(), avatar_task])
            }

            Message::AvatarLoaded { url, result } => {
                if let Ok(bytes) = *result {
                    info!("Avatar successfully loaded for {}", url);
                    let handle = Handle::from_bytes(bytes);

                    if url == config.metadata.urls[0] {
                        self.local_avatar = Some(handle.clone());
                    }

                    for tweet in self.tweets.iter_mut().filter(|t| t.url == url) {
                        tweet.avatar = handle.clone();
                    }
                } else if let Err(e) = *result {
                    error!("Error loading avatar for {}: {}", url, e);
                }

                self.decrement_pending()
            }

            Message::Feed(threaded_feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(threaded_feed::Message::ReplyClicked(index)) => {
                let tweet = self.tweets.get(index).cloned().unwrap(); // TODO: Implement default for safe unwrapping
                let hash = tweet.hash;
                let author = tweet.author;
                let source = tweet.url;

                self.show_composer = true;
                self.composer = text_editor::Content::with_text(
                    format!("(#{}) @<{} {}> ", hash, author, source).as_str(),
                );

                Task::none()
            }

            Message::Feed(msg) => self.feed.update(msg, &self.tweets).map(Message::Feed),

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
        }
    }

    fn sort_and_refresh(&mut self) -> Task<Message> {
        self.tweets.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        let thread_tree = build_threads(&self.tweets);
        self.feed
            .reset(&thread_tree, &self.tweets)
            .map(Message::Feed)
    }

    fn decrement_pending(&mut self) -> Task<Message> {
        if self.pending_downloads > 0 {
            self.pending_downloads -= 1;
        }

        if self.pending_downloads == 0 {
            return self.sort_and_refresh();
        }
        Task::none()
    }

    fn send_tweet(&mut self, config: &AppConfig) -> Task<Message> {
        if let Some(path) = &config.paths.pre_tweet_script {
            run_script(path, &[]).ok();
        }

        let composer_text = self.composer.text();

        if composer_text.trim().is_empty() {
            return Task::none();
        }

        let Some(nick) = config.metadata.nick.clone() else {
            return Task::none();
        };

        let now = Utc::now();
        let avatar = self
            .local_avatar
            .clone()
            .unwrap_or_else(|| Handle::from_bytes(Bytes::new()));

        let (reply_to, display_content) = parse_twt_contents(&composer_text);
        let url = config.metadata.urls.first().cloned().unwrap_or_default();
        let timestamp_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let written = composer_text.replace("\n", "\\u2028");

        let new_tweet = Tweet {
            hash: compute_twt_hash(&url, &timestamp_str, &written),
            reply_to,
            timestamp: now,
            author: nick,
            url,
            avatar,
            content: display_content.clone(),
            md_items: markdown::parse(&display_content).collect(),
        };

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.paths.twtxt)
        {
            let _ = writeln!(file, "{}\t{}", timestamp_str, written);
        }

        self.tweets.insert(0, new_tweet);
        self.composer = text_editor::Content::new();

        if let Some(path) = &config.paths.post_tweet_script {
            run_script(path, &[]).ok();
        }

        self.sort_and_refresh()
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
        } else if self.pending_downloads == 0 {
            let scroll = self.feed.view(&self.tweets).map(Message::Feed);

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
