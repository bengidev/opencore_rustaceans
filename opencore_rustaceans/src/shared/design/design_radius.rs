//! Platform corner radii as iced border values.

use super::design_tokens::RadiusToken;

/// Buttons, chips, rows, and compact controls.
pub fn control_radius() -> iced::border::Radius {
    RadiusToken::Control.value().into()
}

/// Panels, cards, and other large surfaces.
pub fn surface_radius() -> iced::border::Radius {
    RadiusToken::Window.value().into()
}
