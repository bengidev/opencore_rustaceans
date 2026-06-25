//! Provider branding assets for the chat empty state.

use std::sync::OnceLock;

use iced::widget::image::Handle;

const OPENROUTER_LOGO: &[u8] = include_bytes!("../../../assets/openrouter-logo.png");

static OPENROUTER_LOGO_HANDLE: OnceLock<Handle> = OnceLock::new();

/// OpenRouter mark sourced from https://openrouter.ai/apple-touch-icon.png
pub fn openrouter_logo() -> Handle {
    OPENROUTER_LOGO_HANDLE
        .get_or_init(|| Handle::from_bytes(OPENROUTER_LOGO))
        .clone()
}
