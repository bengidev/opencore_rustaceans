//! OpenRouter model catalog client.

use serde::Deserialize;

use super::workspace_ai_provider::{AiError, format_http_error, openrouter_http_client};

const MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

/// One selectable model from the OpenRouter catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelRecord>,
}

#[derive(Debug, Deserialize)]
struct ModelRecord {
    id: String,
    name: String,
}

/// Fetch the user-visible model list from OpenRouter.
///
/// The catalog endpoint is public; an API key is optional.
pub async fn fetch_openrouter_models(api_key: Option<&str>) -> Result<Vec<ModelOption>, AiError> {
    let client = openrouter_http_client();
    let mut request = client
        .get(MODELS_URL)
        .header("HTTP-Referer", "https://opencore.app")
        .header("X-Title", "OpenRouter");

    if let Some(key) = api_key.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(key.trim());
    }

    let response = request
        .send()
        .await
        .map_err(|error| AiError::Request(error.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AiError::Request(format_http_error(status, &body)));
    }

    let payload: ModelsResponse = response
        .json()
        .await
        .map_err(|error| AiError::Request(error.to_string()))?;

    let mut models: Vec<ModelOption> = payload
        .data
        .into_iter()
        .map(|record| ModelOption {
            id: record.id,
            name: record.name,
        })
        .collect();

    models.sort_by_key(|option| option.name.to_lowercase());
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_models_payload() {
        let raw = serde_json::json!({
            "data": [
                { "id": "openai/gpt-4o-mini", "name": "GPT-4o mini" },
                { "id": "anthropic/claude-3.5-sonnet", "name": "Claude 3.5 Sonnet" }
            ]
        });

        let payload: ModelsResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(payload.data.len(), 2);
        assert_eq!(payload.data[0].id, "openai/gpt-4o-mini");
    }
}
