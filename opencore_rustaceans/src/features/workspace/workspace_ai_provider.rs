//! AI provider strategy and stream event types.

use std::pin::Pin;
use std::time::Duration;

use futures_util::Stream;
use reqwest::Client;

use crate::features::chat::ChatMessage;

pub const DEFAULT_MODEL: &str = "openai/gpt-4o-mini";
pub const OPENROUTER_PROVIDER_ID: &str = "openrouter";

const HTTP_TIMEOUT: Duration = Duration::from_secs(120);
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_USER_ERROR_LEN: usize = 200;
const MAX_HTTP_BODY_SNIPPET: usize = 160;

/// Request payload for a streaming chat completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

/// HTTP client shared by OpenRouter adapters.
pub(crate) fn openrouter_http_client() -> Client {
    Client::builder()
        .timeout(HTTP_TIMEOUT)
        .connect_timeout(HTTP_CONNECT_TIMEOUT)
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// User-facing HTTP error — status plus truncated body snippet.
pub fn format_http_error(status: reqwest::StatusCode, body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return format!("HTTP {status}");
    }
    let snippet = if trimmed.len() > MAX_HTTP_BODY_SNIPPET {
        format!("{}…", &trimmed[..MAX_HTTP_BODY_SNIPPET])
    } else {
        trimmed.to_owned()
    };
    format!("HTTP {status}: {snippet}")
}

/// Truncate arbitrary provider errors before showing or persisting them.
pub fn sanitize_user_error(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.len() <= MAX_USER_ERROR_LEN {
        trimmed.to_owned()
    } else {
        format!("{}…", &trimmed[..MAX_USER_ERROR_LEN])
    }
}

/// Events emitted while streaming an assistant reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatStreamEvent {
    Delta { content: String },
    Done,
    Error(String),
}

/// Errors from AI providers.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("request failed: {0}")]
    Request(String),
    #[error("stream parse error: {0}")]
    Parse(String),
}

/// Strategy for streaming chat completions from an AI backend.
pub trait AiProvider: Send + Sync {
    fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, AiError>> + Send>>;
}

/// Test double returning a fixed event sequence without network I/O.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone)]
pub struct CannedAiProvider {
    events: Vec<Result<ChatStreamEvent, AiError>>,
}

impl CannedAiProvider {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(events: Vec<Result<ChatStreamEvent, AiError>>) -> Self {
        Self { events }
    }
}

impl AiProvider for CannedAiProvider {
    fn stream_chat(
        &self,
        _request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, AiError>> + Send>> {
        let events = self.events.clone();
        Box::pin(futures_util::stream::iter(events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::chat::{ChatMessage, ChatRole};
    use std::task::{Context, Poll};

    fn collect_events(
        provider: &CannedAiProvider,
        request: ChatRequest,
    ) -> Vec<Result<ChatStreamEvent, AiError>> {
        let mut stream = provider.stream_chat(request);
        let waker = futures_util::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut events = Vec::new();
        loop {
            match stream.as_mut().poll_next(&mut cx) {
                Poll::Ready(Some(item)) => events.push(item),
                Poll::Ready(None) => break,
                Poll::Pending => panic!("canned stream should not pend"),
            }
        }
        events
    }

    #[test]
    fn canned_provider_yields_configured_events() {
        let provider = CannedAiProvider::new(vec![
            Ok(ChatStreamEvent::Delta {
                content: String::from("hello"),
            }),
            Ok(ChatStreamEvent::Done),
        ]);
        let request = ChatRequest {
            model: DEFAULT_MODEL.into(),
            messages: vec![ChatMessage {
                id: 1,
                role: ChatRole::User,
                content: String::from("hi"),
            }],
        };

        let events = collect_events(&provider, request);

        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            Ok(ChatStreamEvent::Delta {
                content: String::from("hello")
            })
        );
        assert_eq!(events[1], Ok(ChatStreamEvent::Done));
    }
}
