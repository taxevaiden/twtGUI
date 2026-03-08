//! A lazily-rendered threaded feed component.
//!
//! Renders a subset of threads and loads more as the user scrolls.

use crate::components::tweet::{self, TweetComponent};
use crate::utils::{Tweet, TweetNode};
use iced::border::Radius;
use iced::{Border, Theme};
use iced::{
    Element, Length, Task,
    widget::{Column, Id, column, container, row, scrollable, space},
};

/// How many additional threads to load when reaching the bottom of the scroll.
const BATCH_SIZE: usize = 10; // Threads can be large, so smaller batches are safer
/// How many threads to load initially.
const INITIAL_LOAD: usize = 25;
/// How close to the bottom of the scroll before loading more threads.
const LOAD_THRESHOLD: f32 = 400.0;
/// How close to the top of the scroll to reset to the initial load size.
const TOP_THRESHOLD: f32 = 10.0;

/// A memoized thread node with its rendered component and child threads.
struct BuiltNode {
    component: TweetComponent,
    children: Vec<BuiltNode>,
}

/// Messages emitted by the threaded feed component.
#[derive(Debug, Clone)]
pub enum Message {
    /// The scroll position changed.
    Scrolled(scrollable::Viewport),
    /// A link inside a tweet was clicked.
    LinkClicked(String),
    /// Request to navigate to another page.
    RedirectToPage(crate::app::RedirectInfo),
    /// A message coming from a specific tweet component.
    TweetMessage(usize, tweet::Message),
}

/// Lazy-loading threaded feed view.
///
/// This component renders tweet threads incrementally as the user scrolls.
pub struct LazyThreadedFeed {
    scroll_id: Id,
    visible_threads_count: usize,
    built_threads: Vec<BuiltNode>,
}

impl LazyThreadedFeed {
    pub fn new(threads: &[TweetNode], tweets: &[Tweet]) -> (Self, Task<Message>) {
        let (built, task) = build_nodes(threads, tweets);
        let total = built.len();
        (
            Self {
                scroll_id: Id::unique(),
                visible_threads_count: INITIAL_LOAD.min(total),
                built_threads: built,
            },
            task,
        )
    }

    pub fn reset(&mut self, threads: &[TweetNode], tweets: &[Tweet]) -> Task<Message> {
        let (built, task) = build_nodes(threads, tweets);
        self.built_threads = built;
        self.visible_threads_count = INITIAL_LOAD.min(self.built_threads.len());
        task
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let total = self.built_threads.len();
        match message {
            Message::Scrolled(viewport) => {
                let offset = viewport.absolute_offset().y;
                let visible_height = viewport.bounds().height;
                let total_height = viewport.content_bounds().height;

                if offset <= TOP_THRESHOLD {
                    self.visible_threads_count = INITIAL_LOAD.min(total);
                }

                let near_bottom = offset + visible_height >= total_height - LOAD_THRESHOLD;
                if near_bottom && self.visible_threads_count < total {
                    self.visible_threads_count =
                        (self.visible_threads_count + BATCH_SIZE).min(total);
                }

                Task::none()
            }

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),

            Message::LinkClicked(url) => {
                if url.contains("twtxt") && url.ends_with(".txt") {
                    Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                        page: crate::app::Page::View,
                        content: url,
                    }))
                } else {
                    if let Err(err) = webbrowser::open(&url) {
                        eprintln!("Error opening URL: {}", err);
                    }
                    Task::none()
                }
            }

            Message::TweetMessage(_, tweet::Message::LinkClicked(url)) => {
                Task::done(Message::LinkClicked(url))
            }

            Message::TweetMessage(index, msg) => {
                if let Some(node) = find_node_mut(&mut self.built_threads, index) {
                    node.component
                        .update(msg)
                        .map(move |m| Message::TweetMessage(index, m))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view<'a>(&'a self, tweets: &'a [Tweet]) -> Element<'a, Message> {
        let visible =
            &self.built_threads[..self.visible_threads_count.min(self.built_threads.len())];

        fn container_style(theme: &Theme) -> container::Style {
            container::Style {
                border: Border {
                    color: theme.palette().text,
                    width: 1.0,
                    radius: Radius::new(8.0),
                },
                ..Default::default()
            }
        }

        let mut col = column!().spacing(8);
        for node in visible {
            col = col.push(
                container(render_built_node(node, tweets))
                    .width(Length::Fill)
                    .padding(12.0)
                    .style(container_style),
            );
        }

        scrollable(col)
            .id(self.scroll_id.clone())
            .spacing(8)
            .on_scroll(Message::Scrolled)
            .height(Length::Fill)
            .into()
    }
}

fn find_node_mut(nodes: &mut Vec<BuiltNode>, index: usize) -> Option<&mut BuiltNode> {
    for node in nodes.iter_mut() {
        if node.component.index == index {
            return Some(node);
        }
        if let Some(found) = find_node_mut(&mut node.children, index) {
            return Some(found);
        }
    }
    None
}

fn build_nodes(threads: &[TweetNode], tweets: &[Tweet]) -> (Vec<BuiltNode>, Task<Message>) {
    let (nodes, tasks): (Vec<_>, Vec<_>) =
        threads.iter().map(|node| build_node(node, tweets)).unzip();

    (nodes, Task::batch(tasks))
}

fn build_node(node: &TweetNode, tweets: &[Tweet]) -> (BuiltNode, Task<Message>) {
    let index = node.index;
    let (component, task) = TweetComponent::new(index, tweets);
    let task = task.map(move |msg| Message::TweetMessage(index, msg));

    let mut sorted_children = node.children.clone();
    sorted_children.sort_by_key(|child| tweets[child.index].timestamp);

    let (children, child_tasks): (Vec<_>, Vec<_>) = sorted_children
        .iter()
        .map(|child| build_node(child, tweets))
        .unzip();

    let all_tasks = Task::batch(std::iter::once(task).chain(child_tasks));

    (
        BuiltNode {
            component,
            children,
        },
        all_tasks,
    )
}

fn render_built_node<'a>(node: &'a BuiltNode, tweets: &'a [Tweet]) -> Column<'a, Message> {
    let index = node.component.index;
    let tweet_view = node
        .component
        .view(tweets)
        .map(move |msg| Message::TweetMessage(index, msg));

    let mut thread_col = column![tweet_view].spacing(8);

    for child in &node.children {
        let indented = row![space().width(32), render_built_node(child, tweets)];
        thread_col = thread_col.push(indented);
    }

    thread_col
}
