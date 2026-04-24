//! OpenRouter provider.
//!
//! OpenRouter is a unified gateway to hundreds of AI models through a single
//! OpenAI-compatible endpoint. Supports tool calling for models that have it.
//!
//! Base URL: https://openrouter.ai/api/v1
//! Models: provider/model format (e.g., "openai/gpt-4o", "anthropic/claude-sonnet-4")
//! Auth: Bearer token via Authorization header
//! Extra headers: HTTP-Referer (optional), X-OpenRouter-Title (optional)
//! Docs: https://openrouter.ai/docs

use crate::error::{FerroError, Result};
use crate::provider::{BoxFuture, LlmProvider};
use crate::types::{
    Message, MessageContent, ProviderResponse, Role, TokenUsage, ToolCall, ToolDefinition,
};
use reqwest::Client;
use serde_json::{Value, json};

pub struct OpenRouterProvider {
    api_key: String,
    base_url: String,
    site_url: Option<String>,
    site_name: Option<String>,
    client: Client,
}

impl OpenRouterProvider {
    pub fn new(
        api_key: String,
        base_url: String,
        site_url: Option<String>,
        site_name: Option<String>,
    ) -> Self {
        Self {
            api_key,
            base_url,
            site_url,
            site_name,
            client: Client::new(),
        }
    }

