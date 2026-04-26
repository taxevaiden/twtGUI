//! A card displaying a user's profile, with their avatar and name.
//! Displayed on the sidebar.

use crate::utils::{download_binary, styling::prim_button_style};
use bytes::Bytes;
use iced::{
    Element, Length, Task,
    widget::{button, container, image::Handle, row, text},
};
use tracing::error;

#[derive(Debug, Clone)]
pub enum Message {
    /// The user has clicked on the card.
    UserClicked(),
    /// An avatar image has finished downloading.
    AvatarLoaded(String, Result<Bytes, String>),
    /// Navigate to another page.
    RedirectToPage(crate::app::RedirectInfo),
}

/// Card displaying a user's profile, with their avatar and name.
pub struct UserCard {
    user: String,
    user_url: Option<String>,
    avatar: Option<Handle>,
}

impl UserCard {
    pub fn new(
        user: String,
        user_url: Option<String>,
        avatar_url: Option<String>,
    ) -> (Self, Task<Message>) {
        let task = if let Some(url) = avatar_url {
            Task::perform(download_binary(url.clone()), move |res| {
                Message::AvatarLoaded(url, res)
            })
        } else {
            Task::none()
        };
        (
            Self {
                user,
                user_url,
                avatar: None,
            },
            task,
        )
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::AvatarLoaded(_, Ok(bytes)) => {
                if self.avatar.is_none() {
                    self.avatar = Some(Handle::from_bytes(bytes));
                }
                Task::none()
            }

            Message::AvatarLoaded(url, Err(e)) => {
                error!("Failed to load image {}: {}", url, e);
                Task::none()
            }

            Message::UserClicked() => {
                Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                    page: crate::app::Page::View,
                    content: self.user_url.clone().unwrap(),
                }))
            }

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let avatar: Element<'_, Message> = match &self.avatar {
            Some(handle) => iced::widget::image(handle)
                .width(32)
                .height(32)
                .border_radius(16)
                .into(),
            None => iced::widget::space().width(32).into(),
        };

        let username = text(self.user.clone()).font(crate::app::BOLD_FONT);

        if self.user_url.is_none() {
            container(
                row![avatar, username]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
            )
            .padding([8, 16])
            .width(Length::Fill)
            .into()
        } else {
            button(
                row![avatar, username]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
            )
            .on_press(Message::UserClicked())
            .style(prim_button_style)
            .padding([8, 16])
            .width(Length::Fill)
            .into()
        }
    }
}
