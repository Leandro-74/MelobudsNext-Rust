//! Palette, typography and widget styles shared by the interface.

use iced::widget::{button, container, pick_list, scrollable, slider, text_input, toggler};
use iced::{Border, Color, Font, Shadow, Theme};

use crate::app::App;

pub(crate) const BG: Color = Color::from_rgb(0.047, 0.051, 0.059);
pub(crate) const PANEL: Color = Color::from_rgb(0.071, 0.078, 0.090);
pub(crate) const CARD: Color = Color::from_rgb(0.090, 0.098, 0.114);
pub(crate) const CARD_ALT: Color = Color::from_rgb(0.125, 0.137, 0.157);
pub(crate) const CARD_DEEP: Color = Color::from_rgb(0.055, 0.061, 0.071);
pub(crate) const TEXT: Color = Color::from_rgb(0.941, 0.949, 0.961);
pub(crate) const MUTED: Color = Color::from_rgb(0.553, 0.584, 0.635);
pub(crate) const BORDER: Color = Color::from_rgb(0.173, 0.188, 0.216);
pub(crate) const ACCENT: Color = Color::from_rgb(0.533, 0.208, 0.914);
pub(crate) const ACCENT_HOVER: Color = Color::from_rgb(0.612, 0.302, 0.945);
pub(crate) const ACCENT_PRESSED: Color = Color::from_rgb(0.455, 0.157, 0.812);
pub(crate) const VIOLET: Color = Color::from_rgb(0.604, 0.549, 1.0);
pub(crate) const AMBER: Color = Color::from_rgb(1.0, 0.784, 0.341);
pub(crate) const DANGER: Color = Color::from_rgb(1.0, 0.478, 0.478);

pub(crate) const LOGO_SVG: &[u8] = include_bytes!("../../../assets/logo.svg");
pub(crate) const EARBUD_LEFT_SVG: &[u8] = include_bytes!("../../../assets/earbud-left.svg");
pub(crate) const EARBUD_RIGHT_SVG: &[u8] = include_bytes!("../../../assets/earbud-right.svg");

pub(crate) fn bold() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    }
}

pub(crate) fn medium() -> Font {
    Font {
        weight: iced::font::Weight::Medium,
        ..Font::DEFAULT
    }
}

pub(crate) fn theme(_app: &App) -> Theme {
    Theme::custom(
        "MelobudsNext".to_string(),
        iced::theme::Palette {
            background: BG,
            text: TEXT,
            primary: ACCENT,
            success: ACCENT,
            danger: DANGER,
        },
    )
}

pub(crate) fn root_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(BG.into()),
        text_color: Some(TEXT),
        ..Default::default()
    }
}

pub(crate) fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(PANEL.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 16.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn illustration_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD_DEEP.into()),
        border: Border {
            color: Color { a: 0.7, ..BORDER },
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn soft_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD.into()),
        border: Border {
            color: Color { a: 0.8, ..BORDER },
            width: 1.0,
            radius: 11.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn track_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD_ALT.into()),
        border: Border {
            radius: 2.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub(crate) fn divider_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color { a: 0.7, ..BORDER }.into()),
        ..Default::default()
    }
}

pub(crate) fn soft_pill_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(CARD.into()),
        border: Border {
            color: Color { a: 0.8, ..BORDER },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub(crate) fn pill_style(color: Color) -> container::Style {
    container::Style {
        background: Some(Color { a: 0.10, ..color }.into()),
        border: Border {
            color: Color { a: 0.35, ..color },
            width: 1.0,
            radius: 20.0.into(),
        },
        text_color: Some(color),
        ..Default::default()
    }
}

pub(crate) fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused => ACCENT,
        text_input::Status::Hovered => Color { a: 0.8, ..BORDER },
        _ => BORDER,
    };
    text_input::Style {
        background: CARD_DEEP.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 9.0.into(),
        },
        icon: MUTED,
        placeholder: MUTED,
        value: TEXT,
        selection: Color { a: 0.35, ..ACCENT },
    }
}

