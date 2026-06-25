//! Workspace screen state reducer.

use std::path::PathBuf;

use crate::shared::design::{OpenCoreTheme, ThemeMode};

use super::workspace_ai_provider::{ChatRequest, DEFAULT_MODEL};
use super::workspace_messages::WorkspaceMessage;
use super::workspace_model::ChatThread;
use super::workspace_outcome::WorkspaceOutcome;
use super::workspace_overlay::WorkspaceOverlay;

pub struct WorkspaceState {
    pub project_path: PathBuf,
    pub thread: ChatThread,
    pub draft: String,
    pub model: String,
    pub overlay: WorkspaceOverlay,
    pub api_key_input: String,
    pub has_api_key: bool,
    pub is_streaming: bool,
    pub streaming_message_id: Option<u64>,
    #[allow(dead_code)]
    pub theme_mode: ThemeMode,
    pub theme: OpenCoreTheme,
}

impl std::fmt::Debug for WorkspaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceState")
            .field("project_path", &self.project_path)
            .field("draft", &self.draft)
            .field("model", &self.model)
            .field("overlay", &self.overlay)
            .field("has_api_key", &self.has_api_key)
            .field("is_streaming", &self.is_streaming)
            .finish()
    }
}

impl WorkspaceState {
    pub fn new(project_path: PathBuf, theme_mode: ThemeMode) -> Self {
        Self {
            project_path,
            thread: ChatThread::new(),
            draft: String::new(),
            model: DEFAULT_MODEL.into(),
            overlay: WorkspaceOverlay::None,
            api_key_input: String::new(),
            has_api_key: false,
            is_streaming: false,
            streaming_message_id: None,
            theme_mode,
            theme: OpenCoreTheme::from_mode(theme_mode),
        }
    }

    pub fn restore(
        project_path: PathBuf,
        theme_mode: ThemeMode,
        draft: String,
        model: String,
        thread: ChatThread,
    ) -> Self {
        Self {
            project_path,
            thread,
            draft,
            model,
            overlay: WorkspaceOverlay::None,
            api_key_input: String::new(),
            has_api_key: false,
            is_streaming: false,
            streaming_message_id: None,
            theme_mode,
            theme: OpenCoreTheme::from_mode(theme_mode),
        }
    }

