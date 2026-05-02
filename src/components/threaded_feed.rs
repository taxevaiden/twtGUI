//! A lazily-rendered threaded feed component.
//!
//! Renders a subset of threads and loads more as the user scrolls.

use crate::components::tweet::{self, TweetComponent};
use crate::utils::styling::sec_button_style;
use crate::utils::{Tweet, TweetNode};
use iced::widget::container;
use iced::widget::rule::horizontal;
use iced::{
    Element, Length, Task, Theme,
    widget::{Column, Id, button, column, image::Handle, row, scrollable, space},
};
use std::collections::HashMap;
use tracing::{debug, error};

/// How many additional threads to load when reaching the bottom of the scroll.
const BATCH_SIZE: usize = 10; // Threads can be large, so smaller batches are safer
/// How many threads to load initially.
const INITIAL_LOAD: usize = 25;
/// How close to the bottom of the scroll before loading more threads.
const LOAD_THRESHOLD: f32 = 400.0;
/// How close to the top of the scroll to reset to the initial load size.
const TOP_THRESHOLD: f32 = 50.0;

/// A memoized thread node with its rendered component and child threads.
struct BuiltNode {
    component: TweetComponent,
    children: Vec<BuiltNode>,
}

/// A snapshot pushed onto the navigation stack when drilling into a thread.
///
/// Both the source `TweetNode` tree and the already-built `BuiltNode` tree are
/// stored so that `ThreadBack` can restore either without any rebuild.
struct StackEntry {
    source_threads: Vec<TweetNode>,
    built_threads: Vec<BuiltNode>,
    /// The flat lookup map that was active at this stack level.
    node_index: HashMap<usize, Vec<usize>>,
    visible_threads_count: usize,
}

/// Messages emitted by the threaded feed component.
#[derive(Debug, Clone)]
pub enum Message {
    /// The scroll position changed.
    Scrolled(scrollable::Viewport),
    /// A link inside a tweet was clicked.
    LinkClicked(String),
    /// A reply button inside a tweet was clicked.
    ReplyClicked(usize),
    /// Request to navigate to another page.
    RedirectToPage(crate::app::RedirectInfo),
    /// A message coming from a specific tweet component.
    Tweet(usize, tweet::Message),
    /// Navigate back one level in the thread stack.
    ThreadBack,
}

/// Lazy-loading threaded feed view.
pub struct LazyThreadedFeed {
    scroll_id: Id,
    visible_threads_count: usize,
    source_threads: Vec<TweetNode>,
    built_threads: Vec<BuiltNode>,
    /// Flat map from tweet index -> path of child-positions through `built_threads`.
    ///
    /// A path like `[2, 0, 1]` means:
    ///   `built_threads[2].children[0].children[1]`
    ///
    /// This is rebuilt whenever `built_threads` is replaced (new feed, drill,
    /// or back-navigation) and keeps `find_node_mut` from walking the tree on
    /// every message.
    node_index: HashMap<usize, Vec<usize>>,
    thread_stack: Vec<StackEntry>,
    pub avatars: HashMap<String, Handle>,
}

impl LazyThreadedFeed {
    pub fn new(threads: &[TweetNode], tweets: &[Tweet]) -> (Self, Task<Message>) {
        let (built, task) = build_nodes(threads, tweets);
        let node_index = build_index(&built);
        let total = built.len();
        (
            Self {
                scroll_id: Id::unique(),
                visible_threads_count: INITIAL_LOAD.min(total),
                source_threads: threads.to_vec(),
                built_threads: built,
                node_index,
                thread_stack: Vec::new(),
                avatars: HashMap::new(),
            },
            task,
        )
    }

    pub fn reset(&mut self, threads: &[TweetNode], tweets: &[Tweet]) -> Task<Message> {
        self.source_threads = threads.to_vec();
        self.built_threads = Vec::new();
        self.visible_threads_count = INITIAL_LOAD.min(threads.len());
        self.thread_stack.clear();

        let (built, task) = build_nodes(&self.source_threads[..self.visible_threads_count], tweets);
        self.node_index = build_index(&built);
        self.built_threads = built;
        task
    }

