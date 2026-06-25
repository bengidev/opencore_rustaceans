//! Workspace screen state reducer.

use std::path::PathBuf;

use crate::features::chat::{ChatEvent, ChatState, ChatThread};
use crate::shared::design::{OpenCoreTheme, ThemeMode};

use super::workspace_ai_provider::DEFAULT_MODEL;
use super::workspace_chat::{chat_event_from, workspace_outcome_from};
use super::workspace_messages::WorkspaceMessage;
use super::workspace_openrouter_catalog::ModelOption;
use super::workspace_outcome::WorkspaceOutcome;
use super::workspace_overlay::WorkspaceOverlay;

pub struct WorkspaceState {
    pub project_path: PathBuf,
    pub chat: ChatState,
    pub model: String,
    pub available_models: Vec<ModelOption>,
    pub models_loading: bool,
    pub model_query: String,
    pub overlay: WorkspaceOverlay,
    pub api_key_input: String,
    pub api_key_status: Option<String>,
    pub has_api_key: bool,
    pub theme: OpenCoreTheme,
}

impl std::fmt::Debug for WorkspaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceState")
            .field("project_path", &self.project_path)
            .field("draft", &self.chat.draft)
            .field("model", &self.model)
            .field("overlay", &self.overlay)
            .field("has_api_key", &self.has_api_key)
            .field("is_streaming", &self.chat.is_streaming)
            .field("models_loading", &self.models_loading)
            .finish()
    }
}

impl WorkspaceState {
    pub fn new(project_path: PathBuf, theme_mode: ThemeMode) -> Self {
        Self::base(
            project_path,
            theme_mode,
            ChatState::new(),
            DEFAULT_MODEL.into(),
        )
    }

    pub fn restore(
        project_path: PathBuf,
        theme_mode: ThemeMode,
        draft: String,
        model: String,
        thread: ChatThread,
    ) -> Self {
        Self::base(
            project_path,
            theme_mode,
            ChatState::restore(draft, thread),
            model,
        )
    }

    fn base(
        project_path: PathBuf,
        theme_mode: ThemeMode,
        chat: ChatState,
        model: String,
    ) -> Self {
        Self {
            project_path,
            chat,
            model,
            available_models: Vec::new(),
            models_loading: false,
            model_query: String::new(),
            overlay: WorkspaceOverlay::None,
            api_key_input: String::new(),
            api_key_status: None,
            has_api_key: false,
            theme: OpenCoreTheme::from_mode(theme_mode),
        }
    }

    /// Label for the model chip in the composer rail.
    pub fn model_chip_label(&self) -> String {
        if !self.has_api_key {
            return String::from("Not Available");
        }
        if self.models_loading {
            return String::from("Loading…");
        }
        self.model_display_name()
    }

    pub fn model_display_name(&self) -> String {
        self.available_models
            .iter()
            .find(|option| option.id == self.model)
            .map(|option| option.name.clone())
            .unwrap_or_else(|| self.model.clone())
    }

