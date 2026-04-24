//! A page that renders a single twtxt feed and its metadata.
//!
//! This page can load an arbitrary feed URL and display its profile header + timeline.

use bytes::Bytes;
use iced::{
    Alignment, Color, Element, Length, Task,
    widget::{
        button, column, container,
        image::{self, Handle},
        pick_list, rich_text, row, span, text, text_input,
    },
};

use crate::utils::{
    FeedBundle, Metadata, Tweet, TweetNode, download_and_parse_twtxt, download_binary,
};
use crate::{components::threaded_feed, config::AppConfig};
use crate::{components::threaded_feed::LazyThreadedFeed, utils::build_threads};

/// The state for the view page.
///
/// This page is responsible for displaying a single feed and its metadata.
pub struct ViewPage {
    composer: String,
    avatar_bytes: Option<Handle>,
    tweets: Vec<Tweet>,
    thread_tree: Vec<TweetNode>,
    metadata: Option<Metadata>,
    pending_downloads: usize,
    feed: LazyThreadedFeed,
    selected_follow: Option<String>,
}

/// Messages used to update the view page.
#[derive(Debug, Clone)]
pub enum Message {
    /// The feed URL input changed.
    ComposerChanged(String),
    /// The user pressed the "View" button.
    ViewPressed,
    /// A feed has finished loading.
    FeedLoaded {
        url: String,
        result: Result<FeedBundle, String>,
    },
    /// An avatar image has finished downloading.
    AvatarLoaded {
        url: String,
        result: Result<Bytes, String>,
    },
    /// Navigate to another page.
    RedirectToPage(crate::app::RedirectInfo),
    /// A link in the metadata was clicked.
    LinkClicked(String),
    /// Messages forwarded from the threaded feed component.
    Feed(threaded_feed::Message),
    /// A feed was selected from the following dropdown.
    FollowSelected(String),
}

