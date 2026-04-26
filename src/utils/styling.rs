//! Styling utilities for twtxt GUI.

use iced::{Background, Border, Color, Theme, border::Radius, widget::button};

pub fn prim_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        button::Status::Hovered => ext.background.weak.color,
        button::Status::Pressed => ext.background.strong.color,
        _ => ext.background.base.color,
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.text,
        border: Border {
            radius: Radius::from(8.0),
            width: 1.0,
            color: iced::Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

pub fn tab_style(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();
        let ext = theme.extended_palette();

        let bg = match status {
            button::Status::Hovered => ext.background.weak.color,
            button::Status::Pressed => ext.background.strong.color,
            _ => ext.background.base.color,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if is_active {
                palette.text
            } else {
                Color {
                    a: 0.5,
                    ..palette.text
                }
            },
            border: Border {
                radius: Radius::from(8.0),
                // Yes, I know this border radius isn't the inner radius plus the padding
                // It should be 10 but that looks ugly, and an inner radius of 2
                // (which is what every widget uses in the container) is too subtle to notice the inconsistency
                width: 1.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

pub fn sec_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        button::Status::Hovered => ext.background.weaker.color,
        button::Status::Pressed => ext.background.stronger.color,
        _ => ext.background.weak.color,
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.text,
        border: Border {
            radius: Radius::from(4.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        ..Default::default()
    }
}
