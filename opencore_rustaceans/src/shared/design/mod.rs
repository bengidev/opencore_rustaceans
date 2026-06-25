#![allow(dead_code, unused_imports)]

//! Internal design system tokens for OpenCore.
//!
//! Implements the visual design guidance: monochrome surfaces, graphite
//! controls, and grayscale live accent. Exposes typed color tokens,
//! spacing/radius scales, typography roles, and a theme struct that
//! resolves tokens to concrete values per mode.

pub mod design_iced_theme;
pub mod design_palette;
pub mod design_platform;
pub mod design_radius;
pub mod design_theme;
pub mod design_tokens;

pub use design_iced_theme::iced_theme;
pub use design_platform::{control_corner_radius, window_corner_radius};
pub use design_radius::{control_radius, surface_radius};
pub use design_theme::{OpenCoreTheme, ThemeMode};
pub use design_tokens::{
    AccentToken, ActionToken, BackgroundToken, BorderToken, ForegroundToken, RadiusToken,
    SpacingToken, StatusToken, TypeRole,
};
