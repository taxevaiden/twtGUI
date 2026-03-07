use crate::utils::Tweet;
use chrono::Local;
use iced::{
    widget::{container, row, column, Image, rich_text, span},
    Element, Length, font, Color, Background, Border, Padding, Pixels,
    widget::markdown::{self, Highlight},
};

#[derive(Debug, Clone)]
pub enum Message {
    LinkClicked(String),
}

pub struct TweetComponent<'a> {
    pub tweet: &'a Tweet,
}

impl<'a> TweetComponent<'a> {
    pub fn new(tweet: &'a Tweet) -> Self {
        Self { tweet }
    }

    pub fn view(&self) -> Element<'a, Message> {
        let reg = font::Font::with_name("Iosevka Aile");
        let mut bold = font::Font::with_name("Iosevka Aile");
        bold.weight = font::Weight::Bold;

        let bg = iced::Theme::CatppuccinMocha.palette().background;
        let code_bg = Color::from_rgb(bg.r * 0.75, bg.g * 0.75, bg.b * 0.75);

        let content = markdown::view(
            &self.tweet.md_items,
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

        let avatar_img = Image::new(self.tweet.avatar.clone())
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .border_radius(20);

        let formatted_time = self.tweet
            .timestamp
            .with_timezone(&Local)
            .format("%h %-d %Y %-I:%M %p");

        let header = rich_text![
            span(&self.tweet.author).font(bold).link(self.tweet.url.clone()),
            span(" - "),
            span(formatted_time.to_string()),
            span(" "),
            span(self.tweet.hash.clone())
        ]
        .on_link_click(Message::LinkClicked);

        container(
            row![
                avatar_img,
                column![header, container(content)]
                    .padding([4.0, 0.0])
                    .spacing(4)
            ]
            .spacing(8),
        )
        .padding(4)
        .into()
    }
}