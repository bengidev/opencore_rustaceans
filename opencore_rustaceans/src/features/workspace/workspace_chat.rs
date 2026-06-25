//! Bridge between workspace messages and the chat module.

use crate::features::chat::{ChatEvent, ChatOutcome, ChatRole};

use super::workspace_ai_provider::ChatRequest;
use super::workspace_messages::WorkspaceMessage;
use super::workspace_outcome::WorkspaceOutcome;

pub fn chat_event_from(message: &WorkspaceMessage) -> Option<ChatEvent> {
    match message {
        WorkspaceMessage::DraftChanged(draft) => Some(ChatEvent::DraftChanged(draft.clone())),
        WorkspaceMessage::SendPressed => Some(ChatEvent::SendPressed),
        WorkspaceMessage::ApiKeyHintPressed => Some(ChatEvent::ApiKeyHintPressed),
        WorkspaceMessage::ConfigureActionsPressed => Some(ChatEvent::ConfigureActionsPressed),
        WorkspaceMessage::StreamDelta(delta) => Some(ChatEvent::StreamDelta(delta.clone())),
        WorkspaceMessage::StreamCompleted => Some(ChatEvent::StreamCompleted),
        WorkspaceMessage::StreamFailed(error) => Some(ChatEvent::StreamFailed(error.clone())),
        WorkspaceMessage::Noop => Some(ChatEvent::Noop),
        _ => None,
    }
}

pub fn workspace_message_from(event: ChatEvent) -> WorkspaceMessage {
    match event {
        ChatEvent::DraftChanged(draft) => WorkspaceMessage::DraftChanged(draft),
        ChatEvent::SendPressed => WorkspaceMessage::SendPressed,
        ChatEvent::ApiKeyHintPressed => WorkspaceMessage::ApiKeyHintPressed,
        ChatEvent::ConfigureActionsPressed => WorkspaceMessage::ConfigureActionsPressed,
        ChatEvent::ModelChipPressed => WorkspaceMessage::ModelChipPressed,
        ChatEvent::StreamDelta(delta) => WorkspaceMessage::StreamDelta(delta),
        ChatEvent::StreamCompleted => WorkspaceMessage::StreamCompleted,
        ChatEvent::StreamFailed(error) => WorkspaceMessage::StreamFailed(error),
        ChatEvent::Noop => WorkspaceMessage::Noop,
    }
}

pub fn workspace_outcome_from(
    outcome: ChatOutcome,
    model: &str,
) -> WorkspaceOutcome {
    match outcome {
        ChatOutcome::None => WorkspaceOutcome::None,
        ChatOutcome::SessionChanged => WorkspaceOutcome::SessionChanged,
        ChatOutcome::ApiKeyRequired => WorkspaceOutcome::None,
        ChatOutcome::SendRequested(messages) => {
            let outbound = messages
                .into_iter()
                .filter(|message| {
                    !(message.role == ChatRole::Assistant && message.content.is_empty())
                })
                .collect();
            WorkspaceOutcome::StreamRequested(ChatRequest {
                model: model.to_owned(),
                messages: outbound,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_chat_events() {
        let event = ChatEvent::DraftChanged(String::from("hello"));
        let message = workspace_message_from(event.clone());
        assert_eq!(chat_event_from(&message), Some(event));
    }
}