    fn build_request_body(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        model: &str,
        max_tokens: u32,
    ) -> Value {
        let formatted_messages: Vec<Value> = messages.iter().map(|m| format_message(m)).collect();

        let mut body = json!({
            "model": model,
            "max_tokens": max_tokens,
            "messages": formatted_messages,
        });

        if !tools.is_empty() {
            let tool_defs: Vec<Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.input_schema,
                        }
                    })
                })
                .collect();
            body["tools"] = json!(tool_defs);
        }

        body
    }

    fn parse_response(&self, body: &Value) -> Result<ProviderResponse> {
        let choice = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| FerroError::Provider("No choices in OpenRouter response".into()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| FerroError::Provider("No message in OpenRouter choice".into()))?;

        let text = message
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let tool_calls: Vec<ToolCall> = message
            .get("tool_calls")
            .and_then(|tc| tc.as_array())
            .map(|tcs| {
                tcs.iter()
                    .filter_map(|tc| {
                        let id = tc.get("id")?.as_str()?.to_string();
                        let func = tc.get("function")?;
                        let name = func.get("name")?.as_str()?.to_string();
                        let args_str = func.get("arguments")?.as_str()?;
                        let arguments: Value = serde_json::from_str(args_str).ok()?;
                        Some(ToolCall {
                            id,
                            name,
                            arguments,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = body.get("usage").map(|u| TokenUsage {
            input_tokens: u.get("prompt_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
            output_tokens: u
                .get("completion_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
        });

        let stop_reason = choice
            .get("finish_reason")
            .and_then(|s| s.as_str())
            .map(String::from);

        let msg = if tool_calls.is_empty() {
            Message::assistant(text)
        } else {
            let mut msg = Message::assistant_with_tool_calls(tool_calls);
            if !text.is_empty() {
                msg.content = MessageContent::Text(text);
            }
            msg
        };

        Ok(ProviderResponse {
            message: msg,
            usage,
            stop_reason,
        })
    }
}

impl LlmProvider for OpenRouterProvider {
    fn complete<'a>(
        &'a self,
        messages: &'a [Message],
        tools: &'a [ToolDefinition],
        model: &'a str,
        max_tokens: u32,
    ) -> BoxFuture<'a, Result<ProviderResponse>> {
        Box::pin(async move {
            let body = self.build_request_body(messages, tools, model, max_tokens);
            let max_attempts = 3usize;
            let mut last_err: Option<FerroError> = None;

            for attempt in 1..=max_attempts {
                let mut request = self
                    .client
                    .post(format!("{}/chat/completions", self.base_url))
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json");

                // OpenRouter-specific headers for ranking attribution
                if let Some(url) = &self.site_url {
                    request = request.header("HTTP-Referer", url.as_str());
                }
                if let Some(name) = &self.site_name {
                    request = request.header("X-OpenRouter-Title", name.as_str());
                }

                let response = match request.json(&body).send().await {
                    Ok(resp) => resp,
                    Err(e) => {
                        let is_retryable = e.is_timeout() || e.is_connect() || e.is_request();
                        let ferr = FerroError::Provider(format!(
                            "OpenRouter HTTP request failed (attempt {attempt}/{max_attempts}): {e}"
                        ));
                        if is_retryable && attempt < max_attempts {
                            last_err = Some(ferr);
                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                250 * attempt as u64,
                            ))
                            .await;
                            continue;
                        }
                        return Err(ferr);
                    }
                };

                let status = response.status();
                let response_body: Value = response.json().await.map_err(|e| {
                    FerroError::Provider(format!("Failed to parse OpenRouter response: {e}"))
                })?;

                if !status.is_success() {
                    let error_msg = response_body
                        .get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");

                    let retryable_status = status.as_u16() == 408
                        || status.as_u16() == 409
                        || status.as_u16() == 429
                        || status.is_server_error();
                    let ferr = FerroError::Provider(format!(
                        "OpenRouter API error ({status}) attempt {attempt}/{max_attempts}: {error_msg}"
                    ));

                    if retryable_status && attempt < max_attempts {
                        last_err = Some(ferr);
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            250 * attempt as u64,
                        ))
                        .await;
                        continue;
                    }

                    return Err(ferr);
                }

                return self.parse_response(&response_body);
            }

            Err(last_err.unwrap_or_else(|| {
                FerroError::Provider("OpenRouter request failed after retries".into())
            }))
        })
    }

    fn name(&self) -> &str {
        "openrouter"
    }

    fn supports_model(&self, model: &str) -> bool {
        // OpenRouter uses "provider/model" format
        is_openrouter_model(model)
    }
}

/// Check if a model string looks like an OpenRouter model (contains `/`).
pub fn is_openrouter_model(model: &str) -> bool {
    model.contains('/')
}

/// Format a message for OpenRouter (OpenAI-compatible format).
fn format_message(msg: &Message) -> Value {
    let role = match msg.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };

    if msg.role == Role::Tool {
        return json!({
            "role": "tool",
            "tool_call_id": msg.tool_call_id,
            "content": msg.text(),
        });
    }

    if msg.role == Role::Assistant {
        if let Some(tool_calls) = &msg.tool_calls {
            let tc_json: Vec<Value> = tool_calls
                .iter()
                .map(|tc| {
                    json!({
                        "id": tc.id,
                        "type": "function",
                        "function": {
                            "name": tc.name,
                            "arguments": tc.arguments.to_string(),
                        }
                    })
                })
                .collect();
            return json!({
                "role": "assistant",
                "content": msg.text(),
                "tool_calls": tc_json,
            });
        }
    }

    json!({
        "role": role,
        "content": msg.text(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_openrouter_model() {
        assert!(is_openrouter_model("openai/gpt-4o"));
        assert!(is_openrouter_model("anthropic/claude-sonnet-4"));
        assert!(is_openrouter_model("meta-llama/llama-3.1-70b"));
        assert!(!is_openrouter_model("gpt-4o"));
        assert!(!is_openrouter_model("claude-sonnet-4-20250514"));
        assert!(!is_openrouter_model("glm-5"));
    }

    #[test]
    fn test_parse_openrouter_text_response() {
        let provider = OpenRouterProvider::new(
            "test".into(),
            "https://openrouter.ai/api/v1".into(),
            None,
            None,
        );
        let body = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello from OpenRouter!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 12,
                "completion_tokens": 6
            }
        });
        let result = provider.parse_response(&body).unwrap();
        assert_eq!(result.message.text(), "Hello from OpenRouter!");
    }

    #[test]
    fn test_parse_openrouter_tool_call() {
        let provider = OpenRouterProvider::new(
            "test".into(),
            "https://openrouter.ai/api/v1".into(),
            Some("https://example.com".into()),
            Some("Ferroclaw".into()),
        );
        let body = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_456",
                        "type": "function",
                        "function": {
                            "name": "read_file",
                            "arguments": "{\"path\":\"/tmp/test.txt\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });
        let result = provider.parse_response(&body).unwrap();
        let tcs = result.message.tool_calls.as_ref().unwrap();
        assert_eq!(tcs.len(), 1);
        assert_eq!(tcs[0].name, "read_file");
    }

    #[test]
    fn test_build_request_body() {
        let provider = OpenRouterProvider::new(
            "test".into(),
            "https://openrouter.ai/api/v1".into(),
            None,
            None,
        );
        let messages = vec![Message::user("Hello")];
        let body = provider.build_request_body(&messages, &[], "openai/gpt-4o", 4096);
        assert_eq!(body["model"], "openai/gpt-4o");
        assert!(body.get("tools").is_none());
    }
}
