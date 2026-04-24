//! AgentTool - Spawn subagents with isolated context and memory
//!
//! This tool allows the main agent to delegate work to subagents with their own:
//! - Isolated conversation history
//! - Optional custom system prompts
//! - Configurable tool access
//! - Independent memory stores
//!
//! Subagents can be resumed by agent_id, allowing for multi-step collaboration.

use crate::error::{FerroError, Result};
use crate::tool::{ToolFuture, ToolHandler};
use crate::types::{Message, ToolResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Built-in agent types with predefined behaviors
pub const BUILTIN_AGENT_TYPES: &[&str] = &[
    "planner",
    "coder",
    "reviewer",
    "debugger",
    "researcher",
    "generic",
];

/// Default system prompts for built-in agent types
fn get_default_prompt(agent_type: &str) -> Option<String> {
    match agent_type {
        "planner" => Some(
            "You are a planning specialist. Break down complex tasks into clear, \
             actionable steps. Identify dependencies and risks. Output structured plans."
                .to_string(),
        ),
        "coder" => Some(
            "You are a coding specialist. Write clean, well-documented code. \
                  Follow best practices and existing patterns in the codebase."
                .to_string(),
        ),
        "reviewer" => Some(
            "You are a code review specialist. Analyze code for bugs, security issues, \
             and design problems. Provide constructive feedback with specific examples."
                .to_string(),
        ),
        "debugger" => Some(
            "You are a debugging specialist. Investigate errors systematically. \
             Identify root causes and propose fixes. Test your hypotheses."
                .to_string(),
        ),
        "researcher" => Some(
            "You are a research specialist. Gather and synthesize information from \
             multiple sources. Provide comprehensive, well-organized findings."
                .to_string(),
        ),
        "generic" => Some(
            "You are a helpful AI assistant with broad capabilities. Work efficiently \
             and communicate clearly."
                .to_string(),
        ),
        _ => None,
    }
}

/// Definition of an agent's configuration and behavior
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    /// Agent type (built-in or custom)
    pub agent_type: String,
    /// Custom system prompt (overrides default for agent_type)
    pub system_prompt: Option<String>,
    /// Tools this agent can access (empty = all available tools)
    pub allowed_tools: Vec<String>,
    /// Whether this agent has isolated memory
    pub memory_isolation: bool,
}

impl AgentDefinition {
    /// Create a new agent definition
    pub fn new(agent_type: impl Into<String>) -> Self {
        Self {
            agent_type: agent_type.into(),
            system_prompt: None,
            allowed_tools: Vec::new(),
            memory_isolation: true,
        }
    }

    /// Set a custom system prompt
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set allowed tools (empty list = all tools)
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    /// Enable or disable memory isolation
    pub fn with_memory_isolation(mut self, isolated: bool) -> Self {
        self.memory_isolation = isolated;
        self
    }

    /// Get the effective system prompt
    pub fn get_prompt(&self) -> String {
        if let Some(custom) = &self.system_prompt {
            custom.clone()
        } else if let Some(default) = get_default_prompt(&self.agent_type) {
            default
        } else {
            format!(
                "You are a {} agent. Work efficiently and communicate clearly.",
                self.agent_type
            )
        }
    }
}

/// Isolated memory store for a subagent
#[derive(Debug, Clone)]
pub struct AgentMemory {
    /// Unique identifier for this agent instance
    pub agent_id: String,
    /// Conversation history
    pub history: Vec<Message>,
}

impl AgentMemory {
    /// Create new isolated agent memory
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            history: Vec::new(),
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
    }

    /// Get conversation history
    pub fn history(&self) -> &[Message] {
        &self.history
    }
}

/// Result from an agent execution
#[derive(Debug, Clone)]
pub struct AgentExecution {
    /// Agent ID for resumption
    pub agent_id: String,
    /// Final response text
    pub response: String,
    /// Number of tool calls made
    pub tool_calls: usize,
    /// Tokens used
    pub tokens_used: u64,
}

