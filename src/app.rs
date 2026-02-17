use iced::{
    Element, Task,
    widget::{button, column, row},
};

use crate::pages::{settings, timeline};
use crate::{config::AppConfig, pages::view};

pub struct TwtxtApp {
    page: Page,
    config: AppConfig,
    timeline: timeline::TimelinePage,
    view: view::ViewPage,
    settings: settings::SettingsPage,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchToTimeline,
    SwitchToView,
    SwitchToSettings,
    Timeline(timeline::Message),
    View(view::Message),
    Settings(settings::Message),
}

#[derive(Default)]
enum Page {
    #[default]
    Timeline,
    View,
    Settings,
}

impl TwtxtApp {
    pub fn new() -> Self {
        let config = AppConfig::load();
        Self {
            page: Page::Timeline,
            config: config.clone(),
            timeline: timeline::TimelinePage::new(config.clone()),
            view: view::ViewPage::new(config.clone()),
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

            Message::SwitchToSettings => {
                self.page = Page::Settings;
                Task::none()
            }

            Message::Timeline(msg) => {
                self.timeline.update(msg);
                Task::none()
            }

            Message::View(msg) => self.view.update(msg).map(Message::View),

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
            button("Settings")
                .on_press(Message::SwitchToSettings)
                .padding([8, 16]),
        ]
        .spacing(8);

        let content = match self.page {
            Page::Timeline => self.timeline.view().map(Message::Timeline),
            Page::View => self.view.view().map(Message::View),
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
