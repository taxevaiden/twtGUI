mod app;
mod config;
mod pages;
mod utils;

use app::TwtxtApp;
use iced::{Pixels, Settings, font};

fn main() -> iced::Result {
    iced::application(TwtxtApp::default, TwtxtApp::update, TwtxtApp::view)
        .title("twtGUI")
        .settings(Settings {
            default_text_size: Pixels(12.0),
            ..Default::default()
        })
        .font(include_bytes!("../assets/fonts/iosevka-aile.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/iosevka-aile-bold.ttf").as_slice())
        .default_font(font::Font::with_name("Iosevka Aile"))
        .run()
}
