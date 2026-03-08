//! Core application state and top-level view handling.
//!
//! This module wires the high-level pages together and manages the current
//! selected page, routing messages between sub-pages and updating the UI.

use iced::{
    Element, Task,
    widget::{button, column, container, row},
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
#[derive(Debug, Clone, Default)]
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
        let nav = row![
            button("Timeline")
                .on_press(Message::SwitchToTimeline)
                .padding([8, 16]),
            button("View")
                .on_press(Message::SwitchToView)
                .padding([8, 16]),
            button("Following")
                .on_press(Message::SwitchToFollowing)
                .padding([8, 16]),
        ]
        .spacing(8)
        .padding(8);

        let content = match self.page {
            Page::Timeline => self.timeline.view().map(Message::Timeline),
            Page::View => self.view.view().map(Message::View),
            Page::Following => self.following.view(&self.config).map(Message::Following),
        };

        column![nav, container(content).padding(8)].into()
    }
}