    pub fn update(&mut self, message: Message, tweets: &[Tweet]) -> Task<Message> {
        match message {
            Message::Scrolled(viewport) => {
                let offset = viewport.absolute_offset().y;
                let visible_height = viewport.bounds().height;
                let total_height = viewport.content_bounds().height;
                let total = self.source_threads.len();

                if offset <= TOP_THRESHOLD {
                    let new_count = INITIAL_LOAD.min(total);
                    if self.built_threads.len() > new_count {
                        self.built_threads.truncate(new_count);
                        self.node_index = build_index(&self.built_threads);
                    }
                    self.visible_threads_count = new_count;
                }

                let near_bottom = offset + visible_height >= total_height - LOAD_THRESHOLD;
                if near_bottom && self.visible_threads_count < total {
                    let old_count = self.built_threads.len();
                    self.visible_threads_count =
                        (self.visible_threads_count + BATCH_SIZE).min(total);

                    // Build only the newly revealed threads
                    if self.visible_threads_count > old_count {
                        let (new_nodes, task) = build_nodes(
                            &self.source_threads[old_count..self.visible_threads_count],
                            tweets,
                        );
                        self.built_threads.extend(new_nodes);
                        self.node_index = build_index(&self.built_threads);
                        return task;
                    }
                }

                Task::none()
            }

            Message::ThreadBack => {
                if let Some(entry) = self.thread_stack.pop() {
                    self.source_threads = entry.source_threads;
                    self.built_threads = entry.built_threads;
                    self.node_index = entry.node_index;
                    self.visible_threads_count = entry.visible_threads_count;
                    Task::none()
                } else {
                    Task::none()
                }
            }

            Message::RedirectToPage(info) => Task::done(Message::RedirectToPage(info)),

            Message::ReplyClicked(index) => Task::done(Message::ReplyClicked(index)),

            Message::LinkClicked(url) => {
                if url.contains("twtxt") && url.ends_with(".txt") {
                    Task::done(Message::RedirectToPage(crate::app::RedirectInfo {
                        page: crate::app::Page::View,
                        content: url,
                    }))
                } else {
                    debug!("ThreadedFeed: opening URL: {}", url);
                    if let Err(err) = webbrowser::open(&url) {
                        error!("ThreadedFeed: error opening URL: {}", err);
                    }
                    Task::none()
                }
            }

            Message::Tweet(_, tweet::Message::ReplyClicked(index)) => {
                Task::done(Message::ReplyClicked(index))
            }

            Message::Tweet(_, tweet::Message::LinkClicked(url)) => {
                Task::done(Message::LinkClicked(url))
            }

            Message::Tweet(_, tweet::Message::ThreadClicked(index)) => {
                self.drill_into_thread(index, tweets)
            }

            Message::Tweet(index, msg) => {
                if let Some(node) = find_node_mut(&mut self.built_threads, &self.node_index, index)
                {
                    node.component
                        .update(msg)
                        .map(move |m| Message::Tweet(index, m))
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Push the current trees onto the stack, then rebuild from only the
    /// subtree rooted at index.
    fn drill_into_thread(&mut self, index: usize, tweets: &[Tweet]) -> Task<Message> {
        if let Some(focused_source) = find_source_node(&self.source_threads, index) {
            // Take ownership of the current trees before replacing them.
            let prev_source = std::mem::take(&mut self.source_threads);
            let prev_built = std::mem::take(&mut self.built_threads);
            let prev_index = std::mem::take(&mut self.node_index);

            self.thread_stack.push(StackEntry {
                source_threads: prev_source,
                built_threads: prev_built,
                node_index: prev_index,
                visible_threads_count: self.visible_threads_count,
            });

            let focused_sources = vec![focused_source];
            let (built, task) = build_nodes(&focused_sources, tweets);
            self.node_index = build_index(&built);
            self.source_threads = focused_sources;
            self.built_threads = built;
            self.visible_threads_count = INITIAL_LOAD.min(self.built_threads.len());
            task
        } else {
            Task::none()
        }
    }

    pub fn view<'a>(
        &'a self,
        theme: &Theme,
        tweets: &'a [Tweet],
        reply_available: bool,
    ) -> Element<'a, Message> {
        let visible = &self.built_threads;

        let mut col = column!().spacing(8);

        if !self.thread_stack.is_empty() {
            col = col.push(
                button("Back")
                    .on_press(Message::ThreadBack)
                    .padding([8.0, 16.0])
                    .style(sec_button_style),
            );
        }

        for node in visible {
            col = col.push(
                column![
                    render_built_node(theme, node, tweets, &self.avatars, reply_available),
                    horizontal(1),
                ]
                .width(Length::Fill)
                .spacing(8),
            );
        }

        container(
            scrollable(col)
                .id(self.scroll_id.clone())
                .spacing(8)
                .on_scroll(Message::Scrolled)
                .height(Length::Fill),
        )
        .padding(8)
        .height(Length::Fill)
        .into()
    }
}

/// Build a flat  map over the entire `BuiltNode` tree.
///
/// A path is the sequence of child-list positions to follow from the root of
/// built_threads to reach the node. Traversal is O(depth).
/// O(1) for real thread trees.
fn build_index(roots: &[BuiltNode]) -> HashMap<usize, Vec<usize>> {
    let mut map = HashMap::new();
    for (i, root) in roots.iter().enumerate() {
        index_node(root, vec![i], &mut map);
    }
    map
}

fn index_node(node: &BuiltNode, path: Vec<usize>, map: &mut HashMap<usize, Vec<usize>>) {
    map.insert(node.component.index, path.clone());
    for (i, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(i);
        index_node(child, child_path, map);
    }
}

/// Look up a node by tweet index using the cached path map.
///
/// Follows the stored path through roots in O(depth) steps.
fn find_node_mut<'a>(
    roots: &'a mut [BuiltNode],
    index_map: &HashMap<usize, Vec<usize>>,
    tweet_index: usize,
) -> Option<&'a mut BuiltNode> {
    let path = index_map.get(&tweet_index)?;

    let mut iter = path.iter();
    let first = *iter.next()?;
    let mut node = roots.get_mut(first)?;

    for &child_pos in iter {
        node = node.children.get_mut(child_pos)?;
    }

    Some(node)
}

/// Recursively find and clone a TweetNode subtree by tweet index.
fn find_source_node(nodes: &[TweetNode], index: usize) -> Option<TweetNode> {
    for node in nodes {
        if node.index == index {
            return Some(node.clone());
        }
        if let Some(found) = find_source_node(&node.children, index) {
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
    let task = task.map(move |msg| Message::Tweet(index, msg));

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

fn render_built_node<'a>(
    theme: &Theme,
    node: &'a BuiltNode,
    tweets: &'a [Tweet],
    avatars: &'a HashMap<String, Handle>,
    reply_available: bool,
) -> Column<'a, Message> {
    let index = node.component.index;
    let tweet_view = node
        .component
        .view(theme, tweets, avatars, reply_available)
        .map(move |msg| Message::Tweet(index, msg));

    let mut thread_col = column![tweet_view].spacing(8);

    for child in &node.children {
        let indented = row![
            space().width(32),
            render_built_node(theme, child, tweets, avatars, reply_available)
        ];
        thread_col = thread_col.push(indented);
    }

    thread_col
}
