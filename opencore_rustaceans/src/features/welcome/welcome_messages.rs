//! Messages produced by the welcome view and host integrations.

use std::path::PathBuf;

use super::welcome_model::WelcomeItemId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WelcomeMessage {
    ItemHovered(Option<usize>),
    ItemPressed(WelcomeItemId),
    NewFileDialogCompleted(Option<PathBuf>),
    NewFileResult(Result<PathBuf, String>),
    OpenProjectDialogCompleted(Option<PathBuf>),
    CloneUrlChanged(String),
    CloneSubmit,
    CloneCancel,
    CloneCompleted(Result<PathBuf, String>),
    #[allow(dead_code)] // reducer + tests; row press toggles via request_action
    CommandPaletteToggle,
    CommandPaletteQueryChanged(String),
    CommandPaletteSelect(usize),
    CommandPaletteDismiss,
    StatusDismiss,
    ShiftPressed,
}
