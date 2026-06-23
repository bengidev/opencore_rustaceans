//! Messages produced by the welcome view.

use super::welcome_model::WelcomeItemId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeMessage {
    ToggleTheme,
    ItemHovered(Option<usize>),
    ItemPressed(WelcomeItemId),
}
