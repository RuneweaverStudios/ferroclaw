//! Integration tests for the provider subsystem:
//! model routing, request body formatting, response parsing edge cases.

use ferroclaw::config::Config;
use ferroclaw::providers;
use ferroclaw::providers::openrouter::is_openrouter_model;
use ferroclaw::providers::zai::is_zai_model;
use ferroclaw::types::{Message, ToolDefinition, TokenUsage};

// ── Model Routing ──────────────────────────────────────────────────

#[test]
fn test_zai_model_detection() {
    // Positive cases
    assert!(is_zai_model("glm-5"));
    assert!(is_zai_model("glm-5-turbo"));
    assert!(is_zai_model("glm-4.5"));
    assert!(is_zai_model("glm-4.6"));
    assert!(is_zai_model("glm-4.7"));
    assert!(is_zai_model("glm-4.5v"));
    assert!(is_zai_model("glm-4.6v"));
    assert!(is_zai_model("glm-4-32b-0414-128k"));
    assert!(is_zai_model("GLM-5")); // case insensitive

    // Negative cases
    assert!(!is_zai_model("gpt-4"));
    assert!(!is_zai_model("claude-sonnet-4-20250514"));
    assert!(!is_zai_model("llama-3"));
}

#[test]
fn test_openrouter_model_detection() {
    // Positive cases
    assert!(is_openrouter_model("openai/gpt-4o"));
    assert!(is_openrouter_model("anthropic/claude-sonnet-4"));
    assert!(is_openrouter_model("meta-llama/llama-3.1-70b"));
    assert!(is_openrouter_model("google/gemini-2.0-flash"));

    // Negative cases
    assert!(!is_openrouter_model("gpt-4o"));
    assert!(!is_openrouter_model("claude-sonnet-4-20250514"));
    assert!(!is_openrouter_model("glm-5"));
}

#[test]
fn test_routing_priority_zai_first() {
    // glm-5 should route to Zai, not OpenRouter (even though it doesn't contain '/')
    let config = Config::default();
    let result = providers::resolve_provider("glm-5", &config);
    // Should fail because Zai is not configured, confirming it tried Zai first
    match result {
        Ok(_) => panic!("Expected error for unconfigured Zai provider"),
        Err(e) => assert!(e.to_string().contains("Zai"), "Expected Zai error, got: {e}"),
    }
}

#[test]
fn test_routing_priority_openrouter_second() {
    let config = Config::default();
    let result = providers::resolve_provider("openai/gpt-4o", &config);
    match result {
        Ok(_) => panic!("Expected error for unconfigured OpenRouter provider"),
        Err(e) => assert!(e.to_string().contains("OpenRouter"), "Expected OpenRouter error, got: {e}"),
    }
}

#[test]
fn test_routing_priority_anthropic_third() {
    let config = Config::default();
    let result = providers::resolve_provider("claude-opus-4-20250514", &config);
    match result {
        Ok(p) => assert_eq!(p.name(), "anthropic"),
        Err(e) => assert!(e.to_string().contains("ANTHROPIC_API_KEY")),
    }
}

#[test]
fn test_routing_fallback_no_provider() {
    // A model that doesn't match any pattern and no OpenAI configured
    let mut config = Config::default();
    config.providers.openai = None;
    let result = providers::resolve_provider("random-model-xyz", &config);
    match result {
        Ok(_) => panic!("Expected error for unknown model with no fallback"),
        Err(e) => assert!(e.to_string().contains("No provider configured"), "Expected fallback error, got: {e}"),
    }
}

// ── Token Usage ────────────────────────────────────────────────────

#[test]
fn test_token_usage_total() {
    let usage = TokenUsage {
        input_tokens: 100,
        output_tokens: 50,
    };
    assert_eq!(usage.total(), 150);
}

#[test]
fn test_token_usage_zero() {
    let usage = TokenUsage {
        input_tokens: 0,
        output_tokens: 0,
    };
    assert_eq!(usage.total(), 0);
}

// ── Message Formatting ──────────────────────────────────────────────

#[test]
fn test_system_message_text() {
    let msg = Message::system("You are helpful.");
    assert_eq!(msg.text(), "You are helpful.");
    assert!(msg.tool_calls.is_none());
    assert!(msg.tool_call_id.is_none());
}

#[test]
fn test_tool_result_message_fields() {
    let msg = Message::tool_result("tc_123", "result data");
    assert_eq!(msg.text(), "result data");
    assert_eq!(msg.tool_call_id.as_deref(), Some("tc_123"));
    assert_eq!(msg.role, ferroclaw::types::Role::Tool);
}

#[test]
fn test_assistant_with_tool_calls_empty_text() {
    let msg = Message::assistant_with_tool_calls(vec![ferroclaw::types::ToolCall {
        id: "tc_1".into(),
        name: "read_file".into(),
        arguments: serde_json::json!({"path": "/tmp"}),
    }]);
    assert_eq!(msg.text(), ""); // Empty text when tool calls present
    assert!(msg.tool_calls.is_some());
}

// ── Tool Definition ────────────────────────────────────────────────

#[test]
fn test_tool_definition_compact_signature_no_params() {
    let tool = ToolDefinition {
        name: "get_time".into(),
        description: "Get current time".into(),
        input_schema: serde_json::json!({"type": "object", "properties": {}}),
        server_name: None,
    };
    let sig = tool.compact_signature();
    assert_eq!(sig, "get_time()");
}

#[test]
fn test_tool_definition_required_params_empty() {
    let tool = ToolDefinition {
        name: "test".into(),
        description: "Test".into(),
        input_schema: serde_json::json!({"type": "object", "properties": {"a": {"type": "string"}}}),
        server_name: None,
    };
    let required = tool.required_params();
    assert!(required.is_empty());
}
