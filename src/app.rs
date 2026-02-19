use iced::{
    Element, Task,
    widget::{button, column, container, row},
};

use crate::config::AppConfig;
use crate::pages::{following, timeline, view};

pub struct TwtxtApp {
    page: Page,
    config: AppConfig,
    timeline: timeline::TimelinePage,
    view: view::ViewPage,
    following: following::FollowingPage,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchToTimeline,
    SwitchToView,
    SwitchToFollowing,
    Timeline(timeline::Message),
    View(view::Message),
    Following(following::Message),
}

#[derive(Debug, Clone, Default)]
pub enum Page {
    #[default]
    Timeline,
    View,
    Following,
}

#[derive(Debug, Clone)]
pub struct RedirectInfo {
    pub page: Page,

    // This information is specific to the ViewPage, however this is written in a way that it should be easy to implement this for other pages
    pub content: String,
}

impl TwtxtApp {
    pub fn new() -> Self {
        let config = AppConfig::load();
        Self {
            page: Page::Timeline,
            config: config.clone(),
            timeline: timeline::TimelinePage::new(),
            view: view::ViewPage::new(&config),
            following: following::FollowingPage::default(),
        }
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

impl Default for TwtxtApp {
    fn default() -> Self {
        Self::new()
    }
}
