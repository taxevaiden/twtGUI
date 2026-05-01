//! A page that renders a single twtxt feed and its metadata.
//!
//! This page can load an arbitrary feed URL and display its profile header + timeline.

use bytes::Bytes;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Task, Theme,
    border::Radius,
    widget::{
        button, column, container,
        image::{self, Handle},
        pick_list, rich_text, row,
        row::Row,
        space, span, text, text_input,
    },
};
use tracing::{error, info};

use crate::utils::{
    Metadata, ParsedCache, Tweet, TweetNode, download_and_parse_twtxt, download_binary,
    styling::{sec_button_style, sec_pick_list_style, sec_pick_menu_style, secondary_text},
};
use crate::{components::threaded_feed, config::AppConfig};
use crate::{components::threaded_feed::LazyThreadedFeed, utils::build_threads};

/// The state for the view page.
///
/// This page is responsible for displaying a single feed and its metadata.
pub struct ViewPage {
    composer: String,
    tweets: Vec<Tweet>,
    thread_tree: Vec<TweetNode>,
    metadata: Option<Metadata>,
    pending_downloads: usize,
    feed: LazyThreadedFeed,
    feed_hash: String,
    selected_follow: Option<String>,
    info_expanded: bool,
    loading_archive: bool,
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
        result: Box<Result<ParsedCache, String>>,
    },
    /// The user pressed the button.
    LoadArchive,
    /// An avatar image has finished downloading.
    AvatarLoaded {
        url: String,
        result: Box<Result<Bytes, String>>,
        hash: String,
    },
    /// An archived feed has finished loading.
    ArchiveLoaded {
        result: Box<Result<ParsedCache, String>>,
    },
    /// Navigate to another page.
    RedirectToPage(crate::app::RedirectInfo),
    /// A link in the metadata was clicked.
    LinkClicked(String),
    /// Messages forwarded from the threaded feed component.
    Feed(threaded_feed::Message),
    /// A feed was selected from the following dropdown.
    FollowSelected(String),
    /// The user pressed the "Expand" button.
    ExpandPressed,
    /// The user pressed the "Collapse" button.
    CollapsePressed,
}

