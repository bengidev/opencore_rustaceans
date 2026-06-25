//! Chat thread composite — user and assistant message nodes.

use serde::{Deserialize, Serialize};

/// Who authored a chat message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

/// One message in the workspace chat thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub role: ChatRole,
    pub content: String,
}

/// Append-only chat thread with stable message ids.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChatThread {
    messages: Vec<ChatMessage>,
    next_id: u64,
}

impl ChatThread {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_id: 1,
        }
    }

    pub fn from_messages(messages: Vec<ChatMessage>) -> Self {
        let next_id = messages.iter().map(|message| message.id).max().unwrap_or(0) + 1;
        Self { messages, next_id }
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn push_user(&mut self, content: String) -> ChatMessage {
        self.push(ChatRole::User, content)
    }

    pub fn push_assistant(&mut self, content: String) -> ChatMessage {
        self.push(ChatRole::Assistant, content)
    }

    pub fn append_delta(&mut self, message_id: u64, delta: &str) {
        if let Some(message) = self.messages.iter_mut().find(|m| m.id == message_id) {
            message.content.push_str(delta);
        }
    }

    fn push(&mut self, role: ChatRole, content: String) -> ChatMessage {
        let message = ChatMessage {
            id: self.next_id,
            role,
            content,
        };
        self.next_id += 1;
        self.messages.push(message.clone());
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_starts_empty() {
        let thread = ChatThread::new();
        assert!(thread.is_empty());
        assert!(thread.messages().is_empty());
    }

    #[test]
    fn append_user_and_assistant_messages() {
        let mut thread = ChatThread::new();
        let user = thread.push_user(String::from("hello"));
        let assistant = thread.push_assistant(String::from("hi there"));

        assert_eq!(user.id, 1);
        assert_eq!(user.role, ChatRole::User);
        assert_eq!(assistant.id, 2);
        assert_eq!(assistant.role, ChatRole::Assistant);
        assert_eq!(thread.messages().len(), 2);
    }

    #[test]
    fn append_delta_extends_assistant_message() {
        let mut thread = ChatThread::new();
        let assistant = thread.push_assistant(String::from("hel"));
        thread.append_delta(assistant.id, "lo");
        assert_eq!(thread.messages()[0].content, "hello");
    }

    #[test]
    fn from_messages_restores_next_id() {
        let messages = vec![
            ChatMessage {
                id: 5,
                role: ChatRole::User,
                content: String::from("a"),
            },
            ChatMessage {
                id: 10,
                role: ChatRole::Assistant,
                content: String::from("b"),
            },
        ];
        let mut thread = ChatThread::from_messages(messages);
        let next = thread.push_user(String::from("c"));
        assert_eq!(next.id, 11);
    }
}
