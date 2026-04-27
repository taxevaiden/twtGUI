//! A tweet renderer component, responsible for displaying a single tweet line.

use crate::utils::{Tweet, download_binary, styling::sec_button_style};
use bytes::Bytes;
use chrono::Local;
use iced::{
    Background, Border, Color, ContentFit, Element, Length, Padding, Pixels, Task,
    widget::{
        Image, button, column, container,
        image::Handle,
        markdown::{self, Highlight},
        rich_text, row, space, span,
    },
};
use tracing::error;

/// Messages used by the tweet component.
#[derive(Debug, Clone)]
pub enum Message {
    /// A link inside the tweet markdown was clicked.
    LinkClicked(String),
    /// The reply button was clicked.
    ReplyClicked(usize),
    /// The thread button was clicked.
    ThreadClicked(usize),
    /// An image inside the tweet finished downloading.
    ImageLoaded(usize, Result<Bytes, String>), // usize = index into image_urls
}

/// A widget that renders a single tweet, including inline images and avatar.
pub struct TweetComponent {
    pub index: usize,
    image_urls: Vec<String>,
    image_handles: Vec<Option<(Handle, u32, u32)>>,
}

impl TweetComponent {
    pub fn new(index: usize, tweets: &[Tweet]) -> (Self, Task<Message>) {
        let tweet = &tweets[index];
        let image_urls = collect_image_urls(&tweet.md_items);
        let image_handles = vec![None; image_urls.len()];

        let tasks: Vec<Task<Message>> = image_urls
            .iter()
            .enumerate()
            .map(|(i, url)| {
                let url = url.clone();
                Task::perform(download_binary(url), move |res| {
                    Message::ImageLoaded(i, res)
                })
            })
            .collect();

        (
            Self {
                index,
                image_urls,
                image_handles,
            },
            Task::batch(tasks),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LinkClicked(url) => Task::done(Message::LinkClicked(url)),
            Message::ReplyClicked(index) => Task::done(Message::ReplyClicked(index)),
            Message::ThreadClicked(index) => Task::done(Message::ThreadClicked(index)),
            Message::ImageLoaded(i, Ok(bytes)) => {
                if i < self.image_handles.len() {
                    let dims = image::load_from_memory(&bytes).ok().map(|img| {
                        let (w, h) = (img.width(), img.height());
                        (w, h)
                    });

                    if let Some((w, h)) = dims {
                        self.image_handles[i] = Some((Handle::from_bytes(bytes), w, h));
                    } else {
                        // fallback!
                        self.image_handles[i] = Some((Handle::from_bytes(bytes), 0, 0));
                    }
                }
                Task::none()
            }
            Message::ImageLoaded(i, Err(e)) => {
                error!(
                    "Failed to load image {}: {}",
                    self.image_urls.get(i).map(String::as_str).unwrap_or("?"),
                    e
                );
                Task::none()
            }
        }
    }

    pub fn view<'a>(&'a self, tweets: &'a [Tweet]) -> Element<'a, Message> {
        let tweet = &tweets[self.index];

        let code_bg = Color::from_rgba(0.0, 0.0, 0.0, 0.08);

        let content = markdown::view(
            &tweet.md_items,
            markdown::Settings::with_text_size(
                Pixels(12.0),
                markdown::Style {
                    font: crate::app::REGULAR_FONT,
                    link_color: Color::from_rgb(0.4, 0.6, 1.0),
                    inline_code_font: crate::app::MONOSPACE_FONT,
                    inline_code_color: Color::from_rgb(0.85, 0.85, 0.85),
                    inline_code_highlight: Highlight {
                        background: Background::Color(code_bg),
                        border: Border::default(),
                    },
                    inline_code_padding: Padding::from(2.0),
                    code_block_font: crate::app::MONOSPACE_FONT,
                },
            ),
        )
        .map(Message::LinkClicked);

        let avatar_img = Image::new(tweet.avatar.clone())
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .border_radius(24)
            .filter_method(iced::widget::image::FilterMethod::Linear);

        let formatted_time = tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");

        let header = rich_text![
            span(&tweet.author)
                .font(crate::app::BOLD_FONT)
                .link(tweet.url.clone()),
            span(" "),
            span(formatted_time.to_string()).color(Color::from_rgba(1.0, 1.0, 1.0, 0.55)),
        ]
        .on_link_click(Message::LinkClicked);

        const MAX_WIDTH: f32 = 500.0;
        const MAX_HEIGHT: f32 = 400.0;

        let mut images_col = column![].spacing(4);
        for entry in self.image_handles.iter().flatten() {
            let (handle, img_w, img_h) = entry;
            let image = if *img_w > 0 && *img_h > 0 {
                let aspect = *img_h as f32 / *img_w as f32;
                let render_h = (MAX_WIDTH * aspect).min(MAX_HEIGHT);
                Image::new(handle.clone())
                    .width(Length::Fixed(MAX_WIDTH))
                    .height(Length::Fixed(render_h))
                    .content_fit(ContentFit::Contain)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
            } else {
                // fallback!
                Image::new(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .content_fit(ContentFit::Contain)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
            };
            images_col = images_col.push(image);
        }

        let reply_button = button("Reply")
            .style(sec_button_style)
            .padding([8.0, 16.0])
            .on_press(Message::ReplyClicked(self.index));
        let thread_button = button("Thread")
            .style(sec_button_style)
            .padding([8.0, 16.0])
            .on_press(Message::ThreadClicked(self.index));

        column![
            row![
                avatar_img,
                column![
                    header,
                    container(content),
                    images_col,
                    space().height(4),
                    row![reply_button, thread_button].spacing(8)
                ]
                .padding([6.0, 0.0])
                .spacing(4)
            ]
            .spacing(12),
        ]
        .spacing(8)
        .padding(4)
        .into()
    }
}

/// Recursively collects image URLs from markdown items.
fn collect_image_urls(items: &[markdown::Item]) -> Vec<String> {
    items
        .iter()
        .flat_map(|item| match item {
            markdown::Item::Image { url, .. } => vec![url.clone()],
            markdown::Item::Quote(inner) => collect_image_urls(inner),
            _ => vec![],
        })
        .collect()
}