/// Registry of active subagents
#[derive(Debug)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentMemory>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Create a new agent
    pub fn create(&mut self, _def: &AgentDefinition) -> String {
        let agent_id = format!("agent_{}", uuid::Uuid::new_v4());
        let memory = AgentMemory::new(&agent_id);
        self.agents.insert(agent_id.clone(), memory);
        agent_id
    }

    /// Get an existing agent
    pub fn get(&self, agent_id: &str) -> Option<&AgentMemory> {
        self.agents.get(agent_id)
    }

    /// Get mutable reference to an existing agent
    pub fn get_mut(&mut self, agent_id: &str) -> Option<&mut AgentMemory> {
        self.agents.get_mut(agent_id)
    }

    /// Check if agent exists
    pub fn contains(&self, agent_id: &str) -> bool {
        self.agents.contains_key(agent_id)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// AgentTool handler for spawning subagents
///
/// Note: This is a simplified implementation that doesn't actually spawn
/// real subagents due to Rust ownership constraints. In a full implementation,
/// you'd need to restructure the agent loop to support shared state.
pub struct AgentTool {
    /// Registry of active subagents
    registry: Arc<Mutex<AgentRegistry>>,
}

impl AgentTool {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(AgentRegistry::new())),
        }
    }

    /// Simulate executing a subagent task
    /// In a full implementation, this would create a real AgentLoop
    async fn execute_agent(
        &self,
        def: &AgentDefinition,
        task: &str,
        agent_id: Option<&str>,
    ) -> Result<AgentExecution> {
        let agent_id = if let Some(id) = agent_id {
            if !self.registry.lock().await.contains(id) {
                return Err(FerroError::Tool(format!(
                    "Agent '{id}' not found. Cannot resume non-existent agent."
                )));
            }
            id.to_string()
        } else {
            self.registry.lock().await.create(def)
        };

        // Simulate agent processing
        // In real implementation, this would run AgentLoop
        let prompt = def.get_prompt();
        let simulated_response = format!(
            "[Agent {} executed task with prompt: '{}']\nTask: {}",
            agent_id,
            prompt.lines().next().unwrap_or(""),
            task
        );

        // Update agent memory
        {
            let mut registry = self.registry.lock().await;
            if let Some(memory) = registry.get_mut(&agent_id) {
                memory.add_message(Message::user(task));
                memory.add_message(Message::assistant(&simulated_response));
            }
        }

        Ok(AgentExecution {
            agent_id,
            response: simulated_response,
            tool_calls: 0,
            tokens_used: 100,
        })
    }
}

impl ToolHandler for AgentTool {
    fn call<'a>(&'a self, call_id: &'a str, arguments: &'a Value) -> ToolFuture<'a> {
        Box::pin(async move {
            // Parse agent_type
            let agent_type = arguments
                .get("agent_type")
                .and_then(|t| t.as_str())
                .unwrap_or("generic");

            // Validate agent type
            if !BUILTIN_AGENT_TYPES.contains(&agent_type) && agent_type != "generic" {
                return Ok(ToolResult {
                    call_id: call_id.to_string(),
                    content: format!(
                        "Error: Unknown agent type '{}'. Valid types: {}",
                        agent_type,
                        BUILTIN_AGENT_TYPES.join(", ")
                    ),
                    is_error: true,
                });
            }

            // Parse custom prompt
            let system_prompt = arguments.get("prompt").and_then(|p| p.as_str());

            // Parse agent_id for resumption
            let agent_id = arguments.get("agent_id").and_then(|i| i.as_str());

            // Parse task
            let task = arguments
                .get("task")
                .and_then(|t| t.as_str())
                .ok_or_else(|| FerroError::Tool("Missing 'task' argument".into()))?;

            // Build agent definition
            let mut def = AgentDefinition::new(agent_type);
            if let Some(prompt) = system_prompt {
                def = def.with_prompt(prompt);
            }

            // Execute agent
            let execution = self
                .execute_agent(&def, task, agent_id)
                .await
                .map_err(|e| FerroError::Tool(format!("Agent execution failed: {}", e)))?;

            // Format response
            let content = json!({
                "agent_id": execution.agent_id,
                "response": execution.response,
                "tool_calls": execution.tool_calls,
                "tokens_used": execution.tokens_used,
                "message": format!(
                    "Agent {} completed {} tool calls and used {} tokens.",
                    execution.agent_id, execution.tool_calls, execution.tokens_used
                )
            });

            Ok(ToolResult {
                call_id: call_id.to_string(),
                content: serde_json::to_string_pretty(&content).unwrap(),
                is_error: false,
            })
        })
    }
}

