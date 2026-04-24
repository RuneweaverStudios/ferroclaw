//! Core agent loop: ReAct (Reason + Act) cycle.
//!
//! 1. Assemble context (system prompt + diet summaries + conversation + memory)
//! 2. Call LLM with tool definitions
//! 3. Parse tool_use blocks
//! 4. Execute tools (with capability checks)
//! 5. Append results, loop until text response or budget exhausted
use crate::agent::context::ContextManager;
use crate::config::Config;
use crate::error::{FerroError, Result};
use crate::mcp::client::McpClient;
use crate::mcp::diet::SkillSummary;
use crate::mcp::registry::build_diet_context;
use crate::provider::LlmProvider;
use crate::tool::ToolRegistry;
use crate::types::{CapabilitySet, Message, ToolCall, ToolDefinition, ToolSource};
use crate::websocket::{WsBroadcaster, WsEvent};

/// The core agent loop.
pub struct AgentLoop {
    provider: Box<dyn LlmProvider>,
    registry: ToolRegistry,
    mcp_client: Option<McpClient>,
    context: ContextManager,
    config: Config,
    capabilities: CapabilitySet,
    skill_summaries: Vec<SkillSummary>,
    /// Optional WebSocket broadcaster for real-time events.
    ws_broadcaster: Option<WsBroadcaster>,
    /// Agent ID for WebSocket events.
    agent_id: String,
}

/// Events emitted during the agent loop for streaming to the UI.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta(String),
    /// LLM round started (1-based iteration within this user turn).
    LlmRound {
        iteration: u32,
    },
    /// Model chose to invoke these tools before execution.
    ModelToolChoice {
        iteration: u32,
        names: Vec<String>,
    },
    /// Multiple tools in one assistant message (Hermes-style batch).
    ParallelToolBatch {
        count: usize,
    },
    ToolCallStart {
        id: String,
        name: String,
        arguments: String,
    },
    ToolResult {
        id: String,
        name: String,
        content: String,
        is_error: bool,
    },
    Done {
        text: String,
    },
    Error(String),
    TokenUsage {
        input: u64,
        output: u64,
        total_used: u64,
    },
}

