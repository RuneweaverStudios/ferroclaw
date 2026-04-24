# Agent Orchestration - Multi-Agent Coordination

## Overview

The orchestration module provides infrastructure for managing multiple agents, enabling advanced multi-agent patterns like task delegation, agent-to-agent communication, and hierarchical workflows.

**Key Components:**
- `SubagentConfig` - Configuration for spawning subagents
- `AgentMessageBus` - Message passing between agents
- `Orchestrator` - Coordinate multiple agents
- `AgentExecution` - Results from agent execution

## Core Concepts

### SubagentConfig

Configuration for spawning a new agent with specific capabilities:

```rust
use ferroclaw::agent::orchestration::SubagentConfig;

let config = SubagentConfig::new("agent_1".to_string(), "coder".to_string())
    .with_prompt("You are a Rust expert")
    .with_tools(vec!["read_file".to_string(), "write_file".to_string()])
    .with_memory_isolation(false)
    .with_token_budget(50000)
    .with_max_iterations(100);
```

**Fields:**
- `agent_id` - Unique identifier
- `agent_type` - Type of agent (planner, coder, reviewer, etc.)
- `system_prompt` - Custom prompt (optional)
- `allowed_tools` - Tool access restriction
- `memory_isolation` - Whether memory is isolated or shared
- `token_budget` - Token limit (0 = use parent's budget)
- `max_iterations` - Max ReAct iterations

### AgentMessageBus

Message passing infrastructure for agent communication:

```rust
use ferroclaw::agent::orchestration::{AgentMessage, AgentMessageBus};

let mut bus = AgentMessageBus::new();
bus.register("agent_1".to_string());
bus.register("agent_2".to_string());

// Send direct message
let msg = AgentMessage::new("agent_1", "agent_2", "Hello");
bus.send(msg).unwrap();

// Receive messages
let messages = bus.receive("agent_2");

// Broadcast to all agents
let broadcast = AgentMessage::new("agent_1", "", "Announcement");
bus.send(broadcast).unwrap();
```

**Features:**
- Direct agent-to-agent messaging
- Broadcast to all agents (except sender)
- Message queuing per agent
- Thread-safe access

### Orchestrator

Main coordinator for managing multiple agents:

```rust
use ferroclaw::agent::orchestration::{Orchestrator, SubagentConfig};
use ferroclaw::agent::AgentLoop;

// Create orchestrator from parent agent
let mut orchestrator = Orchestrator::new(parent_agent);

// Spawn child agents
let config = SubagentConfig::new("coder_1".to_string(), "coder".to_string())
    .with_tools(vec!["read_file".to_string(), "write_file".to_string()]);
orchestrator.spawn_child(config).unwrap();

// Execute tasks on children
let result = orchestrator.execute_child("coder_1", "Write a function").await?;

// Send messages between agents
orchestrator.send_message(AgentMessage::new(
    "agent_1",
    "agent_2",
    "Status update"
)).unwrap();
```

## Multi-Agent Patterns

### 1. Planner-Executor Pattern

```rust
// 1. Spawn planner agent
let planner_config = SubagentConfig::new("planner".to_string(), "planner".to_string());
orchestrator.spawn_child(planner_config).unwrap();

// 2. Get plan from planner
let plan_result = orchestrator.execute_child(
    "planner",
    "Create a plan for implementing OAuth2"
).await?;

// 3. Parse plan and spawn executors
for task in parse_tasks(&plan_result.response) {
    let executor_config = SubagentConfig::new(
        format!("executor_{}", task.id),
        "coder".to_string()
    );
    orchestrator.spawn_child(executor_config).unwrap();
}

// 4. Execute tasks in parallel
for task in tasks {
    orchestrator.execute_child(
        &format!("executor_{}", task.id),
        &task.description
    ).await?;
}
```

### 2. Reviewer-Coder Loop

```rust
let coder_id = "coder_main".to_string();
let reviewer_id = "reviewer_main".to_string();

// Spawn coder
let coder_config = SubagentConfig::new(coder_id.clone(), "coder".to_string());
orchestrator.spawn_child(coder_config).unwrap();

// Spawn reviewer
let reviewer_config = SubagentConfig::new(reviewer_id.clone(), "reviewer".to_string());
orchestrator.spawn_child(reviewer_config).unwrap();

// Coder writes code
let code_result = orchestrator.execute_child(
    &coder_id,
    "Write authentication function"
).await?;

// Send code to reviewer
orchestrator.send_message(AgentMessage::new(
    &coder_id,
    &reviewer_id,
    &code_result.response
)).unwrap();

// Reviewer reviews
let review_result = orchestrator.execute_child(
    &reviewer_id,
    "Review the code you received"
).await?;

// If issues found, send feedback to coder
if has_issues(&review_result.response) {
    orchestrator.send_message(AgentMessage::new(
        &reviewer_id,
        &coder_id,
        "Fix these issues: ..."
    )).unwrap();
}
```

### 3. Parallel Research

```rust
// Spawn multiple researchers
let topics = vec![
    ("research_1", "OAuth2 providers"),
    ("research_2", "Security best practices"),
    ("research_3", "Database design"),
];

for (agent_id, topic) in topics {
    let config = SubagentConfig::new(
        agent_id.to_string(),
        "researcher".to_string()
    ).with_tools(vec!["web_fetch".to_string()]);
    orchestrator.spawn_child(config).unwrap();

    // Spawn tasks in parallel
    tokio::spawn(async move {
        orchestrator.execute_child(agent_id, topic).await
    });
}

// Collect results later
let results = orchestrator.collect_results();
```

### 4. Hierarchical Delegation

```rust
// Main planner spawns sub-planners for different areas
let areas = vec!["frontend", "backend", "database"];

for area in areas {
    let sub_planner_id = format!("planner_{}", area);
    let config = SubagentConfig::new(
        sub_planner_id.clone(),
        "planner".to_string()
    );
    orchestrator.spawn_child(config).unwrap();

    // Each sub-planner spawns their own coders
    let plan_result = orchestrator.execute_child(
        &sub_planner_id,
        &format!("Plan {} implementation", area)
    ).await?;

    // Parse plan and spawn coders
    for task in parse_tasks(&plan_result.response) {
        let coder_id = format!("coder_{}_{}", area, task.id);
        let coder_config = SubagentConfig::new(coder_id.clone(), "coder".to_string());
        orchestrator.spawn_child(coder_config).unwrap();
    }
}
```

## Tool Filtering

Control which tools subagents can access:

```rust
// Coder agent - only file operations
let coder_config = SubagentConfig::new("coder".to_string(), "coder".to_string())
    .with_tools(vec![
        "read_file".to_string(),
        "write_file".to_string(),
        "file_edit".to_string(),
    ]);

// Researcher agent - only web access
let researcher_config = SubagentConfig::new("researcher".to_string(), "researcher".to_string())
    .with_tools(vec![
        "web_fetch".to_string(),
        "memory_store".to_string(),
    ]);

// Reviewer agent - read-only access
let reviewer_config = SubagentConfig::new("reviewer".to_string(), "reviewer".to_string())
    .with_tools(vec![
        "read_file".to_string(),
        "grep".to_string(),
        "memory_search".to_string(),
    ]);
```

**Security Benefits:**
- Principle of least privilege
- Prevent accidental modifications
- Auditability of tool access

## Memory Management

### Isolated Memory

Each agent has its own memory:

```rust
let config = SubagentConfig::new("agent_1".to_string(), "coder".to_string())
    .with_memory_isolation(true); // Default

// agent_1's memory is completely separate
```

**Use Cases:**
- Parallel independent tasks
- Confidential information handling
- Testing alternative solutions

### Shared Memory

Agents can share memory:

```rust
let config = SubagentConfig::new("agent_2".to_string(), "researcher".to_string())
    .with_memory_isolation(false); // Share with parent

// agent_2 can access parent's memory
```

**Use Cases:**
- Collaborative workflows
- Knowledge sharing
- Task handoffs

## Message Passing

### Direct Messages

```rust
let msg = AgentMessage::new(
    "agent_1",
    "agent_2",
    "Here's the code I wrote"
);
orchestrator.send_message(msg)?;
```

### Broadcasts

```rust
let broadcast = AgentMessage::new(
    "coordinator",
    "", // Empty string = broadcast
    "All agents, pause current work"
);
orchestrator.send_message(broadcast)?;
```

### Receiving Messages

```rust
// Agent checks for messages
let messages = bus.receive("agent_2");
for msg in messages {
    println!("From {}: {}", msg.from_agent_id, msg.content);
}
```

## Best Practices

### 1. Use Appropriate Agent Types

```rust
// Good: Specific agent type
SubagentConfig::new("code_reviewer".to_string(), "reviewer".to_string())

// Bad: Generic agent for specialized task
SubagentConfig::new("code_reviewer".to_string(), "generic".to_string())
```

### 2. Limit Tool Access

```rust
// Good: Minimal tool set
.with_tools(vec!["read_file".to_string()])

// Bad: All tools
.with_tools(vec![]) // Or omit parameter
```

### 3. Use Memory Isolation When Appropriate

```rust
// Good: Isolate for parallel tasks
.with_memory_isolation(true)

// Good: Share for collaboration
.with_memory_isolation(false)
```

### 4. Set Reasonable Token Budgets

```rust
// Good: Specific budget per agent
.with_token_budget(30000)

// Bad: Unbounded (0 = use parent's budget)
.with_token_budget(0)
```

### 5. Clean Up After Execution

```rust
// After task completion
let results = orchestrator.collect_results();

// Review results
for result in results {
    if !result.response.is_empty() {
        println!("{}: {}", result.agent_id, result.response);
    }
}

// Agents can be reused or stopped
```

## Error Handling

```rust
use ferroclaw::error::FerroError;

match orchestrator.execute_child("agent_1", "Task description").await {
    Ok(execution) => {
        println!("Agent completed: {}", execution.response);
    }
    Err(FerroError::Tool(msg)) => {
        println!("Tool error: {}", msg);
    }
    Err(e) => {
        println!("Other error: {}", e);
    }
}
```

## Limitations

### Current Implementation Status

**Phase 1 (Complete):**
- ✅ SubagentConfig structure
- ✅ AgentMessageBus implementation
- ✅ Orchestrator framework
- ✅ Tool filtering infrastructure
- ✅ Test coverage

**Phase 2 (In Progress):**
- ⏳ Real AgentLoop integration
- ⏳ Memory store integration
- ⏳ Full tool filtering enforcement
- ⏳ Per-agent context management

**Phase 3 (Planned):**
- 📋 Agent-to-agent protocol
- 📋 Collaboration patterns
- 📋 Hierarchical workflows

## Examples

See the examples directory for complete working examples:
- `examples/orchestration/planner_executor.rs`
- `examples/orchestration/reviewer_coder_loop.rs`
- `examples/orchestration/parallel_research.rs`

## API Reference

### SubagentConfig

```rust
pub struct SubagentConfig {
    pub agent_id: String,
    pub agent_type: String,
    pub system_prompt: Option<String>,
    pub allowed_tools: Vec<String>,
    pub memory_isolation: bool,
    pub token_budget: u64,
    pub max_iterations: Option<u32>,
}

impl SubagentConfig {
    pub fn new(agent_id: String, agent_type: String) -> Self;
    pub fn with_prompt(self, prompt: impl Into<String>) -> Self;
    pub fn with_tools(self, tools: Vec<String>) -> Self;
    pub fn with_memory_isolation(self, isolated: bool) -> Self;
    pub fn with_token_budget(self, budget: u64) -> Self;
    pub fn with_max_iterations(self, iterations: u32) -> Self;
}
```

### AgentMessage

```rust
pub struct AgentMessage {
    pub from_agent_id: String,
    pub to_agent_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AgentMessage {
    pub fn new(from: impl Into<String>, to: impl Into<String>, content: impl Into<String>) -> Self;
}
```

### AgentMessageBus

```rust
pub struct AgentMessageBus;

impl AgentMessageBus {
    pub fn new() -> Self;
    pub fn register(&mut self, agent_id: String);
    pub fn send(&mut self, msg: AgentMessage) -> Result<()>;
    pub fn receive(&mut self, agent_id: &str) -> Vec<AgentMessage>;
    pub fn has_messages(&self, agent_id: &str) -> bool;
    pub fn message_count(&self, agent_id: &str) -> usize;
}
```

### Orchestrator

```rust
pub struct Orchestrator;

impl Orchestrator {
    pub fn new(parent: AgentLoop) -> Self;
    pub fn spawn_child(&mut self, config: SubagentConfig) -> Result<()>;
    pub fn execute_child(&mut self, agent_id: &str, task: &str) -> Result<AgentExecution>;
    pub fn send_message(&mut self, msg: AgentMessage) -> Result<()>;
    pub fn collect_results(&self) -> Vec<AgentExecution>;
    pub fn parent(&self) -> &AgentLoop;
    pub fn message_bus(&self) -> &AgentMessageBus;
    pub fn has_agent(&self, agent_id: &str) -> bool;
}
```

### AgentExecution

```rust
pub struct AgentExecution {
    pub agent_id: String,
    pub response: String,
    pub tool_calls: usize,
    pub tokens_used: u64,
    pub messages_received: usize,
    pub messages_sent: usize,
}
```

## See Also

- **AgentTool**: [`docs/agents.md`](agents.md) - Subagent spawning from tool calls
- **TaskSystem**: [`docs/tasks.md`](tasks.md) - Persistent task management
- **PlanMode**: [`docs/plan_mode.md`](plan_mode.md) - Structured workflow management
