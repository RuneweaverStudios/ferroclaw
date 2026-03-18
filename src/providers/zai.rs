//! Zai GLM provider.
//!
//! Z.ai's GLM models use an OpenAI-compatible chat completions format
//! with a unique `tool_stream` parameter for streaming tool calls.
//!
//! Base URL: https://api.z.ai/api/paas/v4
//! Models: glm-5, glm-5-turbo, glm-4.5, glm-4.6, glm-4.7, glm-4.5v, glm-4.6v
//! Auth: Bearer token via Authorization header
//! Docs: https://docs.z.ai/guides/capabilities/stream-tool

use crate::error::{FerroError, Result};
use crate::provider::{BoxFuture, LlmProvider};
use crate::types::{
    Message, MessageContent, ProviderResponse, Role, TokenUsage, ToolCall, ToolDefinition,
};
use reqwest::Client;
use serde_json::{json, Value};

const ZAI_MODELS: &[&str] = &[
    "glm-5",
    "glm-5-turbo",
    "glm-4.5",
    "glm-4.6",
    "glm-4.7",
    "glm-4.5v",
    "glm-4.6v",
    "glm-4-32b-0414-128k",
];

pub struct ZaiProvider {
    api_key: String,
    base_url: String,
    client: Client,
}

impl ZaiProvider {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
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
        let formatted_messages: Vec<Value> =
            messages.iter().map(|m| format_message(m)).collect();

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
            body["tool_choice"] = json!("auto");
        }

        body
    }

    fn parse_response(&self, body: &Value) -> Result<ProviderResponse> {
        let choice = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| FerroError::Provider("No choices in Zai response".into()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| FerroError::Provider("No message in Zai choice".into()))?;

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
            input_tokens: u
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
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

impl LlmProvider for ZaiProvider {
    fn complete<'a>(
        &'a self,
        messages: &'a [Message],
        tools: &'a [ToolDefinition],
        model: &'a str,
        max_tokens: u32,
    ) -> BoxFuture<'a, Result<ProviderResponse>> {
        Box::pin(async move {
            let body = self.build_request_body(messages, tools, model, max_tokens);

            let response = self
                .client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| FerroError::Provider(format!("Zai HTTP request failed: {e}")))?;

            let status = response.status();
            let response_body: Value = response
                .json()
                .await
                .map_err(|e| {
                    FerroError::Provider(format!("Failed to parse Zai response: {e}"))
                })?;

            if !status.is_success() {
                let error_msg = response_body
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(FerroError::Provider(format!(
                    "Zai API error ({status}): {error_msg}"
                )));
            }

            self.parse_response(&response_body)
        })
    }

    fn name(&self) -> &str {
        "zai"
    }

    fn supports_model(&self, model: &str) -> bool {
        is_zai_model(model)
    }
}

/// Check if a model string matches a Zai GLM model.
pub fn is_zai_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.starts_with("glm-") || ZAI_MODELS.contains(&lower.as_str())
}

/// Format a message for the Zai API (OpenAI-compatible format).
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
    fn test_is_zai_model() {
        assert!(is_zai_model("glm-5"));
        assert!(is_zai_model("glm-4.5"));
        assert!(is_zai_model("glm-5-turbo"));
        assert!(is_zai_model("GLM-5")); // case-insensitive
        assert!(!is_zai_model("gpt-4"));
        assert!(!is_zai_model("claude-sonnet-4-20250514"));
    }

    #[test]
    fn test_parse_zai_text_response() {
        let provider = ZaiProvider::new(
            "test".into(),
            "https://api.z.ai/api/paas/v4".into(),
        );
        let body = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "The weather is sunny."
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 20,
                "completion_tokens": 8
            }
        });
        let result = provider.parse_response(&body).unwrap();
        assert_eq!(result.message.text(), "The weather is sunny.");
        assert!(result.message.tool_calls.is_none());
    }

    #[test]
    fn test_parse_zai_tool_call_response() {
        let provider = ZaiProvider::new(
            "test".into(),
            "https://api.z.ai/api/paas/v4".into(),
        );
        let body = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [{
                        "id": "call_abc123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\":\"Beijing\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 30,
                "completion_tokens": 15
            }
        });
        let result = provider.parse_response(&body).unwrap();
        let tcs = result.message.tool_calls.as_ref().unwrap();
        assert_eq!(tcs.len(), 1);
        assert_eq!(tcs[0].name, "get_weather");
        assert_eq!(tcs[0].arguments["city"], "Beijing");
    }

    #[test]
    fn test_build_request_with_tools() {
        let provider = ZaiProvider::new(
            "test".into(),
            "https://api.z.ai/api/paas/v4".into(),
        );
        let messages = vec![Message::user("What's the weather?")];
        let tools = vec![ToolDefinition {
            name: "get_weather".into(),
            description: "Get weather for a city".into(),
            input_schema: json!({
                "type": "object",
                "properties": {"city": {"type": "string"}},
                "required": ["city"]
            }),
            server_name: None,
        }];

        let body = provider.build_request_body(&messages, &tools, "glm-5", 4096);
        assert_eq!(body["model"], "glm-5");
        assert_eq!(body["tool_choice"], "auto");
        assert_eq!(body["tools"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_format_tool_result_message() {
        let msg = Message::tool_result("call_abc", "{\"temp\": 25}");
        let formatted = format_message(&msg);
        assert_eq!(formatted["role"], "tool");
        assert_eq!(formatted["tool_call_id"], "call_abc");
    }
}
