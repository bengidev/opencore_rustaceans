//! Shared chat control styling.

use iced::widget::{button, text_input};
use iced::widget::text_input::Status as InputStatus;

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_tokens::{BackgroundToken, BorderToken, ForegroundToken, RadiusToken};

pub fn control_radius() -> iced::border::Radius {
    RadiusToken::Sm.value().into()
}

pub fn text_input_style(theme: OpenCoreTheme, status: InputStatus) -> text_input::Style {
    let base = text_input::Style {
        background: iced::Background::Color(theme.background(BackgroundToken::Primary)),
        border: iced::Border {
            color: theme.border(BorderToken::Default),
            width: 1.0,
            radius: control_radius(),
        },
        icon: theme.foreground(ForegroundToken::Muted),
        placeholder: theme.foreground(ForegroundToken::Muted),
        value: theme.foreground(ForegroundToken::Primary),
        selection: with_alpha(theme.foreground(ForegroundToken::Secondary), 0.35),
    };

    match status {
        InputStatus::Focused { .. } | InputStatus::Hovered => text_input::Style {
            border: iced::Border {
                color: theme.border(BorderToken::Strong),
                ..base.border
            },
            ..base
        },
        InputStatus::Disabled => text_input::Style {
            background: iced::Background::Color(theme.background(BackgroundToken::Tertiary)),
            value: theme.foreground(ForegroundToken::Muted),
            ..base
        },
        InputStatus::Active => base,
    }
}

fn with_alpha(color: iced::Color, alpha: f32) -> iced::Color {
    iced::Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

pub fn chip_button_style(theme: OpenCoreTheme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Tertiary),
        )),
        text_color: theme.foreground(ForegroundToken::Primary),
        border: iced::Border {
            radius: control_radius(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            border: iced::Border {
                color: theme.border(BorderToken::Strong),
                ..base.border
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Secondary),
            )),
            ..base
        },
        _ => base,
    }
}

pub fn icon_button_style(
    theme: OpenCoreTheme,
    background: iced::Color,
    text_color: iced::Color,
    border_width: f32,
    primary: bool,
    status: button::Status,
) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border: iced::Border {
            radius: control_radius(),
            width: border_width,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    };

    if !primary {
        return base;
    }

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(with_alpha(background, 0.88))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(with_alpha(background, 0.72))),
            ..base
        },
        _ => base,
    }
}
