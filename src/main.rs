//! Entry point for twtGUI.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod config;
mod logging;
mod pages;
mod twtxt;
mod utils;

use app::TwtxtApp;
use iced::{Pixels, Settings, font};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::utils::paths::log_root;

const ICON_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.ico"));

fn main() -> iced::Result {
    // Set up logging
    let filter = EnvFilter::new("warn,twtgui=debug");

    let log_buffer: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

    let file_appender = tracing_appender::rolling::daily(
        log_root().expect("Failed to determine log directory"),
        "twtgui.log",
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(logging::UiLayer::new(log_buffer.clone()))
        .init();

    let icon = iced::window::icon::from_file_data(ICON_BYTES, None).unwrap();

    iced::application(
        move || TwtxtApp::new(log_buffer.clone()),
        TwtxtApp::update,
        TwtxtApp::view,
    )
    .subscription(TwtxtApp::subscription)
    .title("twtGUI")
    .window(iced::window::Settings {
        min_size: Some(iced::Size::new(800.0, 700.0)),
        icon: Some(icon),
        #[allow(clippy::needless_update)]
        platform_specific: iced::window::settings::PlatformSpecific {
            #[cfg(target_os = "macos")]
            title_hidden: true,
            #[cfg(target_os = "macos")]
            titlebar_transparent: true,
            #[cfg(target_os = "macos")]
            fullsize_content_view: true,
            ..Default::default()
        },
        ..Default::default()
    })
    .settings(Settings {
        default_text_size: Pixels(12.0),
        ..Default::default()
    })
    .theme(|app: &TwtxtApp| app.theme())
    .font(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/iosevka-aile.ttf"
        ))
        .as_slice(),
    )
    .font(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/iosevka-aile-bold.ttf"
        ))
        .as_slice(),
    )
    .font(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/iosevka-aile-italic.ttf"
        ))
        .as_slice(),
    )
    .font(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/iosevka-aile-italic-bold.ttf"
        ))
        .as_slice(),
    )
    .font(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/iosevka-mono.ttf"
        ))
        .as_slice(),
    )
    .default_font(font::Font::with_name("Iosevka Aile"))
    .run()
}
