//! OpenRouter streaming chat adapter.

use std::pin::Pin;
use std::sync::Arc;

use futures_util::{Stream, StreamExt};
use reqwest::Client;

use super::workspace_ai_provider::{
    AiError, AiProvider, ChatRequest, ChatStreamEvent, OPENROUTER_PROVIDER_ID, format_http_error,
    openrouter_http_client,
};
use super::workspace_credential_store::WorkspaceCredentialStore;
use super::workspace_sse::parse_sse_chunk;
use crate::features::chat::ChatRole;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Clone)]
pub struct OpenRouterProvider {
    client: Client,
    credentials: Arc<dyn WorkspaceCredentialStore>,
}

impl std::fmt::Debug for OpenRouterProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenRouterProvider").finish_non_exhaustive()
    }
}

impl OpenRouterProvider {
    pub fn new(credentials: Arc<dyn WorkspaceCredentialStore>) -> Self {
        Self {
            client: openrouter_http_client(),
            credentials,
        }
    }

    fn resolve_api_key(&self) -> Option<String> {
        self.credentials.resolved_secret(OPENROUTER_PROVIDER_ID)
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
        let api_key = self.resolve_api_key();
        Box::pin(openrouter_stream(client, request, api_key))
    }
}

fn openrouter_stream(
    client: Client,
    request: ChatRequest,
    api_key: Option<String>,
) -> impl Stream<Item = Result<ChatStreamEvent, AiError>> {
    futures_util::stream::unfold(
        StreamState::Connecting {
            client,
            request,
            api_key,
        },
        |state| async move {
            match state {
                StreamState::Connecting {
                    client,
                    request,
                    api_key,
                } => {
                    let Some(api_key) = api_key else {
                        return Some((
                            Some(Err(AiError::Request(String::from(
                                "missing OpenRouter API key",
                            )))),
                            StreamState::Finished,
                        ));
                    };

                    let body = OpenRouterProvider::build_body(&request);
                    let response = client
                        .post(OPENROUTER_URL)
                        .bearer_auth(&api_key)
                        .header("HTTP-Referer", "https://opencore.app")
                        .header("X-Title", "OpenRouter")
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
                                Some(Err(AiError::Request(format_http_error(status, &body)))),
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
        },
    )
    .filter_map(|item| async move { item })
}

enum StreamState {
    Connecting {
        client: Client,
        request: ChatRequest,
        api_key: Option<String>,
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
    use super::super::workspace_memory_credential::InMemoryWorkspaceCredentialStore;
    use super::*;
    use crate::features::chat::ChatMessage;

    #[test]
    fn build_body_includes_stream_flag_and_messages() {
        let request = ChatRequest {
            model: String::from("openai/gpt-4o-mini"),
            messages: vec![ChatMessage {
                id: 1,
                role: ChatRole::User,
                content: String::from("hello"),
            }],
        };
        let body = OpenRouterProvider::build_body(&request);
        assert_eq!(body["stream"], serde_json::json!(true));
        assert_eq!(body["model"], serde_json::json!("openai/gpt-4o-mini"));
        assert_eq!(body["messages"][0]["content"], serde_json::json!("hello"));
    }

    #[test]
    fn resolve_api_key_reads_from_credential_store() {
        let credentials = Arc::new(InMemoryWorkspaceCredentialStore::new());
        credentials
            .save("sk-or-test-key", OPENROUTER_PROVIDER_ID)
            .unwrap();
        let provider = OpenRouterProvider::new(credentials);
        assert_eq!(
            provider.resolve_api_key(),
            Some(String::from("sk-or-test-key"))
        );
    }

    #[test]
    fn resolve_api_key_requires_non_empty_store_value() {
        let credentials = Arc::new(InMemoryWorkspaceCredentialStore::new());
        let provider = OpenRouterProvider::new(credentials);
        assert_eq!(provider.resolve_api_key(), None);
    }
}