    pub fn update(&mut self, message: WorkspaceMessage) -> WorkspaceOutcome {
        match message {
            WorkspaceMessage::DraftChanged(draft) => {
                self.draft = draft;
                WorkspaceOutcome::SessionChanged
            }
            WorkspaceMessage::SendPressed => self.send_message(),
            WorkspaceMessage::ApiKeyHintPressed | WorkspaceMessage::ConfigureActionsPressed => {
                self.overlay = WorkspaceOverlay::ApiKeySettings;
                self.api_key_input.clear();
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeyInputChanged(value) => {
                self.api_key_input = value;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeySave => {
                if self.api_key_input.trim().is_empty() {
                    return WorkspaceOutcome::None;
                }
                self.overlay = WorkspaceOverlay::None;
                self.has_api_key = true;
                self.api_key_input.clear();
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeyRemove => {
                self.overlay = WorkspaceOverlay::None;
                self.has_api_key = false;
                self.api_key_input.clear();
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeyDismiss => {
                self.overlay = WorkspaceOverlay::None;
                self.api_key_input.clear();
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeyPresenceChanged(present) => {
                self.has_api_key = present;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::CloseProjectRequested => {
                self.overlay = WorkspaceOverlay::CloseProjectConfirm;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::CloseProjectCancel => {
                self.overlay = WorkspaceOverlay::None;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::CloseProjectConfirm => {
                self.overlay = WorkspaceOverlay::None;
                WorkspaceOutcome::ProjectClosed
            }
            WorkspaceMessage::StreamDelta(delta) => {
                if let Some(id) = self.streaming_message_id {
                    self.thread.append_delta(id, &delta);
                }
                WorkspaceOutcome::SessionChanged
            }
            WorkspaceMessage::StreamCompleted => {
                self.is_streaming = false;
                self.streaming_message_id = None;
                WorkspaceOutcome::SessionChanged
            }
            WorkspaceMessage::StreamFailed(error) => {
                self.is_streaming = false;
                if let Some(id) = self.streaming_message_id {
                    self.thread.append_delta(id, &format!("\n[error: {error}]"));
                }
                self.streaming_message_id = None;
                WorkspaceOutcome::SessionChanged
            }
        }
    }

    fn send_message(&mut self) -> WorkspaceOutcome {
        if self.is_streaming {
            return WorkspaceOutcome::None;
        }

        let content = self.draft.trim();
        if content.is_empty() {
            return WorkspaceOutcome::None;
        }

        if !self.has_api_key {
            self.overlay = WorkspaceOverlay::ApiKeySettings;
            return WorkspaceOutcome::None;
        }

        self.thread.push_user(content.to_owned());
        self.draft.clear();

        let assistant = self.thread.push_assistant(String::new());
        self.is_streaming = true;
        self.streaming_message_id = Some(assistant.id);

        WorkspaceOutcome::StreamRequested(ChatRequest {
            model: self.model.clone(),
            messages: self.thread.messages().to_vec(),
            api_key: String::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::workspace_model::{ChatMessage, ChatRole};
    use super::*;

    fn sample_state() -> WorkspaceState {
        let mut state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        state.has_api_key = true;
        state
    }

    #[test]
    fn draft_edit_marks_session_changed() {
        let mut state = sample_state();
        let outcome = state.update(WorkspaceMessage::DraftChanged(String::from("hello")));
        assert_eq!(outcome, WorkspaceOutcome::SessionChanged);
        assert_eq!(state.draft, "hello");
    }

    #[test]
    fn send_without_api_key_opens_settings() {
        let mut state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        state.draft = String::from("hello");
        let outcome = state.update(WorkspaceMessage::SendPressed);
        assert_eq!(outcome, WorkspaceOutcome::None);
        assert_eq!(state.overlay, WorkspaceOverlay::ApiKeySettings);
        assert!(state.thread.is_empty());
    }

    #[test]
    fn send_with_key_starts_streaming() {
        let mut state = sample_state();
        state.draft = String::from("hello");
        let outcome = state.update(WorkspaceMessage::SendPressed);
        assert!(matches!(outcome, WorkspaceOutcome::StreamRequested(_)));
        assert!(state.is_streaming);
        assert_eq!(state.thread.messages().len(), 2);
        assert_eq!(state.thread.messages()[0].role, ChatRole::User);
        assert_eq!(state.thread.messages()[0].content, "hello");
        assert!(state.draft.is_empty());
    }

    #[test]
    fn stream_delta_appends_to_assistant_message() {
        let mut state = sample_state();
        state.draft = String::from("hello");
        state.update(WorkspaceMessage::SendPressed);
        state.update(WorkspaceMessage::StreamDelta(String::from("world")));
        assert_eq!(state.thread.messages()[1].content, "world");
    }

    #[test]
    fn stream_completed_clears_streaming_flag() {
        let mut state = sample_state();
        state.draft = String::from("hello");
        state.update(WorkspaceMessage::SendPressed);
        state.update(WorkspaceMessage::StreamCompleted);
        assert!(!state.is_streaming);
        assert!(state.streaming_message_id.is_none());
    }

    #[test]
    fn close_project_confirm_returns_project_closed() {
        let mut state = sample_state();
        state.overlay = WorkspaceOverlay::CloseProjectConfirm;
        let outcome = state.update(WorkspaceMessage::CloseProjectConfirm);
        assert_eq!(outcome, WorkspaceOutcome::ProjectClosed);
        assert_eq!(state.overlay, WorkspaceOverlay::None);
    }

    #[test]
    fn restore_preserves_thread_and_draft() {
        let thread = ChatThread::from_messages(vec![ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: String::from("saved"),
        }]);
        let state = WorkspaceState::restore(
            PathBuf::from("/tmp/project"),
            ThemeMode::Dark,
            String::from("draft"),
            String::from("openai/gpt-4o-mini"),
            thread,
        );
        assert_eq!(state.draft, "draft");
        assert_eq!(state.thread.messages().len(), 1);
    }
}
