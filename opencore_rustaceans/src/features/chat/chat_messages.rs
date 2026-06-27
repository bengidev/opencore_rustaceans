//! Messages produced by the chat view.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    DraftChanged(String),
    SendPressed,
    ApiKeyHintPressed,
    ConfigureActionsPressed,
    ModelChipPressed,
    FolderScopePressed,
    StreamDelta(String),
    StreamCompleted,
    StreamFailed(String),
    /// Decorative widgets that never emit; satisfies iced message typing.
    Noop,
}