impl ViewPage {
    pub fn new(config: &AppConfig) -> (Self, Task<Message>) {
        let (feed, feed_task) = LazyThreadedFeed::new(&[], &[]);
        (
            Self {
                composer: config.metadata.urls.first().cloned().unwrap_or_default(),
                avatar_bytes: None,
                tweets: Vec::new(),
                thread_tree: Vec::new(),
                metadata: None,
                pending_downloads: 0,
                feed,
                selected_follow: None,
            },
            feed_task.map(Message::Feed),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ComposerChanged(value) => {
                self.composer = value;
                Task::none()
            }

            Message::ViewPressed => {
                self.tweets.clear();
                self.thread_tree.clear();
                self.metadata = None;
                self.avatar_bytes = None;
                self.pending_downloads = 1;
                let reset_task = self.feed.reset(&[], &[]).map(Message::Feed);

                let url = self.composer.clone();

                Task::batch([
                    reset_task,
                    Task::perform(
                        download_and_parse_twtxt("unknown".into(), url.clone(), false),
                        move |result| Message::FeedLoaded { url, result },
                    ),
                ])
            }

            Message::FeedLoaded { url, result } => {
                self.pending_downloads -= 1;

                let Ok(bundle) = result else {
                    println!("Error loading feed for {}", url);
                    return Task::none();
                };

                println!("Feed successfully loaded for {}", url);
                self.metadata = bundle.metadata.clone();
                self.tweets = bundle.tweets;
                self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                let thread_tree = build_threads(&self.tweets);
                let feed_task = self
                    .feed
                    .reset(&thread_tree, &self.tweets)
                    .map(Message::Feed);

                let avatar_task = bundle
                    .metadata
                    .and_then(|meta| meta.avatar)
                    .map(|avatar_url| {
                        self.pending_downloads += 1;
                        Task::perform(download_binary(avatar_url), move |res| {
                            Message::AvatarLoaded {
                                url: url.clone(),
                                result: res,
                            }
                        })
                    })
                    .unwrap_or_else(Task::none);

                Task::batch([feed_task, avatar_task])
            }

            Message::AvatarLoaded { url, result } => {
                if let Ok(bytes) = result {
                    println!("Avatar successfully loaded for {}", url);
                    let handle = Handle::from_bytes(bytes);
                    self.avatar_bytes = Some(handle.clone());

                    for tweet in self.tweets.iter_mut().filter(|t| t.url == url) {
                        tweet.avatar = handle.clone();
                    }
                } else if let Err(e) = result {
                    println!("Error loading avatar for {}: {}", url, e);
                }

                self.pending_downloads -= 1;
                Task::none()
            }

            Message::LinkClicked(url) => {
                if url.contains("twtxt") && url.ends_with(".txt") {
                    Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                        page: crate::app::Page::View,
                        content: url,
                    }))
                } else {
                    let _ = webbrowser::open(&url);
                    Task::none()
                }
            }

            Message::Feed(threaded_feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(msg) => self.feed.update(msg, &self.tweets).map(Message::Feed),

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),

            Message::FollowSelected(url) => {
                self.selected_follow = Some(url.clone());

                Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                    page: crate::app::Page::View,
                    content: url,
                }))
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let nick = self
            .metadata
            .as_ref()
            .and_then(|m| m.nick.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                url::Url::parse(&self.composer)
                    .ok()
                    .and_then(|url| url.host_str().map(str::to_string))
                    .unwrap_or_else(|| "unknown".to_string())
            });

        let desc = self
            .metadata
            .as_ref()
            .and_then(|m| m.description.as_ref())
            .cloned()
            .unwrap_or("No description provided.".to_string());

        let follows: Vec<String> = self
            .metadata
            .as_ref()
            .map(|m| m.follows.iter().map(|f| f.url.clone()).collect())
            .unwrap_or_default();

        let following = self
            .metadata
            .as_ref()
            .and_then(|m| m.following.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                if let Some(m) = &self.metadata {
                    m.follows.len() as u64
                } else {
                    0
                }
            });

        let links = self
            .metadata
            .as_ref()
            .map(|m| m.links.clone())
            .unwrap_or_default();

        let avatar: Element<_> = if let Some(handle) = &self.avatar_bytes {
            image::Image::new(handle.clone())
                .width(Length::Fixed(128.0))
                .height(Length::Fixed(128.0))
                .border_radius(64)
                .into()
        } else {
            container("No Avatar")
                .width(Length::Fixed(128.0))
                .height(Length::Fixed(128.0))
                .center_x(Length::Fixed(128.0))
                .center_y(Length::Fixed(128.0))
                .into()
        };

        let timeline = self.feed.view(&self.tweets).map(Message::Feed);

        let mut col: iced::widget::Column<Message> = column!().spacing(8);

        for link in links {
            col = col.push(
                rich_text![
                    span(link.text.clone())
                        .link(link.url.clone())
                        .underline(true)
                        .color(Color::from_rgb(0.4, 0.6, 1.0))
                ]
                .on_link_click(Message::LinkClicked),
            )
        }

        let info = row![
            avatar,
            row![
                column![
                    text(nick).size(24),
                    text(desc),
                    text(format!("Following: {}", following)),
                    pick_list(
                        follows,
                        self.selected_follow.clone(),
                        Message::FollowSelected,
                    )
                    .placeholder("View a followed feed...")
                ]
                .max_width(350.0)
                .spacing(16),
                col,
            ]
            .spacing(64)
            .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center)
        .spacing(32)
        .padding(32);

        let scroll = column![info, timeline]
            .spacing(32)
            .align_x(Alignment::Center)
            .height(iced::Length::Fill);

        let composer = row![
            text_input("https://example.com/twtxt.txt", &self.composer)
                .on_input(Message::ComposerChanged)
                .padding(8),
            button("View")
                .on_press_maybe(if self.pending_downloads == 0 {
                    Some(Message::ViewPressed)
                } else {
                    None
                })
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
                self.selected_follow = None;
                Task::done(Message::ViewPressed)
            }
            _ => Task::none(),
        }
    }
}
