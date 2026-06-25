//! Routing outcomes from the workspace reducer.

use super::workspace_ai_provider::ChatRequest;

/// What the parent router should do after dispatching a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceOutcome {
    /// State updated locally; no transition.
    None,
    /// Workspace snapshot changed; host should persist session.
    SessionChanged,
    /// User confirmed close project; host should clear session and return to welcome.
    ProjectClosed,
    /// User sent a message; host should start AI streaming.
    StreamRequested(ChatRequest),
}
