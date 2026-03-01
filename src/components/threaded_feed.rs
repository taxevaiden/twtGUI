use crate::utils::{Tweet, TweetNode, build_threaded_feed};
use iced::{
    Element, Length, Task,
    widget::{Id, scrollable},
};

const BATCH_SIZE: usize = 10; // threads can be large, so smaller batches are safer
const INITIAL_LOAD: usize = 25;
const LOAD_THRESHOLD: f32 = 400.0;
const TOP_THRESHOLD: f32 = 10.0;

#[derive(Debug, Clone)]
pub enum Message {
    Scrolled(scrollable::Viewport),
    LinkClicked(String),
    RedirectToPage(crate::app::RedirectInfo),
}

pub struct LazyThreadedFeed {
    scroll_id: Id,
    visible_threads_count: usize,
}

impl LazyThreadedFeed {
    pub fn new(total_threads: usize) -> Self {
        Self {
            scroll_id: Id::unique(),
            visible_threads_count: INITIAL_LOAD.min(total_threads),
        }
    }

    pub fn reset(&mut self, total_threads: usize) {
        self.visible_threads_count = INITIAL_LOAD.min(total_threads);
    }

    pub fn update(&mut self, message: Message, total_threads: usize) -> Task<Message> {
        match message {
            Message::Scrolled(viewport) => {
                let offset = viewport.absolute_offset().y;
                let visible_height = viewport.bounds().height;
                let total_height = viewport.content_bounds().height;

                if offset <= TOP_THRESHOLD {
                    self.visible_threads_count = INITIAL_LOAD.min(total_threads);
                }

                let near_bottom = offset + visible_height >= total_height - LOAD_THRESHOLD;

                if near_bottom && self.visible_threads_count < total_threads {
                    self.visible_threads_count =
                        (self.visible_threads_count + BATCH_SIZE).min(total_threads);
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
                    if let Err(err) = webbrowser::open(&url) {
                        eprintln!("Error opening URL: {}", err);
                    }
                    Task::none()
                }
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        threads: &'a [TweetNode],
        tweets: &'a [Tweet],
    ) -> Element<'a, Message> {
        let visible_threads = &threads[..self.visible_threads_count.min(threads.len())];

        scrollable(build_threaded_feed(
            visible_threads,
            tweets,
            Message::LinkClicked,
        ))
        .id(self.scroll_id.clone())
        .on_scroll(Message::Scrolled)
        .height(Length::Fill)
        .into()
    }
}