impl AgentLoop {
    pub fn new(
        provider: Box<dyn LlmProvider>,
        registry: ToolRegistry,
        mcp_client: Option<McpClient>,
        config: Config,
        capabilities: CapabilitySet,
        skill_summaries: Vec<SkillSummary>,
    ) -> Self {
        let context = ContextManager::new(config.agent.token_budget);
        Self {
            provider,
            registry,
            mcp_client,
            context,
            config,
            capabilities,
            skill_summaries,
            ws_broadcaster: None,
            agent_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Set WebSocket broadcaster for real-time event broadcasting.
    pub fn with_ws_broadcaster(mut self, broadcaster: WsBroadcaster) -> Self {
        self.ws_broadcaster = Some(broadcaster);
        self
    }

    /// Set a custom agent ID for WebSocket events.
    pub fn with_agent_id(mut self, id: String) -> Self {
        self.agent_id = id;
        self
    }

    /// Reset per-run budget accounting (used by stateless HTTP runtime requests).
    pub fn reset_run_state(&mut self) {
        self.context.tokens_used = 0;
    }

    /// Broadcast a WebSocket event if broadcaster is configured.
    fn broadcast_event(&self, event: WsEvent) {
        if let Some(broadcaster) = &self.ws_broadcaster {
            if let Err(e) = broadcaster.broadcast(event) {
                tracing::warn!("Failed to broadcast WebSocket event: {}", e);
            }
        }
    }

    /// Run the agent loop for a single user message.
    /// Returns final assistant text response and all events.
    pub async fn run(
        &mut self,
        user_message: &str,
        history: &mut Vec<Message>,
    ) -> Result<(String, Vec<AgentEvent>)> {
        let mut events = Vec::new();
        let text = self
            .run_with_callback(user_message, history, |e: &AgentEvent| {
                events.push(e.clone())
            })
            .await?;
        Ok((text, events))
    }

    /// Run the agent loop, invoking `on_event` for every [`AgentEvent`] as it occurs (streaming).
    pub async fn run_with_callback<F>(
        &mut self,
        user_message: &str,
        history: &mut Vec<Message>,
        mut on_event: F,
    ) -> Result<String>
    where
        F: FnMut(&AgentEvent),
    {
        // Broadcast agent thinking state
        self.broadcast_event(WsEvent::agent_state(
            self.agent_id.clone(),
            crate::websocket::AgentState::Thinking,
        ));

        // Build system message with diet context
        let builtin_defs: Vec<ToolDefinition> = self
            .registry
            .all_meta()
            .iter()
            .filter(|m| matches!(m.source, ToolSource::Builtin))
            .map(|m| m.definition.clone())
            .collect();

        let diet_context = build_diet_context(&self.skill_summaries, &builtin_defs);
        let system_prompt = format!("{}\n\n{}", self.config.agent.system_prompt, diet_context);

        // Ensure system message is first
        if history.is_empty() || history[0].role != crate::types::Role::System {
            history.insert(0, Message::system(&system_prompt));
        } else {
            history[0] = Message::system(&system_prompt);
        }

        // Add user message
        history.push(Message::user(user_message));

        // Get all tool definitions for provider, plus a built-in-only fallback set
        let all_tools = self.registry.definitions();
        let builtin_tools: Vec<ToolDefinition> = self
            .registry
            .all_meta()
            .iter()
            .filter(|m| matches!(m.source, ToolSource::Builtin))
            .map(|m| m.definition.clone())
            .collect();

        let mut iteration = 0;
        let max_iterations = self.config.agent.max_iterations;

        loop {
            iteration += 1;
            if iteration > max_iterations {
                // Broadcast error state
                self.broadcast_event(WsEvent::agent_state(
                    self.agent_id.clone(),
                    crate::websocket::AgentState::Error,
                ));
                return Err(FerroError::MaxIterations(max_iterations));
            }

            // Check token budget
            if self.context.remaining() == 0 {
                // Broadcast error state
                self.broadcast_event(WsEvent::agent_state(
                    self.agent_id.clone(),
                    crate::websocket::AgentState::Error,
                ));
                return Err(FerroError::BudgetExhausted {
                    used: self.context.tokens_used,
                    limit: self.context.token_budget,
                });
            }

            // Prune context if needed
            self.context.prune_to_fit(history);

            let round = AgentEvent::LlmRound { iteration };
            on_event(&round);

            // Call LLM with fallback chain.
            // If we hit provider context overflow, retry once with a reduced toolset
            // (built-ins only, then no tools) and a smaller completion cap.
            let max_tokens = resolve_provider_max_tokens(&self.config, &self.config.agent.default_model);

            let response = match self
                .call_with_fallback(history, &all_tools, max_tokens, &mut on_event)
                .await
            {
                Ok(resp) => resp,
                Err(err) if is_context_overflow_error(&err) => {
                    let ev = AgentEvent::Error(
                        "Provider context overflow detected; retrying with compact toolset".into(),
                    );
                    on_event(&ev);

                    // Try built-in tools only first.
                    let reduced_max_tokens = max_tokens.min(1024);
                    match self
                        .call_with_fallback(
                            history,
                            &builtin_tools,
                            reduced_max_tokens,
                            &mut on_event,
                        )
                        .await
                    {
                        Ok(resp) => resp,
                        Err(err2) if is_context_overflow_error(&err2) => {
                            let ev = AgentEvent::Error(
                                "Context still too large; retrying with tools disabled".into(),
                            );
                            on_event(&ev);
                            self.call_with_fallback(history, &[], reduced_max_tokens, &mut on_event)
                                .await?
                        }
                        Err(other) => return Err(other),
                    }
                }
                Err(other) => return Err(other),
            };

            // Track usage
            if let Some(usage) = &response.usage {
                self.context
                    .record_usage(usage.input_tokens, usage.output_tokens);
                let u = AgentEvent::TokenUsage {
                    input: usage.input_tokens,
                    output: usage.output_tokens,
                    total_used: self.context.tokens_used,
                };
                on_event(&u);
            }

            // Check for tool calls
            let tool_calls = response.message.tool_calls.clone();
            history.push(response.message);

            if let Some(tool_calls) = tool_calls {
                if tool_calls.is_empty() {
                    // No tool calls — return text response
                    let text = history
                        .last()
                        .map(|m| m.text().to_string())
                        .unwrap_or_default();
                    let done = AgentEvent::Done { text: text.clone() };
                    on_event(&done);

                    // Broadcast agent idle state
                    self.broadcast_event(WsEvent::agent_state(
                        self.agent_id.clone(),
                        crate::websocket::AgentState::Idle,
                    ));

                    return Ok(text);
                }

                let names: Vec<String> = tool_calls.iter().map(|tc| tc.name.clone()).collect();
                let choice = AgentEvent::ModelToolChoice { iteration, names };
                on_event(&choice);

                if tool_calls.len() > 1 {
                    let batch = AgentEvent::ParallelToolBatch {
                        count: tool_calls.len(),
                    };
                    on_event(&batch);
                }

                // Execute each tool call
                for tc in &tool_calls {
                    let args_str =
                        serde_json::to_string(&tc.arguments).unwrap_or_else(|_| "{}".into());
                    let start = AgentEvent::ToolCallStart {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: args_str,
                    };
                    on_event(&start);

                    // Broadcast tool start event
                    self.broadcast_event(WsEvent::tool_start(
                        tc.id.clone(),
                        tc.name.clone(),
                        tc.arguments.clone(),
                    ));

                    // Broadcast agent executing state
                    self.broadcast_event(WsEvent::agent_state(
                        self.agent_id.clone(),
                        crate::websocket::AgentState::Executing,
                    ));

                    let result = self.execute_tool_call(tc).await;

                    let (content, is_error) = match result {
                        Ok(tr) => (tr.content, tr.is_error),
                        Err(e) => (format!("Error: {e}"), true),
                    };

                    // Broadcast tool result as output chunk
                    self.broadcast_event(WsEvent::tool_chunk(tc.id.clone(), content.clone(), true));

                    // Broadcast tool completion state
                    self.broadcast_event(WsEvent::tool_update(
                        tc.id.clone(),
                        if is_error {
                            crate::websocket::ToolState::Failed
                        } else {
                            crate::websocket::ToolState::Completed
                        },
                    ));

                    let tres = AgentEvent::ToolResult {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        content: content.clone(),
                        is_error,
                    };
                    on_event(&tres);

                    history.push(Message::tool_result(&tc.id, &content));
                }
            } else {
                // No tool calls — text response
                let text = history
                    .last()
                    .map(|m| m.text().to_string())
                    .unwrap_or_default();
                let done = AgentEvent::Done { text: text.clone() };
                on_event(&done);

                // Broadcast agent idle state
                self.broadcast_event(WsEvent::agent_state(
                    self.agent_id.clone(),
                    crate::websocket::AgentState::Idle,
                ));

                return Ok(text);
            }
        }
    }

    /// Try primary model, then each fallback in order.
    /// Returns first successful response.
    async fn call_with_fallback<F>(
        &self,
        history: &[Message],
        tools: &[ToolDefinition],
        max_tokens: u32,
        on_event: &mut F,
    ) -> Result<crate::types::ProviderResponse>
    where
        F: FnMut(&AgentEvent),
    {
        let primary = &self.config.agent.default_model;
        let fallbacks = &self.config.agent.fallback_models;

        // Try primary model
        match self
            .provider
            .complete(history, tools, primary, max_tokens)
            .await
        {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if fallbacks.is_empty() {
                    return Err(e);
                }
                tracing::warn!("Primary model '{}' failed: {}", primary, e);
                let ev =
                    AgentEvent::Error(format!("Model '{}' failed, trying fallbacks...", primary));
                on_event(&ev);
            }
        }

        // Try each fallback
        for (i, fallback_model) in fallbacks.iter().enumerate() {
            tracing::info!(
                "Trying fallback model {}/{}: {}",
                i + 1,
                fallbacks.len(),
                fallback_model
            );
            match self
                .provider
                .complete(history, tools, fallback_model, max_tokens)
                .await
            {
                Ok(resp) => {
                    tracing::info!("Fallback model '{}' succeeded", fallback_model);
                    let ev = AgentEvent::Error(format!("Using fallback model: {}", fallback_model));
                    on_event(&ev);
                    return Ok(resp);
                }
                Err(e) => {
                    tracing::warn!("Fallback model '{}' failed: {}", fallback_model, e);
                }
            }
        }

        Err(FerroError::Provider(format!(
            "All models failed. Tried: {}, {}",
            primary,
            fallbacks.join(", ")
        )))
    }

    async fn execute_tool_call(&self, tc: &ToolCall) -> Result<crate::types::ToolResult> {
        // Check if this is an MCP tool
        if let Some(meta) = self.registry.get_meta(&tc.name) {
            if let ToolSource::Mcp { server } = &meta.source {
                // Route through MCP client
                if let Some(mcp) = &self.mcp_client {
                    let diet_response = mcp.execute_tool(server, &tc.name, &tc.arguments).await?;
                    return Ok(crate::types::ToolResult {
                        call_id: tc.id.clone(),
                        content: diet_response.content,
                        is_error: false,
                    });
                }
            }
        }

        // Execute through registry (built-in tools)
        self.registry
            .execute(&tc.name, &tc.id, &tc.arguments, &self.capabilities)
            .await
    }

    // Helper methods for orchestration
    /// Get a reference to tool registry
    pub fn get_tool_registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Get current token budget
    pub fn get_token_budget(&self) -> u64 {
        self.context.token_budget
    }
}

fn resolve_provider_max_tokens(config: &Config, model: &str) -> u32 {
    if model.contains('/') {
        return config
            .providers
            .openrouter
            .as_ref()
            .map(|o| o.max_tokens)
            .unwrap_or(8192);
    }

    if model.starts_with("claude-") {
        return config
            .providers
            .anthropic
            .as_ref()
            .map(|a| a.max_tokens)
            .unwrap_or(8192);
    }

    if model.starts_with("glm-") {
        return 8192;
    }

    config
        .providers
        .openai
        .as_ref()
        .map(|o| o.max_tokens)
        .unwrap_or(8192)
}

fn is_context_overflow_error(err: &FerroError) -> bool {
    let FerroError::Provider(msg) = err else {
        return false;
    };

    let m = msg.to_ascii_lowercase();
    (m.contains("context") && (m.contains("length") || m.contains("window") || m.contains("token")))
        || m.contains("maximum context")
        || m.contains("requested") && m.contains("max")
        || m.contains("too long")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AnthropicConfig, OpenAiConfig, OpenRouterConfig, ProvidersConfig};

