//! Messages produced by the welcome view and host integrations.

use std::path::PathBuf;

use super::welcome_model::WelcomeItemId;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // ToggleTheme: reducer + tests; other variants wired from UI/host
pub enum WelcomeMessage {
    ToggleTheme,
    ItemHovered(Option<usize>),
    ItemPressed(WelcomeItemId),
    NewFileDialogCompleted(Option<PathBuf>),
    OpenProjectDialogCompleted(Option<PathBuf>),
    CloneUrlChanged(String),
    CloneSubmit,
    CloneCancel,
    CloneCompleted(Result<PathBuf, String>),
    CommandPaletteToggle,
    CommandPaletteQueryChanged(String),
    CommandPaletteSelect(usize),
    CommandPaletteDismiss,
    StatusDismiss,
    ShiftPressed,
    ActionCompleted { path: PathBuf, summary: String },
}
