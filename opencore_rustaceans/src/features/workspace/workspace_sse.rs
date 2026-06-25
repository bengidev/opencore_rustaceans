//! SSE line parser for OpenRouter streaming chat completions.

use super::workspace_ai_provider::{AiError, ChatStreamEvent};

/// Parse one SSE `data:` payload into a stream event.
pub fn parse_sse_data(payload: &str) -> Result<Option<ChatStreamEvent>, AiError> {
    let trimmed = payload.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return Ok(None);
    }

    if trimmed == "[DONE]" {
        return Ok(Some(ChatStreamEvent::Done));
    }

    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|error| AiError::Parse(error.to_string()))?;

    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        return Ok(Some(ChatStreamEvent::Error(message.to_owned())));
    }

    let delta = value
        .pointer("/choices/0/delta/content")
        .and_then(|content| content.as_str());

    if let Some(content) = delta.filter(|text| !text.is_empty()) {
        return Ok(Some(ChatStreamEvent::Delta {
            content: content.to_owned(),
        }));
    }

    let finish = value
        .pointer("/choices/0/finish_reason")
        .and_then(|reason| reason.as_str());

    if finish.is_some() {
        return Ok(Some(ChatStreamEvent::Done));
    }

    Ok(None)
}

/// Parse a chunk of SSE text that may contain multiple lines.
pub fn parse_sse_chunk(chunk: &str) -> Result<Vec<ChatStreamEvent>, AiError> {
    let mut events = Vec::new();
    for line in chunk.lines() {
        let Some(payload) = line.strip_prefix("data:") else {
            continue;
        };
        if let Some(event) = parse_sse_data(payload)? {
            events.push(event);
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_content_delta() {
        let event = parse_sse_data(r#"{"choices":[{"delta":{"content":"Hello"}}]}"#)
            .unwrap()
            .unwrap();
        assert_eq!(
            event,
            ChatStreamEvent::Delta {
                content: String::from("Hello")
            }
        );
    }

    #[test]
    fn parses_done_marker() {
        assert_eq!(
            parse_sse_data("[DONE]").unwrap(),
            Some(ChatStreamEvent::Done)
        );
    }

    #[test]
    fn parses_finish_reason_as_done() {
        let event = parse_sse_data(r#"{"choices":[{"finish_reason":"stop"}]}"#)
            .unwrap()
            .unwrap();
        assert_eq!(event, ChatStreamEvent::Done);
    }

    #[test]
    fn parses_error_payload() {
        let event = parse_sse_data(r#"{"error":{"message":"bad key"}}"#)
            .unwrap()
            .unwrap();
        assert_eq!(event, ChatStreamEvent::Error(String::from("bad key")));
    }

    #[test]
    fn parse_chunk_handles_multiple_lines() {
        let chunk = "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\n\
                     data: [DONE]\n";
        let events = parse_sse_chunk(chunk).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            ChatStreamEvent::Delta {
                content: String::from("Hi")
            }
        );
        assert_eq!(events[1], ChatStreamEvent::Done);
    }
}
