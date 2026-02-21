use iced::{
    Element, Length, Task,
    widget::{Id, scrollable},
};

use crate::utils::{Tweet, build_feed};

const BATCH_SIZE: usize = 25;

const INITIAL_LOAD: usize = 30;

const LOAD_THRESHOLD: f32 = 200.0;

const TOP_THRESHOLD: f32 = 5.0;

#[derive(Debug, Clone)]
pub enum Message {
    Scrolled(scrollable::Viewport),
    LinkClicked(String),
    RedirectToPage(crate::app::RedirectInfo),
}

pub struct VirtualTimeline {
    scroll_id: Id,
    visible_count: usize,
}

impl VirtualTimeline {
    pub fn new(total_items: usize) -> Self {
        Self {
            scroll_id: Id::unique(),
            visible_count: INITIAL_LOAD.min(total_items),
        }
    }

    pub fn reset(&mut self, total_items: usize) {
        self.visible_count = INITIAL_LOAD.min(total_items);
    }

    pub fn update(&mut self, message: Message, total_items: usize) -> Task<Message> {
        match message {
            Message::Scrolled(viewport) => {
                let offset = viewport.absolute_offset().y;
                let visible_height = viewport.bounds().height;
                let total_height = viewport.content_bounds().height;

                let near_top = offset <= TOP_THRESHOLD;

                if near_top {
                    self.visible_count = INITIAL_LOAD.min(total_items);
                }

                let near_bottom = offset + visible_height >= total_height - LOAD_THRESHOLD;

                if near_bottom && self.visible_count < total_items {
                    self.visible_count = (self.visible_count + BATCH_SIZE).min(total_items);
                }

                Task::none()
            }

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),

            Message::LinkClicked(url) => {
                if url.contains("twtxt") && url.ends_with(".txt") {
                    Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                        page: crate::app::Page::View,
                        content: url.clone(),
                    }))
                } else {
                    // Open the URL in the default browser
                    if let Err(err) = webbrowser::open(&url) {
                        println!("Error opening URL: {}", err);
                    }
                    Task::none()
                }
            }
        }
    }

    pub fn view<'a>(&'a self, tweets: &'a [Tweet]) -> Element<'a, Message> {
        let visible = &tweets[..self.visible_count.min(tweets.len())];

        scrollable(build_feed(visible, Message::LinkClicked))
            .id(self.scroll_id.clone())
            .on_scroll(Message::Scrolled)
            .height(Length::Fill)
            .into()
    }
}
