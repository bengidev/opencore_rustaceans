//! Routing outcomes from the welcome reducer.

use crate::shared::design::ThemeMode;

use super::welcome_model::WelcomeItemId;

/// What the parent router should do after dispatching a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeOutcome {
    /// State updated locally; no transition.
    None,
    /// User toggled the theme.
    ThemeToggled(ThemeMode),
    /// User activated a row; host will wire behavior later.
    ActionRequested(WelcomeItemId),
}
