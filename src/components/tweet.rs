//! A tweet renderer component, responsible for displaying a single tweet line.

use crate::utils::{Tweet, download_binary};
use bytes::Bytes;
use chrono::Local;
use iced::{
    Background, Border, Color, Element, Length, Padding, Pixels, Task,
    border::Radius,
    font,
    widget::{
        Image, Theme, button, column, container,
        image::Handle,
        markdown::{self, Highlight},
        rich_text, row, space, span,
    },
};

static REGULAR_FONT: font::Font = font::Font::with_name("Iosevka Aile");
static BOLD_FONT: font::Font = font::Font {
    family: font::Family::Name("Iosevka Aile"),
    weight: font::Weight::Bold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

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
    image_handles: Vec<Option<Handle>>, // None = still loading
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
                    self.image_handles[i] = Some(Handle::from_bytes(bytes));
                }
                Task::none()
            }
            Message::ImageLoaded(i, Err(e)) => {
                eprintln!(
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
                    font: REGULAR_FONT,
                    link_color: Color::from_rgb(0.4, 0.6, 1.0),
                    inline_code_font: REGULAR_FONT,
                    inline_code_color: Color::from_rgb(0.85, 0.85, 0.85),
                    inline_code_highlight: Highlight {
                        background: Background::Color(code_bg),
                        border: Border::default(),
                    },
                    inline_code_padding: Padding::from(2.0),
                    code_block_font: REGULAR_FONT,
                },
            ),
        )
        .map(Message::LinkClicked);

        let avatar_img = Image::new(tweet.avatar.clone())
            .width(Length::Fixed(48.0))
            .height(Length::Fixed(48.0))
            .border_radius(24);

        let formatted_time = tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");

        let header = rich_text![
            span(&tweet.author).font(BOLD_FONT).link(tweet.url.clone()),
            span(" - "),
            span(formatted_time.to_string()),
        ]
        .on_link_click(Message::LinkClicked);

        // inline loaded images below the markdown content
        let mut images_col = column![].spacing(4);
        for handle in self.image_handles.iter().flatten() {
            images_col = images_col.push(
                Image::new(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fixed(500.0)),
            );
        }

        fn button_style(theme: &Theme, status: button::Status) -> button::Style {
            let palette = theme.palette();
            let ext = theme.extended_palette();

            let bg = match status {
                button::Status::Hovered => ext.background.weaker.color,
                button::Status::Pressed => ext.background.stronger.color,
                _ => ext.background.weak.color,
            };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: palette.text,
                border: Border {
                    radius: Radius::from(4.0),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                ..Default::default()
            }
        }

        let reply_button = button("Reply")
            .style(button_style)
            .padding([8.0, 16.0])
            .on_press(Message::ReplyClicked(self.index));
        let thread_button = button("Thread")
            .style(button_style)
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
