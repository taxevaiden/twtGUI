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
    info_expanded: bool,
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
                avatar_bytes: None,
                tweets: Vec::new(),
                thread_tree: Vec::new(),
                metadata: None,
                pending_downloads: 0,
                feed,
                selected_follow: None,
                info_expanded: true,
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

    pub fn view(&self) -> Element<'_, Message> {
        fn button_style(theme: &Theme, status: button::Status) -> button::Style {
            let palette = theme.palette();
            let ext = theme.extended_palette();

            let bg = match status {
                button::Status::Hovered => ext.background.weaker.color,
                button::Status::Pressed => ext.background.stronger.color,
                _ => ext.background.weak.color,
            };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: palette.text,
                border: Border {
                    radius: Radius::from(4.0),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }

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

        let avatar_expanded: Element<_> = if let Some(handle) = &self.avatar_bytes {
            image::Image::new(handle.clone())
                .width(Length::Fixed(56.0))
                .height(Length::Fixed(56.0))
                .border_radius(28)
                .into()
        } else {
            container(text("?").size(20))
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

        let avatar_collapsed: Element<_> = if let Some(handle) = &self.avatar_bytes {
            image::Image::new(handle.clone())
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .border_radius(16)
                .into()
        } else {
            container(text("?").size(13))
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

        let timeline = self.feed.view(&self.tweets).map(Message::Feed);

        // Link badges row
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

        let info: Element<_> = if self.info_expanded {
            container(
                column![
                    row![
                        avatar_expanded,
                        column![
                            column![
                                text(nick.clone()).font(crate::app::BOLD_FONT),
                                text(desc).color(Color::from_rgba(1.0, 1.0, 1.0, 0.55)),
                                links_row.wrap(),
                            ]
                            .spacing(8),
                            row![
                                text(format!("{} following", following))
                                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.55)),
                                pick_list(
                                    follows,
                                    self.selected_follow.clone(),
                                    Message::FollowSelected,
                                )
                                .placeholder("View a followed feed...")
                                .width(Length::Fill),
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
                        button("Collapse")
                            .on_press(Message::CollapsePressed)
                            .padding([8, 16])
                            .style(button_style),
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
                        .style(button_style),
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
