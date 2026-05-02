//! Styling utilities for twtxt GUI.

use iced::{
    Background, Border, Color, Shadow, Theme,
    border::Radius,
    overlay::menu,
    widget::{button, pick_list, text_editor, text_input},
};

/// The primary button style for the application.
/// Used in places where the background is the "base" color from the theme.
/// (More like secondary since its transparent but whatever)
pub fn prim_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        button::Status::Hovered => ext.background.strong.color,
        button::Status::Pressed => ext.background.weak.color,
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

/// The secondary button style for the application.
/// Used in places where the background is the "weak" color from the theme.
pub fn sec_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        button::Status::Hovered => ext.background.stronger.color,
        button::Status::Pressed => ext.background.weaker.color,
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
            true => ext.background.strong.color,
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
        selected_background: Background::from(ext.background.strong.color),
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
        pick_list::Status::Hovered => ext.background.stronger.color,
        pick_list::Status::Opened { is_hovered } => match is_hovered {
            true => ext.background.stronger.color,
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
        selected_background: Background::from(ext.background.stronger.color),
        selected_text_color: palette.text,
        border: Border {
            radius: Radius::from(8.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
    }
}

/// The style for toolbar buttons.
pub fn toolbar_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        button::Status::Hovered => ext.primary.strong.color,
        button::Status::Pressed => ext.primary.weak.color,
        _ => ext.primary.base.color,
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background,
        border: Border {
            radius: Radius::from(4.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
        ..Default::default()
    }
}

/// The style for toolbar single-line input fields.
pub fn toolbar_sinput_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        text_input::Status::Disabled => ext.background.weak.color,
        _ => ext.background.base.color,
    };

    let border = match status {
        text_input::Status::Focused { is_hovered } => match is_hovered {
            true => Border {
                radius: Radius::from(4.0),
                width: 1.0,
                color: ext.background.strongest.color,
            },
            false => Border {
                radius: Radius::from(4.0),
                width: 1.0,
                color: ext.background.strong.color,
            },
        },
        _ => Border {
            radius: Radius::from(4.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
    };

    text_input::Style {
        background: Background::Color(bg),
        border,
        placeholder: ext.background.strongest.color,
        value: palette.text,
        icon: iced::Color::TRANSPARENT,
        selection: palette.primary,
    }
}

/// The style for toolbar multi-line input fields.
pub fn toolbar_minput_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
    let palette = theme.palette();
    let ext = theme.extended_palette();

    let bg = match status {
        text_editor::Status::Disabled => ext.background.weak.color,
        _ => ext.background.base.color,
    };

    let border = match status {
        text_editor::Status::Focused { is_hovered } => match is_hovered {
            true => Border {
                radius: Radius::from(4.0),
                width: 1.0,
                color: ext.background.strongest.color,
            },
            false => Border {
                radius: Radius::from(4.0),
                width: 1.0,
                color: ext.background.strong.color,
            },
        },
        _ => Border {
            radius: Radius::from(4.0),
            width: 0.0,
            color: iced::Color::TRANSPARENT,
        },
    };

    text_editor::Style {
        background: Background::Color(bg),
        border,
        placeholder: ext.background.strongest.color,
        value: palette.text,
        selection: palette.primary,
    }
}

/// The style for tab buttons in the application.
/// Used in the sidebar, where the background is the "base" color from the theme.
pub fn tab_style(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.palette();
        let ext = theme.extended_palette();

        let bg = match status {
            button::Status::Hovered => ext.background.strong.color,
            button::Status::Pressed => ext.background.weak.color,
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
                width: 1.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

/// Returns the secondary text color for the given theme.
pub fn secondary_text(theme: &Theme) -> Color {
    Color {
        a: 0.55,
        ..theme.palette().text
    }
}
