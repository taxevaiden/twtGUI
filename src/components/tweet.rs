//! A tweet renderer component, responsible for displaying a single tweet line.

use crate::utils::{Tweet, download_binary};
use bytes::Bytes;
use chrono::Local;
use iced::{
    Background, Border, Color, Element, Length, Padding, Pixels, Task, font,
    widget::markdown::{self, Highlight},
    widget::{Image, column, container, image::Handle, rich_text, row, span},
};

/// Messages used by the tweet component.
#[derive(Debug, Clone)]
pub enum Message {
    /// A link inside the tweet markdown was clicked.
    LinkClicked(String),
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

        let reg = font::Font::with_name("Iosevka Aile");
        let mut bold = font::Font::with_name("Iosevka Aile");
        bold.weight = font::Weight::Bold;

        let bg = iced::Theme::CatppuccinMocha.palette().background;
        let code_bg = Color::from_rgb(bg.r * 0.75, bg.g * 0.75, bg.b * 0.75);

        let content = markdown::view(
            &tweet.md_items,
            markdown::Settings::with_text_size(
                Pixels(12.0),
                markdown::Style {
                    font: reg,
                    link_color: Color::from_rgb(0.4, 0.6, 1.0),
                    inline_code_font: reg,
                    inline_code_color: Color::from_rgb(0.85, 0.85, 0.85),
                    inline_code_highlight: Highlight {
                        background: Background::Color(code_bg),
                        border: Border::default(),
                    },
                    inline_code_padding: Padding::from(2.0),
                    code_block_font: reg,
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
            span(&tweet.author).font(bold).link(tweet.url.clone()),
            span(" - "),
            span(formatted_time.to_string()),
            span(" "),
            span(tweet.hash.clone())
        ]
        .on_link_click(Message::LinkClicked);

        // inline loaded images below the markdown content
        let mut images_col = column![].spacing(4);
        for handle in self.image_handles.iter().flatten() {
            images_col = images_col.push(
                Image::new(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fixed(500.0))
                    .border_radius(4),
            );
        }

        container(
            row![
                avatar_img,
                column![header, container(content), images_col]
                    .padding([6.0, 0.0])
                    .spacing(4)
            ]
            .spacing(12),
        )
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
