use iced::{Element, widget::{column, text}};

#[derive(Default)]
pub struct SettingsPage;

#[derive(Debug, Clone)]
pub enum Message {}

impl SettingsPage {
    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        column![text("Settings Page")].into()
    }
}
