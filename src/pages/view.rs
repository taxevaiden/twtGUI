use bytes::Bytes;
use iced::{
    Alignment, Color, Element, Length, Task,
    widget::{
        button, column, container,
        image::{self, Handle},
        rich_text, row, span, text, text_input,
    },
};

use crate::components::feed::{self, VirtualTimeline};
use crate::config::AppConfig;
use crate::utils::{FeedBundle, Metadata, Tweet, download_and_parse_twtxt, download_binary};

pub struct ViewPage {
    composer: String,
    avatar_bytes: Option<Handle>,
    tweets: Vec<Tweet>,
    metadata: Option<Metadata>,
    pending_downloads: usize,
    feed: VirtualTimeline,
}

#[derive(Debug, Clone)]
pub enum Message {
    ComposerChanged(String),
    ViewPressed,
    FeedLoaded {
        url: String,
        result: Result<FeedBundle, String>,
    },
    AvatarLoaded {
        url: String,
        result: Result<Bytes, String>,
    },
    RedirectToPage(crate::app::RedirectInfo),
    LinkClicked(String),
    Feed(feed::Message),
}

impl ViewPage {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            composer: config.metadata.urls.first().cloned().unwrap_or_default(),
            avatar_bytes: None,
            tweets: Vec::new(),
            metadata: None,
            pending_downloads: 0,
            feed: VirtualTimeline::new(0),
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
                self.metadata = None;
                self.avatar_bytes = None;
                self.pending_downloads = 1;
                self.feed.reset(0);

                let url = self.composer.clone();

                Task::perform(
                    download_and_parse_twtxt("unknown".into(), url.clone(), false),
                    move |result| Message::FeedLoaded { url, result },
                )
            }

            Message::FeedLoaded { url, result } => {
                self.pending_downloads -= 1;

                match result {
                    Ok(bundle) => {
                        println!("Feed successfully loaded for {}", url);
                        self.metadata = bundle.metadata.clone();
                        self.tweets = bundle.tweets;

                        self.tweets.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        self.feed.reset(self.tweets.len());

                        // Download avatar if present
                        if let Some(meta) = bundle.metadata {
                            if let Some(avatar_url) = meta.avatar {
                                self.pending_downloads += 1;
                                return Task::perform(download_binary(avatar_url), move |res| {
                                    Message::AvatarLoaded {
                                        url: url.clone(),
                                        result: res,
                                    }
                                });
                            }
                        }
                    }

                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }

                Task::none()
            }

            Message::AvatarLoaded { url, result } => {
                match result {
                    Ok(bytes) => {
                        println!("Avatar successfully loaded for {}", url);
                        let handle = Handle::from_bytes(bytes);
                        self.avatar_bytes = Some(handle.clone());

                        // Patch tweets
                        for tweet in self.tweets.iter_mut() {
                            if tweet.url == url {
                                tweet.avatar = handle.clone();
                            }
                        }
                    }

                    Err(e) => {
                        println!("Error: {}", e);
                    }
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

            Message::Feed(feed::Message::RedirectToPage(info)) => {
                Task::done(Message::RedirectToPage(info))
            }

            Message::Feed(msg) => self.feed.update(msg, self.tweets.len()).map(Message::Feed),

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
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
        let following = self
            .metadata
            .as_ref()
            .and_then(|m| m.following.as_ref())
            .cloned()
            .unwrap_or(0);
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

        let scroll = column![
            row![
                avatar,
                row![
                    column![
                        text(nick).size(24),
                        text(desc),
                        text(format!("Following: {}", following))
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
            .padding(32),
            timeline
        ]
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
                Task::done(Message::ViewPressed)
            }
            _ => Task::none(),
        }
    }
}
