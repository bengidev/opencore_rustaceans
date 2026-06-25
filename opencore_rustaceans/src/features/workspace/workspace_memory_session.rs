//! In-memory workspace session store for tests.

use std::sync::Mutex;

use super::workspace_session::{SessionError, WorkspaceSession, WorkspaceSessionData};

#[derive(Debug, Default)]
pub struct InMemoryWorkspaceSession {
    data: Mutex<WorkspaceSessionData>,
}

impl InMemoryWorkspaceSession {
    pub fn new() -> Self {
        Self::default()
    }
}

impl WorkspaceSession for InMemoryWorkspaceSession {
    fn load(&self) -> Result<WorkspaceSessionData, SessionError> {
        Ok(self.data.lock().unwrap().clone())
    }

    fn save(&self, session: &WorkspaceSessionData) -> Result<(), SessionError> {
        *self.data.lock().unwrap() = session.clone();
        Ok(())
    }

    fn clear_open_project(&self) -> Result<(), SessionError> {
        let mut data = self.data.lock().unwrap();
        data.open_project = None;
        data.messages.clear();
        data.draft.clear();
        data.activity.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn round_trips_in_memory() {
        let store = InMemoryWorkspaceSession::new();
        let session = WorkspaceSessionData {
            open_project: Some(PathBuf::from("/tmp/demo")),
            draft: String::from("draft"),
            model: String::from("openai/gpt-4o-mini"),
            messages: vec![],
            activity: vec![],
        };
        store.save(&session).unwrap();
        assert_eq!(store.load().unwrap(), session);
    }
}
