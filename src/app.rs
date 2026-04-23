//! Core application state and top-level view handling.
//!
//! This module wires the high-level pages together and manages the current
//! selected page, routing messages between sub-pages and updating the UI.
//!

use std::fmt;

use iced::{
    Background, Border, Color, Element, Length, Task, Theme,
    border::Radius,
    widget::{button, column, container, row, space, text},
};

use crate::config::AppConfig;
use crate::pages::{following, timeline, view};

/// The application state (model) used by `iced`.
///
/// Keeps the currently selected page and all of the page-specific state.
pub struct TwtxtApp {
    page: Page,
    config: AppConfig,
    timeline: timeline::TimelinePage,
    view: view::ViewPage,
    following: following::FollowingPage,
}

/// The set of messages that can be sent to the top-level application.
///
/// Messages are used by `iced` to drive updates and route events to submodules.
#[derive(Debug, Clone)]
pub enum Message {
    /// Switch to the timeline page
    SwitchToTimeline,
    /// Switch to the view page
    SwitchToView,
    /// Switch to the following page
    SwitchToFollowing,
    /// A message originating from the timeline page (forwarded)
    Timeline(timeline::Message),
    /// A message originating from the view page (forwarded)
    View(view::Message),
    /// A message originating from the following page (forwarded)
    Following(following::Message),
}

/// A simple top-level routing enum for the active page.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Page {
    /// Show the timeline feed.
    #[default]
    Timeline,
    /// Show a single tweet / thread view.
    View,
    /// Show the following list.
    Following,
}

/// Information used when a page wants to redirect the application to another page.
///
/// Currently only used by `ViewPage` to indicate which feed should be loaded and then shown.
#[derive(Debug, Clone)]
pub struct RedirectInfo {
    /// The target page to switch to.
    pub page: Page,

    /// Content relevant to the redirect target.
    ///
    /// For example, when switching to `View`, this holds the URL of a feed to display.
    pub content: String,
}

impl TwtxtApp {
    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load().expect("Failed to load config");
        let (timeline, timeline_task) = timeline::TimelinePage::new();
        let (view, view_task) = view::ViewPage::new(&config);
        (
            Self {
                page: Page::Timeline,
                config: config.clone(),
                timeline,
                view,
                following: following::FollowingPage::default(),
            },
            Task::batch([
                timeline_task.map(Message::Timeline),
                view_task.map(Message::View),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchToTimeline => {
                self.page = Page::Timeline;
                Task::none()
            }

            Message::SwitchToView => {
                self.page = Page::View;
                Task::none()
            }

            Message::SwitchToFollowing => {
                self.page = Page::Following;
                Task::none()
            }

            Message::Timeline(timeline::Message::RedirectToPage(info)) => {
                self.page = info.page.clone();
                match self.page {
                    Page::View => self.view.process_redirect_info(info).map(Message::View),
                    _ => Task::none(),
                }
            }

            Message::Timeline(msg) => self
                .timeline
                .update(msg, &self.config)
                .map(Message::Timeline),

            Message::View(view::Message::RedirectToPage(info)) => {
                self.page = info.page.clone();
                match self.page {
                    Page::View => self.view.process_redirect_info(info).map(Message::View),
                    _ => Task::none(),
                }
            }

            Message::View(msg) => self.view.update(msg).map(Message::View),

            Message::Following(msg) => {
                self.following.update(msg, &mut self.config);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        fn button_style(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
            move |theme, status| {
                let palette = theme.palette();
                let ext = theme.extended_palette();

                let bg = match status {
                    button::Status::Hovered => ext.background.weak.color,
                    button::Status::Pressed => ext.background.strong.color,
                    _ => ext.background.base.color,
                };

                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: if is_active {
                        palette.text
                    } else {
                        Color {
                            a: 0.5,
                            ..palette.text
                        }
                    },
                    border: Border {
                        radius: Radius::from(8.0),
                        // Yes, I know this border radius isn't the inner radius plus the padding
                        // It should be 10 but that looks ugly, and an inner radius of 2
                        // (which is what every widget uses in the container) is too subtle to notice the inconsistency
                        width: 1.0,
                        color: iced::Color::TRANSPARENT,
                    },
                    ..Default::default()
                }
            }
        }

        fn container_style(theme: &Theme) -> container::Style {
            let ext = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(ext.background.weak.color)),
                border: Border {
                    radius: Radius::from(8.0),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }

        let nav = column![
            button("Timeline")
                .on_press(Message::SwitchToTimeline)
                .padding([8, 16])
                .style(button_style(self.page == Page::Timeline))
                .width(Length::Fill),
            button("View")
                .on_press(Message::SwitchToView)
                .padding([8, 16])
                .style(button_style(self.page == Page::View))
                .width(Length::Fill),
            button("Following")
                .on_press(Message::SwitchToFollowing)
                .padding([8, 16])
                .style(button_style(self.page == Page::Following))
                .width(Length::Fill),
            space().height(Length::Fill),
            text(env!("BUILD_VERSION"))
        ]
        .spacing(8)
        .width(Length::Fixed(175.0));

        let content = match self.page {
            Page::Timeline => self.timeline.view().map(Message::Timeline),
            Page::View => self.view.view().map(Message::View),
            Page::Following => self.following.view(&self.config).map(Message::Following),
        };

        row![
            nav,
            container(content)
                .height(Length::Fill)
                .style(container_style)
                .padding(8)
        ]
        .spacing(8)
        .padding(8)
        .into()
    }
}
