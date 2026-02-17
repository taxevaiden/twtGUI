use iced::{
    Element, Task,
    widget::{button, column, row},
};

use crate::config::AppConfig;
use crate::pages::{following, settings, timeline, view};

pub struct TwtxtApp {
    page: Page,
    config: AppConfig,
    timeline: timeline::TimelinePage,
    view: view::ViewPage,
    following: following::FollowingPage,
    settings: settings::SettingsPage,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchToTimeline,
    SwitchToView,
    SwitchToFollowing,
    SwitchToSettings,
    Timeline(timeline::Message),
    View(view::Message),
    Following(following::Message),
    Settings(settings::Message),
}

#[derive(Default)]
enum Page {
    #[default]
    Timeline,
    View,
    Following,
    Settings,
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
            settings: settings::SettingsPage::default(),
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

            Message::SwitchToSettings => {
                self.page = Page::Settings;
                Task::none()
            }

            Message::Timeline(msg) => {
                self.timeline.update(msg, &self.config);
                Task::none()
            }

            Message::View(msg) => self.view.update(msg).map(Message::View),

            Message::Following(msg) => {
                self.following.update(msg);
                Task::none()
            }

            Message::Settings(msg) => {
                self.settings.update(msg);
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
            button("Settings")
                .on_press(Message::SwitchToSettings)
                .padding([8, 16]),
        ]
        .spacing(8);

        let content = match self.page {
            Page::Timeline => self.timeline.view().map(Message::Timeline),
            Page::View => self.view.view().map(Message::View),
            Page::Following => self.following.view(&self.config).map(Message::Following),
            Page::Settings => self.settings.view().map(Message::Settings),
        };

        column![nav, content].spacing(8).padding(8).into()
    }
}

impl Default for TwtxtApp {
    fn default() -> Self {
        Self::new()
    }
}
