//! A tweet renderer component, responsible for displaying a single tweet line.

use crate::twtxt::Tweet;
use crate::twtxt::metadata::Link;
use crate::twtxt::parsing;
use crate::utils::download::download_binary;
use crate::{
    components::og_embed::{self, OgEmbedComponent},
    utils::{
        download::download_opengraph,
        is_file_url,
        styling::{sec_button_style, secondary_text},
    },
};

use bytes::Bytes;
use chrono::Local;
use iced::{
    Background, Border, Color, ContentFit, Element, Length, Padding, Pixels, Task, Theme,
    border::Radius,
    widget::{
        Image, button, column, container,
        image::Handle,
        markdown::{self, Highlight},
        rich_text, row, space, span, text,
    },
};
use opengraph::Object;
use tracing::error;

use std::collections::HashMap;

/// Messages emitted by the tweet component.
#[derive(Debug, Clone)]
pub enum Message {
    /// A link inside the tweet markdown was clicked.
    LinkClicked(String),
    /// The reply button was clicked.
    ReplyClicked(usize),
    /// The thread button was clicked.
    ThreadClicked(usize),
    /// An image inside the tweet finished downloading.
    ImageLoaded(usize, Box<Result<Bytes, String>>), // usize = index into image_urls
    /// An object was loaded from a URL's OpenGraph metadata.
    OgObjectLoaded(usize, Box<Result<Object, String>>, String), // usize = index into urls
    OgEmbed(usize, crate::components::og_embed::Message),
}

/// A widget that renders a single tweet, including inline images and avatar.
pub struct TweetComponent {
    pub index: usize,
    image_urls: Vec<String>,
    image_handles: Vec<Option<(Handle, u32, u32)>>,
    og_objects: Vec<Option<Object>>,
    og_embeds: Vec<Option<OgEmbedComponent>>,
}

