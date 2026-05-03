//! A card displaying a user's profile, with their avatar and name.
//! Displayed on the sidebar.

use crate::utils::{download::download_binary, styling::prim_button_style};
use bytes::Bytes;
use iced::{
    Background, Border, Element, Length, Task, Theme,
    border::Radius,
    widget::{button, container, image::Handle, row, text},
};
use tracing::error;

#[derive(Debug, Clone)]
pub enum Message {
    /// The user has clicked on the card.
    UserClicked,
    /// An avatar image has finished downloading.
    AvatarLoaded(Box<Result<Bytes, String>>),
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
                Message::AvatarLoaded(Box::new(res))
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
            Message::AvatarLoaded(result) => match *result {
                Ok(bytes) => {
                    if self.avatar.is_none() {
                        self.avatar = Some(Handle::from_bytes(bytes));
                    }
                    Task::none()
                }
                Err(e) => {
                    error!("UserCard: failed to load image: {}", e);
                    Task::none()
                }
            },

            Message::UserClicked => {
                if let Some(url) = self.user_url.clone() {
                    Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                        page: crate::app::Page::View,
                        content: url,
                    }))
                } else {
                    Task::none()
                }
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
                .filter_method(iced::widget::image::FilterMethod::Linear)
                .into(),
            None => container(text("?").size(16))
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .center_x(Length::Fixed(32.0))
                .center_y(Length::Fixed(32.0))
                .style(|theme: &Theme| {
                    let ext = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(ext.background.strong.color)),
                        border: Border {
                            radius: Radius::from(16.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .into(),
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
            .on_press(Message::UserClicked)
            .style(prim_button_style)
            .padding([8, 16])
            .width(Length::Fill)
            .into()
        }
    }
}
