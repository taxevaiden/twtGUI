//! Tracing layer that buffers log messages for UI display.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

pub type LogBuffer = Arc<Mutex<VecDeque<String>>>;

pub struct UiLayer {
    buffer: LogBuffer,
}

impl UiLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

struct MsgVisitor(String);

impl Visit for MsgVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }
}

impl<S: Subscriber> Layer<S> for UiLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MsgVisitor(String::new());
        event.record(&mut visitor);
        let msg = format!("[{}] {}", event.metadata().level(), visitor.0);
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push_back(msg);
            if buf.len() > 1000 {
                buf.pop_front();
            }
        }
    }
}
