//! Routing outcomes from the welcome reducer.

use std::path::PathBuf;

use super::welcome_model::WelcomeItemId;

/// What the parent router should do after dispatching a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WelcomeOutcome {
    /// State updated locally; no transition.
    None,
    /// User activated a row; host should start the matching workflow.
    ActionRequested(WelcomeItemId),
    /// User submitted clone; host should run `git clone` for this URL.
    CloneRequested(String),
    /// A workspace entry point was resolved to a concrete path.
    WorkspaceOpened(PathBuf),
}
