//! Routing outcomes from the chat reducer.

use super::chat_model::ChatMessage;

/// What the workspace reducer should do after a chat event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatOutcome {
    /// State updated locally; no transition.
    None,
    /// Chat snapshot changed; host should persist session.
    SessionChanged,
    /// User sent a message; host should start AI streaming.
    SendRequested(Vec<ChatMessage>),
    /// Send blocked until an API key is configured.
    ApiKeyRequired,
}
