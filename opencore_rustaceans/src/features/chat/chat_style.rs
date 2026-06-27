//! Chat-specific control styling.

use iced::widget::text_input;
use iced::widget::text_input::Status as InputStatus;

use crate::shared::design::OpenCoreTheme;
use crate::shared::design::design_controls::with_alpha;
use crate::shared::design::design_radius::control_radius;
use crate::shared::design::design_tokens::ForegroundToken;

pub fn composer_input_style(theme: OpenCoreTheme, status: InputStatus) -> text_input::Style {
    let base = text_input::Style {
        background: iced::Background::Color(iced::Color::TRANSPARENT),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: control_radius(),
        },
        icon: theme.foreground(ForegroundToken::Muted),
        placeholder: theme.foreground(ForegroundToken::Muted),
        value: theme.foreground(ForegroundToken::Primary),
        selection: with_alpha(theme.foreground(ForegroundToken::Secondary), 0.35),
    };

    match status {
        InputStatus::Disabled => text_input::Style {
            value: theme.foreground(ForegroundToken::Muted),
            ..base
        },
        _ => base,
    }
}
