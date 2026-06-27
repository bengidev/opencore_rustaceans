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
    ModelChipPressed,
    SandboxScopePressed,
    FolderScopePressed,
    ModelPickerDismiss,
    ModelPickerQueryChanged(String),
    ModelPickerSelect(usize),
    ModelsLoadStarted,
    ModelsLoaded(Vec<super::workspace_openrouter_catalog::ModelOption>),
    ModelsLoadFailed(String),
    /// Decorative widgets that never emit; satisfies iced message typing.
    Noop,
}
