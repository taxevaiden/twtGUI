mod app;
mod components;
mod config;
mod pages;
mod utils;

use app::TwtxtApp;
use iced::{Pixels, Settings, font};

const ICON_BYTES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.ico"));

fn main() -> iced::Result {
    let icon = iced::window::icon::from_file_data(ICON_BYTES, None).unwrap();

    iced::application(TwtxtApp::default, TwtxtApp::update, TwtxtApp::view)
        .title("twtGUI")
        .window(iced::window::Settings {
            icon: Some(icon),
            ..Default::default()
        })
        .settings(Settings {
            default_text_size: Pixels(12.0),
            ..Default::default()
        })
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
        .default_font(font::Font::with_name("Iosevka Aile"))
        .theme(iced::Theme::CatppuccinMocha)
        .run()
}
