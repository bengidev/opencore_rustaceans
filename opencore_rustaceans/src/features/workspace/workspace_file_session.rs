//! Filesystem-backed workspace session store.

use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use super::workspace_session::{SessionError, WorkspaceSession, WorkspaceSessionData};

const SESSION_FILENAME: &str = "workspace-session.json";

#[derive(Debug, Clone)]
pub struct FileWorkspaceSession {
    session_path: PathBuf,
}

impl FileWorkspaceSession {
    pub fn from_project_dirs() -> Result<Self, SessionError> {
        let proj_dirs =
            ProjectDirs::from("com", "opencore", "opencore").ok_or(SessionError::NoProjectDirs)?;
        Ok(Self::new_at(proj_dirs.data_dir()))
    }

    pub fn new_at<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            session_path: dir.as_ref().join(SESSION_FILENAME),
        }
    }
}

impl WorkspaceSession for FileWorkspaceSession {
    fn load(&self) -> Result<WorkspaceSessionData, SessionError> {
        if !self.session_path.exists() {
            return Ok(WorkspaceSessionData::default());
        }
        let raw = fs::read_to_string(&self.session_path)?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    fn save(&self, session: &WorkspaceSessionData) -> Result<(), SessionError> {
        if let Some(parent) = self.session_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(session)?;
        fs::write(&self.session_path, raw)?;
        restrict_session_file_permissions(&self.session_path);
        Ok(())
    }

    fn clear_open_project(&self) -> Result<(), SessionError> {
        let mut session = self.load()?;
        session.open_project = None;
        session.messages.clear();
        session.draft.clear();
        session.activity.clear();
        self.save(&session)
    }
}

#[cfg(unix)]
fn restrict_session_file_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(not(unix))]
fn restrict_session_file_permissions(_path: &Path) {}

#[cfg(test)]
mod tests {
    use crate::features::chat::{ChatMessage, ChatRole};
    use super::*;

    #[test]
    fn starts_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = FileWorkspaceSession::new_at(tmp.path());
        let session = store.load().unwrap();
        assert!(session.open_project.is_none());
        assert!(session.messages.is_empty());
    }

    #[test]
    fn round_trips_session() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = FileWorkspaceSession::new_at(tmp.path());
        let session = WorkspaceSessionData {
            open_project: Some(PathBuf::from("/tmp/demo")),
            draft: String::from("draft"),
            model: String::from("openai/gpt-4o-mini"),
            messages: vec![ChatMessage {
                id: 1,
                role: ChatRole::User,
                content: String::from("hello"),
            }],
            activity: vec![],
        };
        store.save(&session).unwrap();
        assert_eq!(store.load().unwrap(), session);
    }

    #[test]
    fn corrupt_file_returns_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = FileWorkspaceSession::new_at(tmp.path());
        fs::write(store.session_path.clone(), "not json").unwrap();
        assert_eq!(store.load().unwrap(), WorkspaceSessionData::default());
    }

    #[test]
    fn clear_open_project_resets_workspace_fields() {
        let tmp = tempfile::TempDir::new().unwrap();
        let store = FileWorkspaceSession::new_at(tmp.path());
        store
            .save(&WorkspaceSessionData {
                open_project: Some(PathBuf::from("/tmp/demo")),
                draft: String::from("draft"),
                model: String::from("openai/gpt-4o-mini"),
                messages: vec![ChatMessage {
                    id: 1,
                    role: ChatRole::User,
                    content: String::from("hello"),
                }],
                activity: vec![String::from("sent")],
            })
            .unwrap();
        store.clear_open_project().unwrap();
        let session = store.load().unwrap();
        assert!(session.open_project.is_none());
        assert!(session.messages.is_empty());
        assert!(session.draft.is_empty());
    }
}
