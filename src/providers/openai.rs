use crate::error::{FerroError, Result};
use crate::provider::{BoxFuture, LlmProvider};
use crate::types::{
    Message, MessageContent, ProviderResponse, Role, TokenUsage, ToolCall, ToolDefinition,
};
use reqwest::Client;
use serde_json::{json, Value};

/// OpenAI-compatible provider. Works with OpenAI, OpenRouter, Ollama, and any
/// endpoint that implements the OpenAI chat completions API.
pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    #[allow(dead_code)]
    max_tokens: u32,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, max_tokens: u32) -> Self {
        Self {
            api_key,
            base_url,
            max_tokens,
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
        let formatted_messages: Vec<Value> = messages
            .iter()
            .map(|m| self.format_message(m))
            .collect();

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

    fn format_message(&self, msg: &Message) -> Value {
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

    fn parse_response(&self, body: &Value) -> Result<ProviderResponse> {
        let choice = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| FerroError::Provider("No choices in response".into()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| FerroError::Provider("No message in choice".into()))?;

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

impl LlmProvider for OpenAiProvider {
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
                .map_err(|e| FerroError::Provider(format!("HTTP request failed: {e}")))?;

            let status = response.status();
            let response_body: Value = response
                .json()
                .await
                .map_err(|e| FerroError::Provider(format!("Failed to parse response: {e}")))?;

            if !status.is_success() {
                let error_msg = response_body
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(FerroError::Provider(format!(
                    "OpenAI API error ({status}): {error_msg}"
                )));
            }

            self.parse_response(&response_body)
        })
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn supports_model(&self, model: &str) -> bool {
        // OpenAI-compatible endpoints accept any model string
        !model.starts_with("claude-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_response() {
        let provider = OpenAiProvider::new("test".into(), "https://api.openai.com/v1".into(), 4096);
        let body = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            }
        });
        let result = provider.parse_response(&body).unwrap();
        assert_eq!(result.message.text(), "Hello!");
    }

    #[test]
    fn test_parse_openai_tool_call() {
        let provider = OpenAiProvider::new("test".into(), "https://api.openai.com/v1".into(), 4096);
        let body = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_123",
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
}
