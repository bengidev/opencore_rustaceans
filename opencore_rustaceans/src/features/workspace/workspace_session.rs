//! Workspace session persistence strategy.

use std::path::PathBuf;

use super::workspace_model::ChatMessage;

/// Serializable workspace session snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceSessionData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_project: Option<PathBuf>,
    #[serde(default)]
    pub draft: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub activity: Vec<String>,
}

fn default_model() -> String {
    super::workspace_ai_provider::DEFAULT_MODEL.into()
}

/// Errors from session persistence backends.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("could not resolve project directories")]
    NoProjectDirs,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Strategy for loading and saving workspace session snapshots.
pub trait WorkspaceSession: Send + Sync {
    fn load(&self) -> Result<WorkspaceSessionData, SessionError>;
    fn save(&self, session: &WorkspaceSessionData) -> Result<(), SessionError>;
    fn clear_open_project(&self) -> Result<(), SessionError>;
}