impl ViewPage {
    pub fn new(config: &AppConfig) -> (Self, Task<Message>) {
        let (feed, feed_task) = LazyThreadedFeed::new(&[], &[]);
        (
            Self {
                composer: config.metadata.urls.first().cloned().unwrap_or_default(),
                tweets: Vec::new(),
                thread_tree: Vec::new(),
                metadata: None,
                pending_downloads: 0,
                feed,
                feed_hash: String::new(),
                selected_follow: None,
                info_expanded: true,
                loading_archive: false,
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
                self.feed.avatars.clear();
                self.metadata = None;
                self.feed_hash = String::new();
                self.pending_downloads = 1;
                let reset_task = self.feed.reset(&[], &[]).map(Message::Feed);

                let url = self.composer.clone();

                Task::batch([
                    reset_task,
                    Task::perform(
                        download_and_parse_twtxt("unknown".into(), url.clone(), None, false),
                        move |result| Message::FeedLoaded {
                            url,
                            result: Box::new(result),
                        },
                    ),
                ])
            }

            Message::FeedLoaded { url, result } => {
                self.pending_downloads -= 1;

                let Ok(parsed) = *result else {
                    error!("Error loading feed for {}", url);
                    return Task::none();
                };

                info!("Feed successfully loaded for {}", url);
                self.metadata = parsed.bundle.metadata.clone();
                self.tweets = parsed.bundle.tweets;
                self.tweets.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
                self.feed_hash = parsed.content_hash.clone();
                let thread_tree = build_threads(&self.tweets);
                let feed_task = self
                    .feed
                    .reset(&thread_tree, &self.tweets)
                    .map(Message::Feed);

                let avatar_task = parsed
                    .bundle
                    .metadata
                    .and_then(|meta| meta.avatar)
                    .map(|avatar_url| {
                        self.pending_downloads += 1;
                        Task::perform(download_binary(avatar_url), move |res| {
                            Message::AvatarLoaded {
                                url: url.clone(),
                                result: Box::new(res),
                                hash: parsed.content_hash.clone(),
                            }
                        })
                    })
                    .unwrap_or_else(Task::none);

                Task::batch([feed_task, avatar_task])
            }

            Message::AvatarLoaded { url, result, hash } => {
                if let Ok(bytes) = *result {
                    info!("Avatar successfully loaded for {}", url);
                    let handle = Handle::from_bytes(bytes);
                    self.feed.avatars.insert(hash, handle.clone());
                } else if let Err(e) = *result {
                    error!("Error loading avatar for {}: {}", url, e);
                }

                self.pending_downloads -= 1;
                Task::none()
            }

            Message::LoadArchive => {
                if let Some(meta) = &self.metadata
                    && let Some(prev) = &meta.prev
                {
                    self.loading_archive = true;
                    self.pending_downloads += 1;

                    let url = if let Ok(base) = url::Url::parse(&self.composer) {
                        match base.join(&prev.url) {
                            Ok(joined) => joined.to_string(),
                            Err(_) => prev.url.clone(),
                        }
                    } else {
                        prev.url.clone()
                    };

                    return Task::perform(
                        download_and_parse_twtxt(
                            "archive".into(),
                            url,
                            Some(self.composer.clone()),
                            false,
                        ),
                        |result| Message::ArchiveLoaded {
                            result: Box::new(result),
                        },
                    );
                }

                Task::none()
            }

            Message::ArchiveLoaded { result } => {
                self.pending_downloads -= 1;
                self.loading_archive = false;

                let Ok(mut parsed) = *result else {
                    error!("Failed to load archive feed");
                    return Task::none();
                };

                for tweet in &mut parsed.bundle.tweets {
                    tweet.feed_hash = self.feed_hash.clone();
                }

                self.tweets.extend(parsed.bundle.tweets);
                self.tweets.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
                self.thread_tree = build_threads(&self.tweets);

                let feed_task = self
                    .feed
                    .reset(&self.thread_tree, &self.tweets)
                    .map(Message::Feed);

                let chain_task = if let Some(prev) = parsed.bundle.metadata.and_then(|m| m.prev) {
                    self.loading_archive = true;
                    self.pending_downloads += 1;

                    let url = if let Ok(base) = url::Url::parse(&self.composer) {
                        match base.join(&prev.url) {
                            Ok(joined) => joined.to_string(),
                            Err(_) => prev.url.clone(),
                        }
                    } else {
                        prev.url.clone()
                    };

                    Task::perform(
                        download_and_parse_twtxt(
                            "archive".into(),
                            url,
                            Some(self.composer.clone()),
                            false,
                        ),
                        |result| Message::ArchiveLoaded {
                            result: Box::new(result),
                        },
                    )
                } else {
                    Task::none()
                };

                Task::batch([feed_task, chain_task])
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

            Message::ExpandPressed => {
                self.info_expanded = true;
                Task::none()
            }

            Message::CollapsePressed => {
                self.info_expanded = false;
                Task::none()
            }
        }
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
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
            .unwrap_or({
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

        let avatar_expanded: Element<_> =
            if let Some(handle) = self.feed.avatars.get(&self.feed_hash) {
                image::Image::new(handle.clone())
                    .width(Length::Fixed(56.0))
                    .height(Length::Fixed(56.0))
                    .border_radius(28)
                    .into()
            } else {
                container(text("?").size(28))
                    .width(Length::Fixed(56.0))
                    .height(Length::Fixed(56.0))
                    .center_x(Length::Fixed(56.0))
                    .center_y(Length::Fixed(56.0))
                    .style(|theme: &Theme| {
                        let ext = theme.extended_palette();
                        container::Style {
                            background: Some(Background::Color(ext.background.strong.color)),
                            border: Border {
                                radius: Radius::from(28.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .into()
            };

        let avatar_collapsed: Element<_> =
            if let Some(handle) = self.feed.avatars.get(&self.feed_hash) {
                image::Image::new(handle)
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .border_radius(16)
                    .into()
            } else {
                container(text("?").size(16))
                    .width(Length::Fixed(32.0))
                    .height(Length::Fixed(32.0))
                    .center_x(Length::Fixed(32.0))
                    .center_y(Length::Fixed(32.0))
                    .style(|theme: &Theme| {
                        let ext = theme.extended_palette();
                        container::Style {
                            background: Some(Background::Color(ext.background.strong.color)),
                            border: Border {
                                radius: Radius::from(16.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .into()
            };

        let timeline = self
            .feed
            .view(theme, &self.tweets, false)
            .map(Message::Feed);

        // Links row
        let mut links_row: Row<Message> = row!().spacing(4);
        for link in links {
            links_row = links_row.push(
                rich_text![
                    span(link.text.clone())
                        .link(link.url.clone())
                        .underline(true)
                        .color(Color::from_rgb(0.4, 0.6, 1.0))
                ]
                .on_link_click(Message::LinkClicked),
            );
        }

        // Archive button
        let archive_button: Element<_> = if let Some(meta) = &self.metadata {
            if meta.prev.is_some() {
                button(if self.loading_archive {
                    "Loading..."
                } else {
                    "Load older posts"
                })
                .on_press_maybe(if self.loading_archive {
                    None
                } else {
                    Some(Message::LoadArchive)
                })
                .padding([8, 16])
                .style(sec_button_style)
                .into()
            } else {
                space().into()
            }
        } else {
            space().into()
        };

        let info: Element<_> = if self.info_expanded {
            container(
                column![
                    row![
                        avatar_expanded,
                        column![
                            column![
                                text(nick.clone()).font(crate::app::BOLD_FONT),
                                text(desc).color(secondary_text(theme)),
                                links_row.wrap(),
                            ]
                            .spacing(8),
                            row![
                                text(format!("{} following", following))
                                    .color(secondary_text(theme)),
                                pick_list(
                                    follows,
                                    self.selected_follow.clone(),
                                    Message::FollowSelected,
                                )
                                .placeholder("View a followed feed...")
                                .width(Length::Fill)
                                .style(sec_pick_list_style)
                                .menu_style(sec_pick_menu_style),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(8),
                        ]
                        .padding([8, 0])
                        .spacing(18)
                        .width(Length::Fill),
                    ]
                    .spacing(16)
                    .align_y(Alignment::Start),
                    row![
                        space().width(Length::Fill),
                        archive_button,
                        button("Collapse")
                            .on_press(Message::CollapsePressed)
                            .padding([8, 16])
                            .style(sec_button_style),
                    ],
                ]
                .spacing(12),
            )
            .padding([20, 24])
            .into()
        } else {
            container(
                row![
                    avatar_collapsed,
                    text(nick).font(crate::app::BOLD_FONT),
                    space().width(Length::Fill),
                    text(format!("{} following", following))
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.4)),
                    button("Expand")
                        .on_press(Message::ExpandPressed)
                        .padding([8, 16])
                        .style(sec_button_style),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding([10, 20])
            .into()
        };

        let scroll = column![info, timeline]
            .spacing(if self.info_expanded { 16 } else { 8 })
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

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
