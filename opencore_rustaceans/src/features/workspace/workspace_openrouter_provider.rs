//! OpenRouter streaming chat adapter.

use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use reqwest::Client;

use super::workspace_ai_provider::{AiError, AiProvider, ChatRequest, ChatStreamEvent};
use super::workspace_model::ChatRole;
use super::workspace_sse::parse_sse_chunk;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Clone)]
pub struct OpenRouterProvider {
    client: Client,
}

impl Default for OpenRouterProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn build_body(request: &ChatRequest) -> serde_json::Value {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|message| {
                let role = match message.role {
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                };
                serde_json::json!({
                    "role": role,
                    "content": message.content,
                })
            })
            .collect();

        serde_json::json!({
            "model": request.model,
            "stream": true,
            "messages": messages,
        })
    }
}

impl AiProvider for OpenRouterProvider {
    fn stream_chat(
        &self,
        request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatStreamEvent, AiError>> + Send>> {
        let client = self.client.clone();
        Box::pin(openrouter_stream(client, request))
    }
}

fn openrouter_stream(
    client: Client,
    request: ChatRequest,
) -> impl Stream<Item = Result<ChatStreamEvent, AiError>> {
    futures_util::stream::unfold(StreamState::Connecting { client, request }, |state| async {
        match state {
            StreamState::Connecting { client, request } => {
                let body = OpenRouterProvider::build_body(&request);
                let api_key = request.api_key;
                let response = client
                    .post(OPENROUTER_URL)
                    .bearer_auth(api_key)
                    .header("HTTP-Referer", "https://opencore.app")
                    .header("X-Title", "OpenCore")
                    .json(&body)
                    .send()
                    .await;

                match response {
                    Ok(response) if response.status().is_success() => Some((
                        None,
                        StreamState::Reading {
                            byte_stream: Box::pin(response.bytes_stream()),
                            buffer: String::new(),
                            pending: Vec::new(),
                        },
                    )),
                    Ok(response) => {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        Some((
                            Some(Err(AiError::Request(format!("HTTP {status}: {body}")))),
                            StreamState::Finished,
                        ))
                    }
                    Err(error) => Some((
                        Some(Err(AiError::Request(error.to_string()))),
                        StreamState::Finished,
                    )),
                }
            }
            StreamState::Reading {
                mut byte_stream,
                mut buffer,
                mut pending,
            } => {
                if let Some(event) = pending.pop() {
                    return Some((
                        Some(Ok(event)),
                        StreamState::Reading {
                            byte_stream,
                            buffer,
                            pending,
                        },
                    ));
                }

                while let Some(chunk) = byte_stream.next().await {
                    match chunk {
                        Ok(chunk) => {
                            buffer.push_str(&String::from_utf8_lossy(&chunk));
                            if let Some(frame_end) = buffer.find("\n\n") {
                                let frame: String = buffer.drain(..frame_end + 2).collect();
                                match parse_sse_chunk(&frame) {
                                    Ok(events) if !events.is_empty() => {
                                        let mut events = events.into_iter();
                                        let first = events.next().unwrap();
                                        pending.extend(events);
                                        return Some((
                                            Some(Ok(first)),
                                            StreamState::Reading {
                                                byte_stream,
                                                buffer,
                                                pending,
                                            },
                                        ));
                                    }
                                    Ok(_) => continue,
                                    Err(error) => {
                                        return Some((Some(Err(error)), StreamState::Finished));
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            return Some((
                                Some(Err(AiError::Request(error.to_string()))),
                                StreamState::Finished,
                            ));
                        }
                    }
                }

                if !buffer.trim().is_empty() {
                    match parse_sse_chunk(&buffer) {
                        Ok(events) if !events.is_empty() => {
                            let mut events = events.into_iter();
                            let first = events.next().unwrap();
                            pending.extend(events);
                            return Some((
                                Some(Ok(first)),
                                StreamState::Reading {
                                    byte_stream,
                                    buffer: String::new(),
                                    pending,
                                },
                            ));
                        }
                        Ok(_) => {}
                        Err(error) => {
                            return Some((Some(Err(error)), StreamState::Finished));
                        }
                    }
                }

                Some((Some(Ok(ChatStreamEvent::Done)), StreamState::Finished))
            }
            StreamState::Finished => None,
        }
    })
    .filter_map(|item| async move { item })
}

enum StreamState {
    Connecting {
        client: Client,
        request: ChatRequest,
    },
    Reading {
        byte_stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
        buffer: String,
        pending: Vec<ChatStreamEvent>,
    },
    Finished,
}

#[cfg(test)]
mod tests {
    use super::super::workspace_ai_provider::OPENROUTER_PROVIDER_ID;
    use super::super::workspace_model::ChatMessage;
    use super::*;

    #[test]
    fn build_body_includes_stream_flag_and_messages() {
        let request = ChatRequest {
            model: String::from("openai/gpt-4o-mini"),
            messages: vec![ChatMessage {
                id: 1,
                role: ChatRole::User,
                content: String::from("hello"),
            }],
            api_key: String::from("test"),
        };
        let body = OpenRouterProvider::build_body(&request);
        assert_eq!(body["stream"], serde_json::json!(true));
        assert_eq!(body["model"], serde_json::json!("openai/gpt-4o-mini"));
        assert_eq!(body["messages"][0]["content"], serde_json::json!("hello"));
    }

    #[test]
    fn provider_id_constant_matches_openrouter() {
        assert_eq!(OPENROUTER_PROVIDER_ID, "openrouter");
    }
}
