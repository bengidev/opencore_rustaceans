//! Maps [`OpenCoreTheme`] to an iced [`Theme`] with a fully monochrome palette.

use iced::Theme;
use iced::theme::Palette;

use super::design_theme::{OpenCoreTheme, ThemeMode};
use super::design_tokens::{BackgroundToken, ForegroundToken};

/// Build an iced theme without chromatic primary accents (no default purple).
pub fn iced_theme(mode: ThemeMode) -> Theme {
    let core = OpenCoreTheme::from_mode(mode);
    let palette = Palette {
        background: core.background(BackgroundToken::Primary),
        text: core.foreground(ForegroundToken::Primary),
        primary: core.foreground(ForegroundToken::Secondary),
        success: core.foreground(ForegroundToken::Muted),
        warning: core.foreground(ForegroundToken::Secondary),
        danger: core.foreground(ForegroundToken::Secondary),
    };
    Theme::custom("OpenCore Monochrome", palette)
}
