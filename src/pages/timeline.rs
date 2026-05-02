//! A page that displays the user's timeline and allows composing new tweets.

use bytes::Bytes;
use iced::{
    Alignment, Element, Length, Task, Theme,
    widget::{Stack, button, column, container, image::Handle, row, space, text, text_editor},
};

use tracing::{error, info};

use crate::twtxt::{compose_twtxt_tweet, download_and_parse_feed, load_local_twtxt_feed};
use crate::utils::{ParsedCache, Tweet, TweetNode, build_threads, download_binary};
use crate::{
    components::threaded_feed::{self, LazyThreadedFeed},
    utils::styling::toolbar_minput_style,
};
use crate::{config::AppConfig, utils::styling::toolbar_button_style};

/// The state for the timeline page.
///
/// This page is responsible for showing a combined timeline from the user's
/// own feed and any followed feeds, as well as providing a composer for new tweets.
pub struct TimelinePage {
    show_composer: bool,
    composer: text_editor::Content,
    tweets: Vec<Tweet>,
    thread_tree: Vec<TweetNode>,
    pending_downloads: usize,
    feed: LazyThreadedFeed,
    local_hash: Option<String>,
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
        result: Box<Result<ParsedCache, String>>,
    },
    /// An avatar image has finished downloading.
    AvatarLoaded {
        url: String, // The URL of the feed these tweets belong to
        result: Box<Result<Bytes, String>>,
        hash: String,
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
                pending_downloads: 0,
                local_hash: None,
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
                self.post_composed_tweet(config)
            }

            Message::Refresh => {
                self.tweets.clear();
                self.thread_tree.clear();
                self.feed.avatars.clear();
                let reset_task = self.feed.reset(&[], &[]).map(Message::Feed);

                let mut tasks = Vec::new();

                // handle local file
                if let Some((nick, url, parsed)) = load_local_twtxt_feed(config) {
                    self.local_hash = Some(parsed.content_hash.clone());
                    tasks.push(Task::done(Message::FeedLoaded {
                        nick,
                        url,
                        result: Box::new(Ok(parsed)),
                    }));
                }

                // handle following
                for link in &config.metadata.follows {
                    self.pending_downloads += 1;

                    let follow_nick = link.text.clone();
                    let follow_url = link.url.clone();

                    tasks.push(Task::perform(
                        download_and_parse_feed(
                            follow_nick.clone(),
                            follow_url.clone(),
                            None,
                            true,
                        ),
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
                let Ok(parsed) = *result else {
                    error!("Timeline: error loading feed for {} @ {}", nick, url);
                    return self.decrement_pending();
                };

                info!("Timeline: feed successfully loaded for {} @ {}", nick, url);

                let content_hash = parsed.content_hash.clone();
                let avatar_url = parsed
                    .bundle
                    .metadata
                    .as_ref()
                    .and_then(|m| m.avatar.clone());

                self.tweets.extend(parsed.bundle.tweets);

                let avatar_task = avatar_url
                    .map(|avatar_url| {
                        self.pending_downloads += 1;
                        Task::perform(download_binary(avatar_url), move |res| {
                            Message::AvatarLoaded {
                                url: url.clone(),
                                result: Box::new(res),
                                hash: content_hash.clone(),
                            }
                        })
                    })
                    .unwrap_or_else(Task::none);

                Task::batch([self.decrement_pending(), avatar_task])
            }

            Message::AvatarLoaded { url, result, hash } => {
                match *result {
                    Ok(bytes) => {
                        info!("Timeline: avatar successfully loaded for {}", url);
                        self.feed.avatars.insert(hash, Handle::from_bytes(bytes));
                    }
                    Err(e) => {
                        error!("Timeline: error loading avatar for {}: {}", url, e);
                    }
                }
                self.decrement_pending()
            }

            Message::Feed(threaded_feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(threaded_feed::Message::ReplyClicked(index)) => {
                if let Some(tweet) = self.tweets.get(index).cloned() {
                    let hash = tweet.hash;
                    let author = tweet.author;
                    let source = tweet.url;

                    self.show_composer = true;
                    self.composer = text_editor::Content::with_text(
                        format!("(#{}) @<{} {}> ", hash, author, source).as_str(),
                    );
                }

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

    fn post_composed_tweet(&mut self, config: &AppConfig) -> Task<Message> {
        let composer_text = self.composer.text();

        if let Some(tweet) = compose_twtxt_tweet(&composer_text, config, self.local_hash.clone()) {
            self.tweets.insert(0, tweet);
            self.composer = text_editor::Content::new();
            self.sort_and_refresh()
        } else {
            Task::none()
        }
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let compose_button = button(
            text("Compose Twt")
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .on_press(Message::ToggleComposer)
        .width(Length::Fill)
        .padding([8, 16])
        .style(toolbar_button_style);

        let refresh_button = button("Refresh")
            .on_press_maybe(if self.pending_downloads == 0 {
                Some(Message::Refresh)
            } else {
                None
            })
            .padding([8, 16])
            .style(toolbar_button_style);

        let toolbar = row![compose_button, refresh_button].spacing(8);

        let feed = self.feed.view(theme, &self.tweets, true).map(Message::Feed);

        let base = column![toolbar, feed]
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);

        let stack = Stack::new().push(base);

        if self.show_composer {
            let composer_sheet = container(
                column![
                    text_editor(&self.composer)
                        .placeholder("What's on your mind?")
                        .on_action(Message::ComposerEdit)
                        .height(300)
                        .padding(8)
                        .style(toolbar_minput_style),
                    row![
                        button(text("Post").align_x(Alignment::Center).width(Length::Fill))
                            .on_press(Message::PostPressed)
                            .width(Length::Fill)
                            .padding([8, 16])
                            .style(toolbar_button_style),
                        button(
                            text("Cancel")
                                .align_x(Alignment::Center)
                                .width(Length::Fill)
                        )
                        .on_press(Message::CancelCompose)
                        .width(Length::Fill)
                        .padding([8, 16])
                        .style(toolbar_button_style),
                    ]
                    .spacing(8)
                    .width(Length::Fill),
                ]
                .spacing(8),
            )
            .width(Length::Fill)
            .height(Length::Shrink);

            stack
                .push(
                    column![
                        space().height(40),
                        composer_sheet,
                        space().height(Length::Fill)
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .into()
        } else {
            stack.into()
        }
    }
}