pub(crate) fn pick_list_style(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: TEXT,
        placeholder_color: MUTED,
        handle_color: if matches!(status, pick_list::Status::Opened) {
            ACCENT
        } else {
            MUTED
        },
        background: CARD_DEEP.into(),
        border: Border {
            color: if matches!(
                status,
                pick_list::Status::Hovered | pick_list::Status::Opened
            ) {
                ACCENT
            } else {
                BORDER
            },
            width: 1.0,
            radius: 9.0.into(),
        },
    }
}

pub(crate) fn menu_style(_theme: &Theme) -> iced::widget::overlay::menu::Style {
    iced::widget::overlay::menu::Style {
        background: CARD.into(),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: TEXT,
        selected_text_color: BG,
        selected_background: ACCENT.into(),
    }
}

pub(crate) fn slider_style(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (ACCENT.into(), CARD_ALT.into()),
            width: 5.0,
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: ACCENT.into(),
            border_width: 3.0,
            border_color: BG,
        },
    }
}

pub(crate) fn toggler_style(
    toggled: bool,
    _theme: &Theme,
    status: toggler::Status,
) -> toggler::Style {
    let hovered = matches!(
        status,
        toggler::Status::Hovered { .. } | toggler::Status::Active { .. }
    );
    toggler::Style {
        background: if toggled {
            if hovered {
                ACCENT_HOVER
            } else {
                ACCENT
            }
        } else {
            CARD_ALT
        },
        background_border_width: 1.0,
        background_border_color: if toggled { ACCENT } else { BORDER },
        foreground: if toggled { BG } else { MUTED },
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
    }
}

pub(crate) fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => ACCENT_HOVER,
        button::Status::Pressed => ACCENT_PRESSED,
        button::Status::Disabled => CARD_ALT,
        _ => ACCENT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: if matches!(status, button::Status::Disabled) {
            MUTED
        } else {
            BG
        },
        border: Border {
            radius: 9.0.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
    }
}

pub(crate) fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color {
            a: 0.55,
            ..CARD_ALT
        },
        button::Status::Pressed => CARD_ALT,
        button::Status::Disabled => Color { a: 0.4, ..CARD_ALT },
        _ => CARD_ALT,
    };
    button::Style {
        background: Some(background.into()),
        text_color: if matches!(status, button::Status::Disabled) {
            MUTED
        } else {
            TEXT
        },
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub(crate) fn danger_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let color = if matches!(status, button::Status::Disabled) {
        Color { a: 0.4, ..DANGER }
    } else if matches!(status, button::Status::Hovered) {
        DANGER
    } else {
        Color { a: 0.85, ..DANGER }
    };
    button::Style {
        background: Some(Color { a: 0.08, ..DANGER }.into()),
        text_color: color,
        border: Border {
            color: Color { a: 0.35, ..DANGER },
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub(crate) fn tab_style(active: bool, _theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color) = if active {
        (ACCENT, BG)
    } else {
        match status {
            button::Status::Hovered => (CARD_ALT, TEXT),
            button::Status::Pressed => (CARD, TEXT),
            _ => (Color::TRANSPARENT, MUTED),
        }
    };

    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            color: if active { ACCENT } else { Color::TRANSPARENT },
            width: 1.0,
            radius: 9.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub(crate) fn chip_style(selected: bool, _theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color, border_color) = if selected {
        (
            Color { a: 0.16, ..ACCENT },
            ACCENT,
            Color { a: 0.6, ..ACCENT },
        )
    } else {
        match status {
            button::Status::Hovered => (CARD_ALT, TEXT, BORDER),
            _ => (CARD, MUTED, BORDER),
        }
    };

    button::Style {
        background: Some(background.into()),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub(crate) fn scrollable_style(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BORDER,
                border: Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: BORDER,
                border: Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
            },
        },
        gap: None,
    }
}