impl TweetComponent {
    pub fn new(index: usize, tweets: &[Tweet]) -> (Self, Task<Message>) {
        let tweet = &tweets[index];
        let image_urls = collect_image_urls(&tweet.md_items);
        let image_handles = vec![None; image_urls.len()];
        let urls = collect_urls(&tweet.content);
        let og_objects = vec![None; urls.len()];
        let og_embeds = vec![None; urls.len()];

        let img_tasks: Vec<Task<Message>> = image_urls
            .iter()
            .enumerate()
            .map(|(i, url)| {
                let url = url.clone();
                Task::perform(download_binary(url), move |res| {
                    Message::ImageLoaded(i, Box::new(res))
                })
            })
            .collect();

        let url_tasks: Vec<Task<Message>> = urls
            .iter()
            .enumerate()
            .map(|(i, url)| {
                let url = url.clone();
                Task::perform(download_opengraph(url.url.clone()), move |res| {
                    Message::OgObjectLoaded(i, Box::new(res), url.url)
                })
            })
            .collect();

        let tasks = img_tasks.into_iter().chain(url_tasks).collect::<Vec<_>>();

        (
            Self {
                index,
                image_urls,
                image_handles,
                og_objects,
                og_embeds,
            },
            Task::batch(tasks),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LinkClicked(url) => Task::done(Message::LinkClicked(url)),
            Message::ReplyClicked(index) => Task::done(Message::ReplyClicked(index)),
            Message::ThreadClicked(index) => Task::done(Message::ThreadClicked(index)),
            Message::ImageLoaded(i, result) => {
                match *result {
                    Ok(bytes) => {
                        if i < self.image_handles.len() {
                            let dims = image::load_from_memory(&bytes)
                                .ok()
                                .map(|img| (img.width(), img.height()));

                            self.image_handles[i] = Some(match dims {
                                Some((w, h)) => (Handle::from_bytes(bytes), w, h),
                                None => (Handle::from_bytes(bytes), 0, 0),
                            });
                        }
                    }
                    Err(e) => {
                        error!(
                            "Tweet: failed to load image {}: {}",
                            self.image_urls.get(i).map(String::as_str).unwrap_or("?"),
                            e
                        );
                    }
                }
                Task::none()
            }
            Message::OgObjectLoaded(i, result, url) => {
                match *result {
                    Ok(obj) => {
                        if i < self.og_objects.len() {
                            self.og_objects[i] = Some(obj.clone());
                        }
                        if i < self.og_embeds.len() {
                            let (embed, task) = OgEmbedComponent::new(&obj, &url);
                            self.og_embeds[i] = Some(embed);
                            return task.map(move |msg| Message::OgEmbed(i, msg));
                        }
                    }
                    Err(e) => {
                        error!("Tweet: failed to load OpenGraph object: {}", e);
                    }
                }
                Task::none()
            }
            Message::OgEmbed(i, msg) => {
                // Handle a link click by bubbling it up
                if let og_embed::Message::Clicked(url) = &msg {
                    return Task::done(Message::LinkClicked(url.clone()));
                }
                // Otherwise delegate to the embed component
                if let Some(Some(embed)) = self.og_embeds.get_mut(i) {
                    embed.update(msg).map(move |m| Message::OgEmbed(i, m))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        theme: &Theme,
        tweets: &'a [Tweet],
        avatars: &'a HashMap<String, Handle>,
        reply_available: bool,
    ) -> Element<'a, Message> {
        let tweet = &tweets[self.index];

        let code_bg = Color::from_rgba(0.0, 0.0, 0.0, 0.55);

        let content = markdown::view(
            &tweet.md_items,
            markdown::Settings::with_text_size(
                Pixels(12.0),
                markdown::Style {
                    font: crate::app::REGULAR_FONT,
                    link_color: theme.palette().primary,
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

        let handle = avatars.get(&tweet.feed_hash);
        let avatar_img: Element<Message> = if let Some(avatar) = handle {
            Image::new(avatar)
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0))
                .border_radius(24)
                .filter_method(iced::widget::image::FilterMethod::Linear)
                .into()
        } else {
            container(text("?").size(24))
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0))
                .center_x(Length::Fixed(48.0))
                .center_y(Length::Fixed(48.0))
                .style(|theme: &Theme| {
                    let ext = theme.extended_palette();
                    container::Style {
                        background: Some(Background::Color(ext.background.strong.color)),
                        border: Border {
                            radius: Radius::from(24.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                })
                .into()
        };

        let formatted_time = tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");

        let header = rich_text![
            span(&tweet.author)
                .font(crate::app::BOLD_FONT)
                .link(&tweet.url),
            span(" "),
            span(formatted_time.to_string()).color(secondary_text(theme)),
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
                Image::new(handle)
                    .width(Length::Fixed(MAX_WIDTH))
                    .height(Length::Fixed(render_h))
                    .content_fit(ContentFit::Contain)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
            } else {
                // fallback!
                Image::new(handle)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .content_fit(ContentFit::Contain)
                    .filter_method(iced::widget::image::FilterMethod::Linear)
            };
            images_col = images_col.push(image);
        }

        let mut og_embeds_col = column![].spacing(6);
        for (i, embed) in self.og_embeds.iter().enumerate() {
            if let Some(embed) = embed {
                og_embeds_col =
                    og_embeds_col.push(embed.view(theme).map(move |msg| Message::OgEmbed(i, msg)));
            }
        }

        let reply_button: Element<Message> = if reply_available {
            button("Reply")
                .style(sec_button_style)
                .padding([8.0, 16.0])
                .on_press(Message::ReplyClicked(self.index))
                .into()
        } else {
            space().into()
        };

        column![
            button(
                row![
                    avatar_img,
                    column![header, container(content), images_col, og_embeds_col]
                        .padding([6.0, 0.0])
                        .spacing(4)
                ]
                .spacing(12),
            )
            .on_press(Message::ThreadClicked(self.index))
            .width(Length::Fill)
            .padding(16)
            .style(sec_button_style),
            reply_button
        ]
        .spacing(8)
        .width(Length::Fill)
        .into()
    }
}

/// Recursively collects image URLs from markdown items.
fn collect_image_urls(items: &[markdown::Item]) -> Vec<String> {
    let mut urls = Vec::new();
    collect_image_urls_into(items, &mut urls);
    urls
}

/// Recursively collects image URLs from markdown items into the output vector.
fn collect_image_urls_into(items: &[markdown::Item], out: &mut Vec<String>) {
    for item in items {
        match item {
            markdown::Item::Image { url, .. } => out.push(url.clone()),
            markdown::Item::Quote(inner) => collect_image_urls_into(inner, out),
            _ => {}
        }
    }
}

fn collect_urls(text: &str) -> Vec<Link> {
    let url_re = parsing::get_url_re();
    let mut links = Vec::new();
    for cap in url_re.captures_iter(text) {
        if let Some(url) = cap.get(0) {
            let m = url.as_str();

            // Skip markdown images ![alt](url), feeds, and files
            if m.starts_with('!') || m.contains("twtxt.txt") || is_file_url(m) {
                continue;
            }

            // Extract URL from markdown hyperlink: [text](url)
            if m.contains("](") {
                let pair = m.split_once("](");
                if let Some((label, url)) = pair {
                    let url = url.trim_end_matches(')').to_string();
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        continue;
                    }
                    links.push(Link {
                        text: label.trim_start_matches('[').to_string(),
                        url,
                    });
                }
            } else {
                // Plain URL
                if !m.starts_with("http://") && !m.starts_with("https://") {
                    continue;
                }
                links.push(Link {
                    text: m.to_string(),
                    url: m.to_string(),
                });
            }
        }
    }
    links
}