impl Default for AgentTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the AgentTool metadata for registration
pub fn agent_tool_meta() -> crate::types::ToolMeta {
    use crate::types::{Capability, ToolDefinition, ToolMeta, ToolSource};

    ToolMeta {
        definition: ToolDefinition {
            name: "agent".into(),
            description: "Spawn a subagent with isolated context to delegate work. Supports custom prompts and agent resumption.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_type": {
                        "type": "string",
                        "description": "Type of agent (planner, coder, reviewer, debugger, researcher, generic)",
                        "enum": ["planner", "coder", "reviewer", "debugger", "researcher", "generic"]
                    },
                    "prompt": {
                        "type": "string",
                        "description": "Custom system prompt for this agent (overrides agent_type default)"
                    },
                    "task": {
                        "type": "string",
                        "description": "The task to delegate to the subagent"
                    },
                    "agent_id": {
                        "type": "string",
                        "description": "Optional agent ID to resume a previous agent"
                    }
                },
                "required": ["task"]
            }),
            server_name: None,
        },
        required_capabilities: vec![Capability::ProcessExec], // Subagents can execute processes
        source: ToolSource::Builtin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ToolSource;

    #[test]
    fn test_builtin_agent_types() {
        assert!(BUILTIN_AGENT_TYPES.contains(&"planner"));
        assert!(BUILTIN_AGENT_TYPES.contains(&"coder"));
        assert!(BUILTIN_AGENT_TYPES.contains(&"generic"));
    }

    #[test]
    fn test_default_prompts() {
        let planner_prompt = get_default_prompt("planner");
        assert!(planner_prompt.is_some());
        assert!(planner_prompt.unwrap().contains("planning"));

        let generic_prompt = get_default_prompt("generic");
        assert!(generic_prompt.is_some());

        let invalid_prompt = get_default_prompt("nonexistent");
        assert!(invalid_prompt.is_none());
    }

    #[test]
    fn test_agent_definition_builder() {
        let def = AgentDefinition::new("coder")
            .with_prompt("Custom prompt")
            .with_tools(vec!["read_file".to_string(), "write_file".to_string()])
            .with_memory_isolation(false);

        assert_eq!(def.agent_type, "coder");
        assert_eq!(def.system_prompt, Some("Custom prompt".to_string()));
        assert_eq!(def.allowed_tools.len(), 2);
        assert!(!def.memory_isolation);
    }

    #[test]
    fn test_agent_prompt_fallback() {
        let custom_def = AgentDefinition::new("custom").with_prompt("My custom prompt");
        assert_eq!(custom_def.get_prompt(), "My custom prompt");

        let builtin_def = AgentDefinition::new("planner");
        assert!(builtin_def.get_prompt().contains("planning"));

        let fallback_def = AgentDefinition::new("unknown_type");
        assert!(fallback_def.get_prompt().contains("unknown_type"));
    }

    #[tokio::test]
    async fn test_agent_registry() {
        let mut registry = AgentRegistry::new();

        // Create agent
        let def = AgentDefinition::new("test");
        let id1 = registry.create(&def);
        let id2 = registry.create(&def);

        assert_ne!(id1, id2);
        assert!(registry.contains(&id1));
        assert!(registry.contains(&id2));

        // Get agent
        let agent = registry.get(&id1);
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().agent_id, id1);

        // Get non-existent agent
        assert!(registry.get("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_agent_memory() {
        let mut memory = AgentMemory::new("test_agent");

        assert_eq!(memory.agent_id, "test_agent");
        assert!(memory.history().is_empty());

        memory.add_message(Message::user("Hello"));
        assert_eq!(memory.history().len(), 1);
        assert_eq!(memory.history()[0].text(), "Hello");
    }

    #[tokio::test]
    async fn test_agent_spawn_default() {
        let tool = AgentTool::new();

        let args = json!({
            "agent_type": "generic",
            "task": "Say hello"
        });

        let result = tool.call("test-call", &args).await.unwrap();
        assert!(!result.is_error);

        let response: Value = serde_json::from_str(&result.content).unwrap();
        assert!(response["agent_id"].is_string());
        assert_eq!(response["tool_calls"], 0);
        assert!(response["tokens_used"].is_number());
    }

    #[tokio::test]
    async fn test_agent_invalid_type() {
        let tool = AgentTool::new();

        let args = json!({
            "agent_type": "nonexistent_type",
            "task": "Test task"
        });

        let result = tool.call("test-call", &args).await.unwrap();
        assert!(result.is_error);
        assert!(result.content.contains("Unknown agent type"));
    }

    #[tokio::test]
    async fn test_agent_missing_task() {
        let tool = AgentTool::new();

        let args = json!({
            "agent_type": "generic"
            // Missing "task" parameter
        });

        let result = tool.call("test-call", &args).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FerroError::Tool(_)));
    }

    #[tokio::test]
    async fn test_agent_memory_isolation() {
        let tool = AgentTool::new();

        // Spawn first agent
        let args1 = json!({
            "agent_type": "planner",
            "task": "Plan a project"
        });

        let result1 = tool.call("call-1", &args1).await.unwrap();
        let response1: Value = serde_json::from_str(&result1.content).unwrap();
        let agent_id_1 = response1["agent_id"].as_str().unwrap();

        // Spawn second agent
        let args2 = json!({
            "agent_type": "coder",
            "task": "Write some code"
        });

        let result2 = tool.call("call-2", &args2).await.unwrap();
        let response2: Value = serde_json::from_str(&result2.content).unwrap();
        let agent_id_2 = response2["agent_id"].as_str().unwrap();

        // Verify agents have different IDs
        assert_ne!(agent_id_1, agent_id_2);

        // Verify agents have isolated memory
        let registry = tool.registry.lock().await;
        let agent1 = registry.get(agent_id_1).unwrap();
        let agent2 = registry.get(agent_id_2).unwrap();

        assert_eq!(agent1.history().len(), 2);
        assert_eq!(agent2.history().len(), 2);

        // Verify each agent has its own task
        assert!(agent1.history()[0].text().contains("Plan a project"));
        assert!(agent2.history()[0].text().contains("Write some code"));
    }

    #[tokio::test]
    async fn test_agent_resumption() {
        let tool = AgentTool::new();

        // Create agent
        let args1 = json!({
            "agent_type": "researcher",
            "task": "Research topic A"
        });

        let result1 = tool.call("call-1", &args1).await.unwrap();
        let response1: Value = serde_json::from_str(&result1.content).unwrap();
        let agent_id = response1["agent_id"].as_str().unwrap();

        // Resume agent
        let args2 = json!({
            "agent_id": agent_id,
            "task": "Continue research on topic A"
        });

        let result2 = tool.call("call-2", &args2).await.unwrap();
        assert!(!result2.is_error);

        let response2: Value = serde_json::from_str(&result2.content).unwrap();
        assert_eq!(response2["agent_id"].as_str().unwrap(), agent_id);

        // Verify agent has accumulated history
        let registry = tool.registry.lock().await;
        let agent = registry.get(agent_id).unwrap();
        assert_eq!(agent.history().len(), 4); // 2 messages per task
    }

    #[test]
    fn test_agent_tool_meta() {
        let meta = agent_tool_meta();

        assert_eq!(meta.definition.name, "agent");
        assert!(meta.definition.description.contains("subagent"));
        assert_eq!(meta.definition.input_schema["required"], json!(["task"]));
        assert!(matches!(meta.source, ToolSource::Builtin));
    }
}
