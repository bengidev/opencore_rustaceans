//! Chat thread and composer state reducer.

use super::chat_messages::ChatEvent;
use super::chat_model::ChatThread;
use super::chat_outcome::ChatOutcome;

/// Default context-window budget when the selected model has no known limit.
pub const DEFAULT_TOKEN_BUDGET: u32 = 128_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenEstimate {
    pub used: u32,
    pub budget: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatState {
    pub thread: ChatThread,
    pub draft: String,
    pub is_streaming: bool,
    pub streaming_message_id: Option<u64>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            thread: ChatThread::new(),
            draft: String::new(),
            is_streaming: false,
            streaming_message_id: None,
        }
    }

    pub fn restore(draft: String, thread: ChatThread) -> Self {
        Self {
            thread,
            draft,
            is_streaming: false,
            streaming_message_id: None,
        }
    }

    /// Rough token count from thread content and draft (chars / 4).
    pub fn estimated_tokens(&self) -> u32 {
        let chars: usize = self
            .thread
            .messages()
            .iter()
            .map(|message| message.content.len())
            .sum::<usize>()
            + self.draft.len();
        (chars as u32) / 4
    }

    pub fn token_estimate(&self, budget: u32) -> TokenEstimate {
        TokenEstimate {
            used: self.estimated_tokens(),
            budget,
        }
    }

    pub fn update(&mut self, event: ChatEvent, has_api_key: bool) -> ChatOutcome {
        match event {
            ChatEvent::DraftChanged(draft) => {
                self.draft = draft;
                ChatOutcome::SessionChanged
            }
            ChatEvent::SendPressed => self.send_message(has_api_key),
            ChatEvent::StreamDelta(delta) => {
                if let Some(id) = self.streaming_message_id {
                    self.thread.append_delta(id, &delta);
                }
                ChatOutcome::SessionChanged
            }
            ChatEvent::StreamCompleted => {
                self.is_streaming = false;
                self.streaming_message_id = None;
                ChatOutcome::SessionChanged
            }
            ChatEvent::StreamFailed(error) => {
                self.is_streaming = false;
                if let Some(id) = self.streaming_message_id {
                    let message = truncate_error(&error);
                    self.thread
                        .append_delta(id, &format!("\n[error: {message}]"));
                }
                self.streaming_message_id = None;
                ChatOutcome::SessionChanged
            }
            ChatEvent::ApiKeyHintPressed
            | ChatEvent::ConfigureActionsPressed
            | ChatEvent::ModelChipPressed
            | ChatEvent::FolderScopePressed
            | ChatEvent::Noop => ChatOutcome::None,
        }
    }

    fn send_message(&mut self, has_api_key: bool) -> ChatOutcome {
        if self.is_streaming {
            return ChatOutcome::None;
        }

        let content = self.draft.trim();
        if content.is_empty() {
            return ChatOutcome::None;
        }

        if !has_api_key {
            return ChatOutcome::ApiKeyRequired;
        }

        self.thread.push_user(content.to_owned());
        self.draft.clear();

        let assistant = self.thread.push_assistant(String::new());
        self.is_streaming = true;
        self.streaming_message_id = Some(assistant.id);

        ChatOutcome::SendRequested(self.thread.messages().to_vec())
    }
}

fn truncate_error(message: &str) -> String {
    const MAX: usize = 200;
    let trimmed = message.trim();
    if trimmed.len() <= MAX {
        trimmed.to_owned()
    } else {
        format!("{}…", &trimmed[..MAX])
    }
}

#[cfg(test)]
mod tests {
    use super::super::chat_model::{ChatMessage, ChatRole};
    use super::*;

    #[test]
    fn draft_edit_marks_session_changed() {
        let mut state = ChatState::new();
        let outcome = state.update(ChatEvent::DraftChanged(String::from("hello")), true);
        assert_eq!(outcome, ChatOutcome::SessionChanged);
        assert_eq!(state.draft, "hello");
    }

    #[test]
    fn send_without_api_key_requires_key() {
        let mut state = ChatState::new();
        state.draft = String::from("hello");
        let outcome = state.update(ChatEvent::SendPressed, false);
        assert_eq!(outcome, ChatOutcome::ApiKeyRequired);
        assert!(state.thread.is_empty());
    }

    #[test]
    fn send_with_key_starts_streaming() {
        let mut state = ChatState::new();
        state.draft = String::from("hello");
        let outcome = state.update(ChatEvent::SendPressed, true);
        assert!(matches!(outcome, ChatOutcome::SendRequested(_)));
        assert!(state.is_streaming);
        assert_eq!(state.thread.messages().len(), 2);
        assert_eq!(state.thread.messages()[0].role, ChatRole::User);
        assert_eq!(state.thread.messages()[0].content, "hello");
        assert!(state.draft.is_empty());
    }

    #[test]
    fn stream_delta_appends_to_assistant_message() {
        let mut state = ChatState::new();
        state.draft = String::from("hello");
        state.update(ChatEvent::SendPressed, true);
        state.update(ChatEvent::StreamDelta(String::from("world")), true);
        assert_eq!(state.thread.messages()[1].content, "world");
    }

    #[test]
    fn stream_completed_clears_streaming_flag() {
        let mut state = ChatState::new();
        state.draft = String::from("hello");
        state.update(ChatEvent::SendPressed, true);
        state.update(ChatEvent::StreamCompleted, true);
        assert!(!state.is_streaming);
        assert!(state.streaming_message_id.is_none());
    }

    #[test]
    fn restore_preserves_thread_and_draft() {
        let thread = ChatThread::from_messages(vec![ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: String::from("saved"),
        }]);
        let state = ChatState::restore(String::from("draft"), thread);
        assert_eq!(state.draft, "draft");
        assert_eq!(state.thread.messages().len(), 1);
    }

    #[test]
    fn estimated_tokens_is_zero_when_empty() {
        let state = ChatState::new();
        assert_eq!(state.estimated_tokens(), 0);
    }

    #[test]
    fn estimated_tokens_counts_thread_and_draft() {
        let mut state = ChatState::new();
        state.draft = String::from("abcd");
        state.thread.push_user(String::from("1234"));
        assert_eq!(state.estimated_tokens(), 2);
    }

    #[test]
    fn token_estimate_carries_budget() {
        let state = ChatState::new();
        let estimate = state.token_estimate(128_000);
        assert_eq!(estimate.used, 0);
        assert_eq!(estimate.budget, 128_000);
    }
}
