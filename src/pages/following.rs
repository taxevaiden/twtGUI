//! A page for managing the list of followed feeds.

use iced::{
    Alignment, Element, Length,
    widget::{button, column, row, text, text_input},
};

use crate::{config::AppConfig, utils::Link};

/// State for the following page.
///
/// Holds the current editing state and the in-progress form fields.
#[derive(Default)]
pub struct FollowingPage {
    pub new_name: String,
    pub new_url: String,

    pub editing: Option<String>,
    pub edit_name: String,
    pub edit_url: String,
}

/// Messages used to update the following page.
#[derive(Debug, Clone)]
pub enum Message {
    /// The new follow name changed.
    NameChanged(String),
    /// The new follow URL changed.
    UrlChanged(String),
    /// Add the new follow item.
    AddPressed,
    /// Remove an existing follow item.
    RemovePressed(String),

    /// Start editing an existing follow item.
    EditPressed(String),
    /// Change the edit name field.
    EditNameChanged(String),
    /// Change the edit URL field.
    EditUrlChanged(String),
    /// Save the edited follow item.
    SaveEdit,
    /// Cancel the current edit.
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
                if let Some(old_name) = self.editing.take()
                    && let Some(link) = config
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
                            .width(Length::Fill)
                            .padding(8),
                        text_input("URL", &self.edit_url)
                            .on_input(Message::EditUrlChanged)
                            .width(Length::FillPortion(2))
                            .padding(8),
                        row![
                            button(text("Save").align_x(Alignment::Center).width(Length::Fill))
                                .on_press(Message::SaveEdit)
                                .width(Length::Fill)
                                .padding([8, 16]),
                            button(
                                text("Cancel")
                                    .align_x(Alignment::Center)
                                    .width(Length::Fill)
                            )
                            .on_press(Message::CancelEdit)
                            .width(Length::Fill)
                            .padding([8, 16]),
                        ]
                        .width(Length::Fixed(175.0))
                        .spacing(8)
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            } else {
                // Normal mode
                list = list.push(
                    row![
                        row![
                            text(name.to_string()).width(Length::Fill),
                            text(url.to_string()).width(Length::FillPortion(2))
                        ]
                        .spacing(16)
                        .padding(8),
                        row![
                            button(text("Edit").align_x(Alignment::Center).width(Length::Fill))
                                .on_press(Message::EditPressed(name.clone()))
                                .width(Length::Fill)
                                .padding([8, 16]),
                            button(
                                text("Remove")
                                    .align_x(Alignment::Center)
                                    .width(Length::Fill)
                            )
                            .on_press(Message::RemovePressed(name.clone()))
                            .width(Length::Fill)
                            .padding([8, 16]),
                        ]
                        .width(Length::Fixed(175.0))
                        .spacing(8)
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }

        let add_section = row![
            text_input("Name", &self.new_name)
                .on_input(Message::NameChanged)
                .width(Length::Fill)
                .padding(8),
            text_input("URL", &self.new_url)
                .on_input(Message::UrlChanged)
                .width(Length::FillPortion(2))
                .padding(8),
            button(text("Add").align_x(Alignment::Center).width(Length::Fill))
                .on_press(Message::AddPressed)
                .width(Length::Fixed(175.0))
                .padding([8, 16]),
        ]
        .spacing(8);

        column![add_section, list].spacing(8).into()
    }
}