    #[test]
    fn test_agent_event_variants() {
        let event = AgentEvent::TextDelta("hello".into());
        match event {
            AgentEvent::TextDelta(t) => assert_eq!(t, "hello"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_detects_context_overflow_error_strings() {
        let e1 = FerroError::Provider(
            "OpenRouter API error (400): This model's maximum context length is 400000 tokens. However, your messages resulted in 678640 tokens".into(),
        );
        assert!(is_context_overflow_error(&e1));

        let e2 = FerroError::Provider("timeout after 240000ms".into());
        assert!(!is_context_overflow_error(&e2));
    }

    #[test]
    fn test_resolve_provider_max_tokens_for_openrouter_model() {
        let mut cfg = Config::default();
        cfg.providers = ProvidersConfig {
            anthropic: Some(AnthropicConfig {
                api_key_env: "ANTHROPIC_API_KEY".into(),
                base_url: "https://api.anthropic.com".into(),
                max_tokens: 1111,
            }),
            openai: Some(OpenAiConfig {
                api_key_env: "OPENAI_API_KEY".into(),
                base_url: "https://api.openai.com/v1".into(),
                max_tokens: 2222,
            }),
            zai: None,
            openrouter: Some(OpenRouterConfig {
                api_key_env: "OPENROUTER_API_KEY".into(),
                base_url: "https://openrouter.ai/api/v1".into(),
                site_url: None,
                site_name: None,
                max_tokens: 3333,
            }),
        };

        assert_eq!(
            resolve_provider_max_tokens(&cfg, "openai/gpt-5.3-codex"),
            3333
        );
        assert_eq!(
            resolve_provider_max_tokens(&cfg, "claude-sonnet-4-20250514"),
            1111
        );
        assert_eq!(resolve_provider_max_tokens(&cfg, "gpt-4.1"), 2222);
    }
}
