//! Styling utilities for twtxt GUI.

use iced::{
    Background, Border, Color, Shadow, Theme,
    border::Radius,
    overlay::menu,
    widget::{button, pick_list},
};

/// The primary button style for the application.
/// Used in places where the background is the "base" color from the theme.
/// (More like secondary since its transparent but whatever)
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

/// The style for tab buttons in the application.
/// Used in the sidebar, where the background is the "base" color from the theme.
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

/// The secondary button style for the application.
/// Used in places where the background is the "weak" color from the theme.
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
            radius: Radius::from(8.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

/// The primary pick list style used in the application.
/// Used in places where the background is the "base" color from the theme.
pub fn prim_pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        pick_list::Status::Hovered => ext.background.weak.color,
        pick_list::Status::Opened { is_hovered } => match is_hovered {
            true => ext.background.weaker.color,
            false => ext.background.weak.color,
        },
        _ => ext.background.base.color,
    };

    pick_list::Style {
        text_color: palette.text,
        background: Background::from(bg),
        handle_color: palette.primary,
        placeholder_color: secondary_text(theme),
        border: Border {
            radius: Radius::from(8.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
    }
}

/// The primary pick list menu style used in the application.
/// Used in places where the background is the "base" color from the theme.
pub fn prim_pick_menu_style(theme: &Theme) -> menu::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    menu::Style {
        background: Background::from(ext.background.weak.color),
        text_color: palette.text,
        selected_background: Background::from(ext.background.weaker.color),
        selected_text_color: palette.text,
        border: Border {
            radius: Radius::from(8.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
    }
}

/// The secondary pick list style used in the application.
/// Used in places where the background is the "weak" color from the theme.
pub fn sec_pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        pick_list::Status::Hovered => ext.background.weaker.color,
        pick_list::Status::Opened { is_hovered } => match is_hovered {
            true => ext.background.weakest.color,
            false => ext.background.weaker.color,
        },
        _ => ext.background.weak.color,
    };

    pick_list::Style {
        text_color: palette.text,
        background: Background::from(bg),
        handle_color: palette.primary,
        placeholder_color: secondary_text(theme),
        border: Border {
            radius: Radius::from(8.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
    }
}

/// The secondary pick list menu style used in the application.
/// Used in places where the background is the "weak" color from the theme.
pub fn sec_pick_menu_style(theme: &Theme) -> menu::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    menu::Style {
        background: Background::from(ext.background.weaker.color),
        text_color: palette.text,
        selected_background: Background::from(ext.background.weakest.color),
        selected_text_color: palette.text,
        border: Border {
            radius: Radius::from(8.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
    }
}

/// Returns the secondary text color for the given theme.
pub fn secondary_text(theme: &Theme) -> Color {
    Color {
        a: 0.55,
        ..theme.palette().text
    }
}
