//! Routing outcomes from the welcome reducer.

use std::path::PathBuf;

use crate::shared::design::ThemeMode;

use super::welcome_model::WelcomeItemId;

/// What the parent router should do after dispatching a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WelcomeOutcome {
    /// State updated locally; no transition.
    None,
    /// User toggled the theme.
    ThemeToggled(ThemeMode),
    /// User activated a row; host should start the matching workflow.
    ActionRequested(WelcomeItemId),
    /// A workspace entry point was resolved to a concrete path.
    WorkspaceOpened(PathBuf),
}
