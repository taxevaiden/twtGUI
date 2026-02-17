use iced::{
    Element,
    widget::{column, text},
};

use crate::config::AppConfig;

#[derive(Default)]
pub struct FollowingPage;

#[derive(Debug, Clone)]
pub enum Message {}

impl FollowingPage {
    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self, config: &AppConfig) -> Element<'_, Message> {
        let mut col = column!().spacing(2);

        for (key, value) in config
            .following
            .as_ref()
            .unwrap_or(&std::collections::HashMap::new())
        {
            col = col.push(text(format!("{}: {}", key, value)));
        }

        column![text("Following Page"), col].into()
    }
}
