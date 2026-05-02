//! OpenGraph embed card component.

use crate::utils::{
    download_binary,
    styling::{prim_button_style, secondary_text},
};
use bytes::Bytes;
use iced::{
    Element, Length, Task, Theme,
    widget::{Image, column, container, image::Handle, row, text},
};
use opengraph::Object;
use tracing::error;
use url::Url;

/// Messages emitted by the embed card component.
#[derive(Debug, Clone)]
pub enum Message {
    /// The embed was clicked.
    Clicked(String),
    /// The preview image finished downloading.
    ImageLoaded(Box<Result<Bytes, String>>),
}

#[derive(Clone)]
pub struct OgEmbedComponent {
    title: String,
    description: String,
    site_name: String,
    url: String,
    image_url: Option<String>,
    image_handle: Option<(Handle, u32, u32)>,
}

impl OgEmbedComponent {
    pub fn new(obj: &Object, url: &str) -> (Self, Task<Message>) {
        let url = if obj.url.is_empty() {
            url.to_string()
        } else {
            obj.url.clone()
        };
        let image_url = obj.images.first().and_then(|img| {
            let raw = &img.url;
            if raw.starts_with("http://") || raw.starts_with("https://") {
                Some(raw.clone())
            } else {
                Url::parse(&url)
                    .ok()
                    .and_then(|base| base.join(raw).ok())
                    .map(|u| u.to_string())
            }
        });

        let task = match &image_url {
            Some(url) => {
                let url = url.clone();
                Task::perform(download_binary(url), |res| {
                    Message::ImageLoaded(Box::new(res))
                })
            }
            None => Task::none(),
        };

        (
            Self {
                title: obj.title.clone(),
                description: obj.description.clone().unwrap_or_default(),
                site_name: obj.site_name.clone().unwrap_or_else(|| {
                    // Fall back to hostname from URL
                    Url::parse(&url)
                        .ok()
                        .and_then(|u| u.host_str().map(|h| h.to_string()))
                        .unwrap_or_default()
                }),
                url,
                image_url,
                image_handle: None,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Clicked(url) => Task::done(Message::Clicked(url)),
            Message::ImageLoaded(result) => {
                match *result {
                    Ok(bytes) => {
                        let dims = image::load_from_memory(&bytes)
                            .ok()
                            .map(|img| (img.width(), img.height()));
                        self.image_handle = Some(match dims {
                            Some((w, h)) => (Handle::from_bytes(bytes), w, h),
                            None => (Handle::from_bytes(bytes), 0, 0),
                        });
                    }
                    Err(e) => {
                        error!(
                            "OgEmbed: failed to load image {}: {}",
                            self.image_url.as_deref().unwrap_or("?"),
                            e
                        );
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        const MAX_WIDTH: f32 = 200.0;
        const MAX_HEIGHT: f32 = 200.0;

        let image: Option<Element<Message>> = self.image_handle.as_ref().map(|handle| {
            let (handle, img_w, img_h) = handle;
            if *img_w > 0 && *img_h > 0 {
                let aspect = *img_h as f32 / *img_w as f32;
                let render_h = (MAX_WIDTH * aspect).min(MAX_HEIGHT);
                Image::new(handle)
                    .width(Length::Fixed(MAX_WIDTH))
                    .height(Length::Fixed(render_h))
                    .content_fit(iced::ContentFit::Cover)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
                    .into()
            } else {
                Image::new(handle)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .content_fit(iced::ContentFit::Contain)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
                    .into()
            }
        });

        let site = text(&self.site_name);
        let title = text(&self.title).font(crate::app::BOLD_FONT);
        let desc = text(&self.description).color(secondary_text(theme));

        let text_block = column![site, title, desc].spacing(2).padding([8, 0]);

        let inner: Element<Message> = if let Some(img) = image {
            row![img, text_block].spacing(10).into()
        } else {
            text_block.into()
        };

        let url = self.url.clone();
        container(
            iced::widget::button(inner)
                .on_press(Message::Clicked(url))
                .style(prim_button_style)
                .padding(8)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }
}
