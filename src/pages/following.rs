use iced::{
    Alignment, Element, Length,
    widget::{button, column, row, text, text_input},
};

use crate::{config::AppConfig, utils::Link};

#[derive(Default)]
pub struct FollowingPage {
    pub new_name: String,
    pub new_url: String,

    pub editing: Option<String>,
    pub edit_name: String,
    pub edit_url: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    UrlChanged(String),
    AddPressed,
    RemovePressed(String),

    EditPressed(String),
    EditNameChanged(String),
    EditUrlChanged(String),
    SaveEdit,
    CancelEdit,
}

impl FollowingPage {
    pub fn update(&mut self, message: Message, config: &mut AppConfig) {
        match message {
            Message::NameChanged(v) => self.new_name = v,
            Message::UrlChanged(v) => self.new_url = v,

            Message::AddPressed => {
                if !self.new_name.is_empty() && !self.new_url.is_empty() {
                    config.metadata.follows.push(Link {
                        text: self.new_name.clone(),
                        url: self.new_url.clone(),
                    });

                    self.new_name.clear();
                    self.new_url.clear();
                    let _ = config.save();
                }
            }

            Message::RemovePressed(name) => {
                config.metadata.follows.retain(|l| l.text != name);
                let _ = config.save();
            }

            Message::EditPressed(name) => {
                if let Some(link) = config.metadata.follows.iter().find(|l| l.text == name) {
                    self.editing = Some(name.clone());
                    self.edit_name = link.text.clone();
                    self.edit_url = link.url.clone();
                }
            }

            Message::EditNameChanged(v) => self.edit_name = v,
            Message::EditUrlChanged(v) => self.edit_url = v,

            Message::SaveEdit => {
                if let Some(old_name) = self.editing.take() {
                    if let Some(link) = config
                        .metadata
                        .follows
                        .iter_mut()
                        .find(|l| l.text == old_name)
                    {
                        link.text = self.edit_name.clone();
                        link.url = self.edit_url.clone();
                        let _ = config.save();
                    }
                }
            }

            Message::CancelEdit => {
                self.editing = None;
            }
        }
    }

    pub fn view(&self, config: &AppConfig) -> Element<'_, Message> {
        let mut list = column!().spacing(8);

        for link in &config.metadata.follows {
            let name = &link.text;
            let url = &link.url;

            if self.editing.as_deref() == Some(name) {
                // Editing mode
                list = list.push(
                    row![
                        text_input("Name", &self.edit_name)
                            .on_input(Message::EditNameChanged)
                            .padding(8),
                        text_input("URL", &self.edit_url)
                            .on_input(Message::EditUrlChanged)
                            .padding(8),
                        button("Save").on_press(Message::SaveEdit).padding([8, 16]),
                        button("Cancel")
                            .on_press(Message::CancelEdit)
                            .padding([8, 16]),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            } else {
                // Normal mode
                list = list.push(
                    row![
                        text(format!(" {} → {}", name, url)).width(Length::Fill),
                        button("Edit")
                            .on_press(Message::EditPressed(name.clone()))
                            .padding([8, 16]),
                        button("Remove")
                            .on_press(Message::RemovePressed(name.clone()))
                            .padding([8, 16]),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }

        let add_section = row![
            text_input("Name", &self.new_name)
                .on_input(Message::NameChanged)
                .padding(8),
            text_input("URL", &self.new_url)
                .on_input(Message::UrlChanged)
                .padding(8),
            button("Add").on_press(Message::AddPressed).padding([8, 16]),
        ]
        .spacing(8);

        column![add_section, list].spacing(8).into()
    }
}
