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

/// The core agent loop.
pub struct AgentLoop {
    provider: Box<dyn LlmProvider>,
    registry: ToolRegistry,
    mcp_client: Option<McpClient>,
    context: ContextManager,
    config: Config,
    capabilities: CapabilitySet,
    skill_summaries: Vec<SkillSummary>,
}

/// Events emitted during the agent loop for streaming to the UI.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta(String),
    ToolCallStart { id: String, name: String },
    ToolResult { id: String, content: String, is_error: bool },
    Done { text: String },
    Error(String),
    TokenUsage { input: u64, output: u64, total_used: u64 },
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
        }
    }

    /// Run the agent loop for a single user message.
    /// Returns the final assistant text response and all events.
    pub async fn run(
        &mut self,
        user_message: &str,
        history: &mut Vec<Message>,
    ) -> Result<(String, Vec<AgentEvent>)> {
        let mut events = Vec::new();

        // Build system message with diet context
        let builtin_defs: Vec<ToolDefinition> = self
            .registry
            .all_meta()
            .iter()
            .filter(|m| matches!(m.source, ToolSource::Builtin))
            .map(|m| m.definition.clone())
            .collect();

        let diet_context = build_diet_context(&self.skill_summaries, &builtin_defs);
        let system_prompt = format!(
            "{}\n\n{}",
            self.config.agent.system_prompt, diet_context
        );

        // Ensure system message is first
        if history.is_empty() || history[0].role != crate::types::Role::System {
            history.insert(0, Message::system(&system_prompt));
        } else {
            history[0] = Message::system(&system_prompt);
        }

        // Add user message
        history.push(Message::user(user_message));

        // Get all tool definitions for the provider
        let all_tools = self.registry.definitions();

        let mut iteration = 0;
        let max_iterations = self.config.agent.max_iterations;

        loop {
            iteration += 1;
            if iteration > max_iterations {
                return Err(FerroError::MaxIterations(max_iterations));
            }

            // Check token budget
            if self.context.remaining() == 0 {
                return Err(FerroError::BudgetExhausted {
                    used: self.context.tokens_used,
                    limit: self.context.token_budget,
                });
            }

            // Prune context if needed
            self.context.prune_to_fit(history);

            // Call LLM with fallback chain
            let max_tokens = self
                .config
                .providers
                .anthropic
                .as_ref()
                .map(|a| a.max_tokens)
                .or_else(|| {
                    self.config
                        .providers
                        .openrouter
                        .as_ref()
                        .map(|o| o.max_tokens)
                })
                .unwrap_or(8192);

            let response = self
                .call_with_fallback(history, &all_tools, max_tokens, &mut events)
                .await?;

            // Track usage
            if let Some(usage) = &response.usage {
                self.context
                    .record_usage(usage.input_tokens, usage.output_tokens);
                events.push(AgentEvent::TokenUsage {
                    input: usage.input_tokens,
                    output: usage.output_tokens,
                    total_used: self.context.tokens_used,
                });
            }

            // Check for tool calls
            let tool_calls = response.message.tool_calls.clone();
            history.push(response.message);

            if let Some(tool_calls) = tool_calls {
                if tool_calls.is_empty() {
                    // No tool calls — return text response
                    let text = history.last().map(|m| m.text().to_string()).unwrap_or_default();
                    events.push(AgentEvent::Done { text: text.clone() });
                    return Ok((text, events));
                }

                // Execute each tool call
                for tc in &tool_calls {
                    events.push(AgentEvent::ToolCallStart {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                    });

                    let result = self.execute_tool_call(tc).await;

                    let (content, is_error) = match result {
                        Ok(tr) => (tr.content, tr.is_error),
                        Err(e) => (format!("Error: {e}"), true),
                    };

                    events.push(AgentEvent::ToolResult {
                        id: tc.id.clone(),
                        content: content.clone(),
                        is_error,
                    });

                    history.push(Message::tool_result(&tc.id, &content));
                }
            } else {
                // No tool calls — text response
                let text = history.last().map(|m| m.text().to_string()).unwrap_or_default();
                events.push(AgentEvent::Done { text: text.clone() });
                return Ok((text, events));
            }
        }
    }

    /// Try the primary model, then each fallback in order.
    /// Returns the first successful response.
    async fn call_with_fallback(
        &self,
        history: &[Message],
        tools: &[ToolDefinition],
        max_tokens: u32,
        events: &mut Vec<AgentEvent>,
    ) -> Result<crate::types::ProviderResponse> {
        let primary = &self.config.agent.default_model;
        let fallbacks = &self.config.agent.fallback_models;

        // Try primary model
        match self.provider.complete(history, tools, primary, max_tokens).await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if fallbacks.is_empty() {
                    return Err(e);
                }
                tracing::warn!("Primary model '{primary}' failed: {e}");
                events.push(AgentEvent::Error(format!(
                    "Model '{primary}' failed, trying fallbacks..."
                )));
            }
        }

        // Try each fallback
        for (i, fallback_model) in fallbacks.iter().enumerate() {
            tracing::info!("Trying fallback model {}/{}: {fallback_model}", i + 1, fallbacks.len());
            match self.provider.complete(history, tools, fallback_model, max_tokens).await {
                Ok(resp) => {
                    tracing::info!("Fallback model '{fallback_model}' succeeded");
                    events.push(AgentEvent::Error(format!(
                        "Using fallback model: {fallback_model}"
                    )));
                    return Ok(resp);
                }
                Err(e) => {
                    tracing::warn!("Fallback model '{fallback_model}' failed: {e}");
                }
            }
        }

        Err(FerroError::Provider(format!(
            "All models failed. Tried: {primary}, {}",
            fallbacks.join(", ")
        )))
    }

    async fn execute_tool_call(&self, tc: &ToolCall) -> Result<crate::types::ToolResult> {
        // Check if this is an MCP tool
        if let Some(meta) = self.registry.get_meta(&tc.name) {
            if let ToolSource::Mcp { server } = &meta.source {
                // Route through MCP client
                if let Some(mcp) = &self.mcp_client {
                    let diet_response = mcp
                        .execute_tool(server, &tc.name, &tc.arguments)
                        .await?;
                    return Ok(crate::types::ToolResult {
                        call_id: tc.id.clone(),
                        content: diet_response.content,
                        is_error: false,
                    });
                }
            }
        }

        // Execute through the registry (built-in tools)
        self.registry
            .execute(&tc.name, &tc.id, &tc.arguments, &self.capabilities)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_event_variants() {
        let event = AgentEvent::TextDelta("hello".into());
        match event {
            AgentEvent::TextDelta(t) => assert_eq!(t, "hello"),
            _ => panic!("Wrong variant"),
        }
    }
}