    pub fn filtered_models(&self) -> Vec<&ModelOption> {
        let query = self.model_query.trim().to_lowercase();
        self.available_models
            .iter()
            .filter(|option| {
                query.is_empty()
                    || option.name.to_lowercase().contains(&query)
                    || option.id.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn update(&mut self, message: WorkspaceMessage) -> WorkspaceOutcome {
        if matches!(message, WorkspaceMessage::ModelChipPressed) {
            return self.model_chip_pressed();
        }

        if let Some(event) = chat_event_from(&message) {
            return self.update_chat(event);
        }

        match message {
            WorkspaceMessage::ApiKeyInputChanged(value) => {
                self.api_key_input = value;
                self.api_key_status = None;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeySave => {
                if self.api_key_input.trim().is_empty() {
                    return WorkspaceOutcome::None;
                }
                self.overlay = WorkspaceOverlay::None;
                self.api_key_input.clear();
                self.models_loading = true;
                WorkspaceOutcome::ModelsFetchRequested
            }
            WorkspaceMessage::ApiKeyRemove => {
                self.overlay = WorkspaceOverlay::None;
                self.has_api_key = false;
                self.api_key_input.clear();
                self.available_models.clear();
                self.models_loading = false;
                WorkspaceOutcome::SessionChanged
            }
            WorkspaceMessage::ApiKeyDismiss => {
                self.overlay = WorkspaceOverlay::None;
                self.api_key_input.clear();
                self.api_key_status = None;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ApiKeyPresenceChanged(present) => {
                self.has_api_key = present;
                if present && self.available_models.is_empty() && !self.models_loading {
                    self.models_loading = true;
                    return WorkspaceOutcome::ModelsFetchRequested;
                }
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ModelPickerDismiss => {
                self.overlay = WorkspaceOverlay::None;
                self.model_query.clear();
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ModelPickerQueryChanged(query) => {
                self.model_query = query;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ModelPickerSelect(index) => {
                let models = self.filtered_models();
                if let Some(option) = models.get(index) {
                    self.model = option.id.clone();
                    self.overlay = WorkspaceOverlay::None;
                    self.model_query.clear();
                    return WorkspaceOutcome::SessionChanged;
                }
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ModelsLoadStarted => {
                self.models_loading = true;
                WorkspaceOutcome::None
            }
            WorkspaceMessage::ModelsLoaded(models) => {
                self.models_loading = false;
                self.available_models = models;
                if !self
                    .available_models
                    .iter()
                    .any(|option| option.id == self.model)
                    && let Some(fallback) = self
                        .available_models
                        .iter()
                        .find(|option| option.id == DEFAULT_MODEL)
                        .or_else(|| self.available_models.first())
                {
                    self.model = fallback.id.clone();
                }
                WorkspaceOutcome::SessionChanged
            }
            WorkspaceMessage::ModelsLoadFailed(error) => {
                self.models_loading = false;
                eprintln!("failed to load OpenRouter models: {error}");
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
            _ => WorkspaceOutcome::None,
        }
    }

    fn update_chat(&mut self, event: ChatEvent) -> WorkspaceOutcome {
        if matches!(
            event,
            ChatEvent::ApiKeyHintPressed | ChatEvent::ConfigureActionsPressed
        ) {
            self.overlay = WorkspaceOverlay::ApiKeySettings;
            self.api_key_input.clear();
            return WorkspaceOutcome::None;
        }

        let outcome = self.chat.update(event, self.has_api_key);
        if matches!(outcome, crate::features::chat::ChatOutcome::ApiKeyRequired) {
            self.overlay = WorkspaceOverlay::ApiKeySettings;
            return WorkspaceOutcome::None;
        }

        workspace_outcome_from(outcome, &self.model)
    }

    fn model_chip_pressed(&mut self) -> WorkspaceOutcome {
        if !self.has_api_key {
            self.overlay = WorkspaceOverlay::ApiKeySettings;
            return WorkspaceOutcome::None;
        }
        if self.available_models.is_empty() && !self.models_loading {
            self.models_loading = true;
            return WorkspaceOutcome::ModelsFetchRequested;
        }
        self.open_model_picker();
        WorkspaceOutcome::None
    }

    pub fn open_model_picker(&mut self) {
        self.model_query.clear();
        self.overlay = WorkspaceOverlay::ModelPicker;
    }
}

#[cfg(test)]
mod tests {
    use crate::features::chat::{ChatMessage, ChatRole};

    use super::*;

    fn sample_state() -> WorkspaceState {
        let mut state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        state.has_api_key = true;
        state.available_models = vec![ModelOption {
            id: String::from("openai/gpt-4o-mini"),
            name: String::from("GPT-4o mini"),
        }];
        state
    }

    #[test]
    fn model_chip_label_without_key_is_not_available() {
        let state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        assert_eq!(state.model_chip_label(), "Not Available");
    }

    #[test]
    fn model_chip_label_uses_catalog_name() {
        let state = sample_state();
        assert_eq!(state.model_chip_label(), "GPT-4o mini");
    }

    #[test]
    fn api_key_save_requests_model_fetch() {
        let mut state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        state.api_key_input = String::from("sk-or-test");
        let outcome = state.update(WorkspaceMessage::ApiKeySave);
        assert_eq!(outcome, WorkspaceOutcome::ModelsFetchRequested);
        assert!(!state.has_api_key);
        assert!(state.models_loading);
    }

    #[test]
    fn models_loaded_updates_catalog() {
        let mut state = sample_state();
        state.available_models.clear();
        let outcome = state.update(WorkspaceMessage::ModelsLoaded(vec![ModelOption {
            id: String::from("anthropic/claude-3.5-sonnet"),
            name: String::from("Claude 3.5 Sonnet"),
        }]));
        assert_eq!(outcome, WorkspaceOutcome::SessionChanged);
        assert_eq!(state.model, "anthropic/claude-3.5-sonnet");
    }

    #[test]
    fn draft_edit_marks_session_changed() {
        let mut state = sample_state();
        let outcome = state.update(WorkspaceMessage::DraftChanged(String::from("hello")));
        assert_eq!(outcome, WorkspaceOutcome::SessionChanged);
        assert_eq!(state.chat.draft, "hello");
    }

    #[test]
    fn send_without_api_key_opens_settings() {
        let mut state = WorkspaceState::new(PathBuf::from("/tmp/project"), ThemeMode::Dark);
        state.chat.draft = String::from("hello");
        let outcome = state.update(WorkspaceMessage::SendPressed);
        assert_eq!(outcome, WorkspaceOutcome::None);
        assert_eq!(state.overlay, WorkspaceOverlay::ApiKeySettings);
        assert!(state.chat.thread.is_empty());
    }

    #[test]
    fn send_with_key_starts_streaming() {
        let mut state = sample_state();
        state.chat.draft = String::from("hello");
        let outcome = state.update(WorkspaceMessage::SendPressed);
        assert!(matches!(outcome, WorkspaceOutcome::StreamRequested(_)));
        assert!(state.chat.is_streaming);
        assert_eq!(state.chat.thread.messages().len(), 2);
        assert_eq!(state.chat.thread.messages()[0].role, ChatRole::User);
        assert_eq!(state.chat.thread.messages()[0].content, "hello");
        assert!(state.chat.draft.is_empty());
    }

    #[test]
    fn stream_delta_appends_to_assistant_message() {
        let mut state = sample_state();
        state.chat.draft = String::from("hello");
        state.update(WorkspaceMessage::SendPressed);
        state.update(WorkspaceMessage::StreamDelta(String::from("world")));
        assert_eq!(state.chat.thread.messages()[1].content, "world");
    }

    #[test]
    fn stream_completed_clears_streaming_flag() {
        let mut state = sample_state();
        state.chat.draft = String::from("hello");
        state.update(WorkspaceMessage::SendPressed);
        state.update(WorkspaceMessage::StreamCompleted);
        assert!(!state.chat.is_streaming);
        assert!(state.chat.streaming_message_id.is_none());
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
        assert_eq!(state.chat.draft, "draft");
        assert_eq!(state.chat.thread.messages().len(), 1);
    }
}
