//! Core application state and top-level view handling.
//!
//! This module wires the high-level pages together and manages the current
//! selected page, routing messages between sub-pages and updating the UI.
//!

use iced::{
    Background, Border, Element, Length, Task, Theme,
    border::Radius,
    font,
    widget::{
        Id, button, column, container, operation::snap_to, row, scrollable,
        scrollable::RelativeOffset, space, text,
    },
};

static LOG_SCROLL_ID: std::sync::LazyLock<Id> = std::sync::LazyLock::new(Id::unique);

use crate::logging::LogBuffer;
use crate::{
    components::user_card,
    pages::{following, timeline, view},
    utils::styling::tab_style,
};
use crate::{components::user_card::UserCard, config::AppConfig};

/// The application state (model) used by `iced`.
///
/// Keeps the currently selected page and all of the page-specific state.
pub struct TwtxtApp {
    page: Page,
    config: AppConfig,
    timeline: timeline::TimelinePage,
    view: view::ViewPage,
    following: following::FollowingPage,
    user_card: UserCard,
    log_buffer: LogBuffer,
    log_lines: Vec<String>,
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
    /// Switch to the logs page
    SwitchToLogs,
    Tick,
    /// A message originating from the timeline page (forwarded)
    Timeline(timeline::Message),
    /// A message originating from the view page (forwarded)
    View(view::Message),
    /// A message originating from the following page (forwarded)
    Following(following::Message),
    /// A message originating from the user card (forwarded)
    UserCard(user_card::Message),
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
    /// Show the logs.
    Logs,
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

/// The regular font used throughout the application.
pub static REGULAR_FONT: font::Font = font::Font::with_name("Iosevka Aile");
/// The bold font used throughout the application.
pub static BOLD_FONT: font::Font = font::Font {
    family: font::Family::Name("Iosevka Aile"),
    weight: font::Weight::Bold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
/// The monospace font used throughout the application.
/// Mainly used in code blocks.
pub static MONOSPACE_FONT: font::Font = font::Font::with_name("Iosevka");

impl TwtxtApp {
    pub fn new(log_buffer: LogBuffer) -> (Self, Task<Message>) {
        let config = AppConfig::load().expect("Failed to load config");
        let (timeline, timeline_task) = timeline::TimelinePage::new();
        let (view, view_task) = view::ViewPage::new(&config);
        let (user_card, user_card_task) = UserCard::new(
            config
                .metadata
                .nick
                .clone()
                .unwrap_or("unknown".to_string()),
            config.metadata.urls.first().cloned(),
            config.metadata.avatar.clone(),
        );
        (
            Self {
                page: Page::Timeline,
                config: config.clone(),
                timeline,
                view,
                following: following::FollowingPage::default(),
                user_card,
                log_buffer,
                log_lines: Vec::new(),
            },
            Task::batch([
                timeline_task.map(Message::Timeline),
                view_task.map(Message::View),
                user_card_task.map(Message::UserCard),
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

            Message::SwitchToLogs => {
                self.page = Page::Logs;
                Task::none()
            }

            Message::Tick => {
                let prev_len = self.log_lines.len();
                if let Ok(mut buf) = self.log_buffer.lock() {
                    self.log_lines.extend(buf.drain(..));
                }
                if self.log_lines.len() > 500 {
                    let excess = self.log_lines.len() - 500;
                    self.log_lines.drain(..excess);
                }

                if self.log_lines.len() != prev_len {
                    return snap_to(LOG_SCROLL_ID.clone(), RelativeOffset::END);
                }
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

            Message::UserCard(user_card::Message::RedirectToPage(info)) => {
                self.page = info.page.clone();
                match self.page {
                    Page::View => self.view.process_redirect_info(info).map(Message::View),
                    _ => Task::none(),
                }
            }

            Message::UserCard(msg) => self.user_card.update(msg).map(Message::UserCard),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.page == Page::Logs {
            iced::time::every(std::time::Duration::from_millis(200)).map(|_| Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    fn view_logs(&self) -> Element<'_, Message> {
        let lines = self
            .log_lines
            .iter()
            .fold(column![].spacing(2), |col, line| {
                col.push(text(line).font(MONOSPACE_FONT))
            });

        scrollable(lines)
            .id(LOG_SCROLL_ID.clone())
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
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
                .style(tab_style(self.page == Page::Timeline))
                .width(Length::Fill),
            button("View")
                .on_press(Message::SwitchToView)
                .padding([8, 16])
                .style(tab_style(self.page == Page::View))
                .width(Length::Fill),
            button("Following")
                .on_press(Message::SwitchToFollowing)
                .padding([8, 16])
                .style(tab_style(self.page == Page::Following))
                .width(Length::Fill),
            button("Logs")
                .on_press(Message::SwitchToLogs)
                .padding([8, 16])
                .style(tab_style(self.page == Page::Logs))
                .width(Length::Fill),
            space().height(Length::Fill),
            self.user_card.view().map(Message::UserCard),
            container(text(env!("BUILD_VERSION")))
                .padding([8, 16])
                .width(Length::Fill),
        ]
        .spacing(8)
        .width(Length::Fixed(175.0));

        let content = match self.page {
            Page::Timeline => self.timeline.view().map(Message::Timeline),
            Page::View => self.view.view().map(Message::View),
            Page::Following => self.following.view(&self.config).map(Message::Following),
            Page::Logs => self.view_logs(), // We could make Logs its own separate page struct,
                                            // But it makes more sense to implement this way
                                            // Not like it owns any data and has an update fn we just give data to it
        };

        column![
            space().height(if cfg!(target_os = "macos") { 16 } else { 8 }),
            row![
                nav,
                container(content)
                    .height(Length::Fill)
                    .style(container_style)
                    .padding(8)
            ]
            .spacing(8)
            .padding(8)
        ]
        .spacing(8)
        .into()
    }
}
