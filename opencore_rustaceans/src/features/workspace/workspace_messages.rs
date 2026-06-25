//! Messages produced by the workspace view and host integrations.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceMessage {
    DraftChanged(String),
    SendPressed,
    ApiKeyHintPressed,
    ApiKeyInputChanged(String),
    ApiKeySave,
    ApiKeyRemove,
    ApiKeyDismiss,
    ConfigureActionsPressed,
    CloseProjectRequested,
    CloseProjectCancel,
    CloseProjectConfirm,
    StreamDelta(String),
    StreamCompleted,
    StreamFailed(String),
    ApiKeyPresenceChanged(bool),
}
