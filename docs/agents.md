# AgentTool - Subagent Spawning

## Overview

AgentTool enables Ferroclaw to spawn subagents with isolated context and memory, allowing for task delegation and parallel processing. Each subagent maintains its own conversation history and can be resumed later.

**Key Features:**
- Six built-in agent types with specialized prompts
- Memory isolation between agents
- Agent resumption via agent_id
- Custom system prompts
- Tool filtering capabilities

## Built-in Agent Types

### 1. planner

**Purpose**: Break down complex tasks into steps

**System Prompt**:
```
You are a planning specialist. Break down complex tasks into clear, actionable steps.
Identify dependencies, estimate effort, and suggest execution order.
Focus on creating structured plans that can be executed systematically.
```

**Best For**:
- Feature planning
- Architecture design
- Task breakdown
- Risk assessment

### 2. coder

**Purpose**: Write clean, documented code

**System Prompt**:
```
You are a coding specialist. Write clean, well-documented code following best practices.
Focus on readability, maintainability, and proper error handling.
Include tests and documentation with your code.
```

**Best For**:
- Feature implementation
- Code refactoring
- Test writing
- Documentation

### 3. reviewer

**Purpose**: Analyze code for bugs and design problems

**System Prompt**:
```
You are a code review specialist. Analyze code for bugs, security issues, and design problems.
Provide actionable feedback with specific suggestions.
Focus on correctness, security, and maintainability.
```

**Best For**:
- Code review
- Security analysis
- Performance optimization
- Bug detection

### 4. debugger

**Purpose**: Investigate errors systematically

**System Prompt**:
```
You are a debugging specialist. Investigate errors systematically and identify root causes.
Analyze error messages, stack traces, and code context.
Provide clear explanations and step-by-step solutions.
```

**Best For**:
- Error diagnosis
- Bug reproduction
- Root cause analysis
- Fix validation

### 5. researcher

**Purpose**: Gather and synthesize information

**System Prompt**:
```
You are a research specialist. Gather information from multiple sources and synthesize findings.
Focus on accuracy, relevance, and comprehensive coverage.
Cite sources and provide evidence for conclusions.
```

**Best For**:
- Documentation research
- Technology evaluation
- Best practices research
- Competitive analysis

### 6. generic

**Purpose**: General-purpose helper

**System Prompt**:
```
You are a helpful assistant. Assist with a wide range of tasks effectively.
Adapt your approach based on the specific task requirements.
Provide clear, accurate, and helpful responses.
```

**Best For**:
- General assistance
- Simple tasks
- Exploratory work
- Ad-hoc requests

## Tool Interface

### JSON Schema

```json
{
  "name": "agent",
  "description": "Spawn a subagent with isolated context to delegate work.",
  "parameters": {
    "agent_type": "string (optional)",
    "prompt": "string (optional)",
    "task": "string (required)",
    "agent_id": "string (optional)",
    "allowed_tools": "array[string] (optional)"
  }
}
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `agent_type` | string | No | Built-in agent type (planner/coder/reviewer/debugger/researcher/generic) |
| `prompt` | string | No | Custom system prompt (overrides agent_type) |
| `task` | string | Yes | Task description for the agent |
| `agent_id` | string | No | Resume existing agent (instead of spawning new) |
| `allowed_tools` | array[string] | No | Restrict agent to specific tools |

## Usage Examples

### Example 1: Spawn Generic Agent

```json
{
  "agent_type": "generic",
  "task": "Analyze this codebase structure and identify the main components"
}
```

**Response**:
```
Agent ID: agent_abc123

Based on the codebase analysis, here are the main components:

1. Core Agent Loop (src/agent/)
   - ReAct-style reasoning loop
   - Context management
   - Tool execution

2. Tool Registry (src/tools/)
   - Built-in tools
   - MCP tool integration
   - Skill system

3. Memory System (src/memory/)
   - SQLite storage
   - FTS5 search
   - Memdir for file-based memory

...
```

### Example 2: Use Built-in Agent Type

```json
{
  "agent_type": "planner",
  "task": "Create a plan for implementing OAuth2 authentication"
}
```

**Response**:
```
Agent ID: agent_def456

# OAuth2 Implementation Plan

## Phase 1: Research
- Research OAuth2 providers (Google, GitHub)
- Understand OAuth2 flow
- Identify required dependencies

## Phase 2: Design
- Design authentication flow
- Define data models
- Plan API endpoints

## Phase 3: Implementation
- Implement OAuth2 client
- Add session management
- Create authentication endpoints

## Phase 4: Testing
- Write unit tests
- Integration testing
- Security testing
```

### Example 3: Custom System Prompt

```json
{
  "agent_type": "coder",
  "prompt": "You are a Rust specialist. Focus on memory safety, zero-copy patterns, and async/await best practices.",
  "task": "Review this async Rust code for potential issues"
}
```

### Example 4: Resume Existing Agent

```json
{
  "agent_id": "agent_def456",
  "task": "Continue with the implementation phase. Focus on the OAuth2 client."
}
```

**Response**:
```
Resumed agent: agent_def456

# Phase 3: Implementation - OAuth2 Client

Implementing OAuth2 client with the following approach:

1. Use `oauth2` crate for core functionality
2. Implement `OAuthClient` struct
3. Add token refresh logic
4. Handle error cases

Code structure:
```rust
pub struct OAuthClient {
    client_id: String,
    client_secret: String,
    redirect_url: String,
}
...
```
```

### Example 5: Restrict Tools

```json
{
  "agent_type": "reviewer",
  "task": "Review the authentication module for security issues",
  "allowed_tools": ["read_file", "grep_code", "analyze_security"]
}
```

## Memory Isolation

Each agent maintains:

- **Unique agent_id**: UUID v4 identifier
- **Isolated conversation history**: No shared state
- **Independent context**: Separate from parent agent

### Example: Parallel Agents

```json
// Spawn agent 1
{
  "agent_type": "coder",
  "task": "Implement user authentication"
}
// Returns: agent_abc123

// Spawn agent 2 (independent)
{
  "agent_type": "coder",
  "task": "Implement user authorization"
}
// Returns: agent_def456

// Agent 1 and Agent 2 have separate memories
```

## Agent Lifecycle

```
┌─────────────┐
│   Creation  │  ← AgentRegistry generates unique ID
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Execution  │  ← Task processed with isolated context
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Storage   │  ← History stored in AgentMemory
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Resumption │  ← agent_id used to continue work
└─────────────┘
```

### Creation

```rust
let agent = AgentDefinition::builder()
    .agent_type("coder")
    .task("Implement feature")
    .build()?;

let agent_id = registry.spawn(agent)?;
```

### Execution

```rust
let response = registry.execute(&agent_id, task)?;
```

### Storage

```rust
// Conversation history stored automatically
let memory = registry.get_memory(&agent_id)?;
let history = memory.get_history();
```

### Resumption

```rust
// Resume with agent_id
let response = registry.execute(&agent_id, "Continue the work")?;
```

## Advanced Usage

### Custom Agent Prompts

Override built-in prompts:

```json
{
  "prompt": "You are a database optimization expert. Focus on query performance, indexing strategies, and normalization.",
  "task": "Optimize this slow query"
}
```

**Prompt Hierarchy**:
1. Custom `prompt` (if provided)
2. Built-in `agent_type` prompt (if agent_type specified)
3. Generic prompt (fallback)

### Tool Filtering

Restrict agent capabilities:

```json
{
  "agent_type": "coder",
  "task": "Write tests for auth module",
  "allowed_tools": ["read_file", "write_file", "run_tests"]
}
```

**Benefits**:
- Security: Limit agent access
- Performance: Reduce tool search time
- Focus: Guide agent to appropriate tools

### Multi-Agent Orchestration

Coordinate multiple agents:

```rust
// 1. Spawn planner agent
let plan = spawn_agent("planner", "Create implementation plan")?;

// 2. Spawn coder agents for parallel tasks
for task in plan.tasks {
    spawn_agent("coder", task.description)?;
}

// 3. Spawn reviewer agent
spawn_agent("reviewer", "Review all implementations")?;
```

## Current Limitations

This is a **simplified implementation** that simulates agent execution. In production:

### 1. Real AgentLoop Integration

Currently simulates responses. Production would use:

```rust
let mut agent_loop = AgentLoop::new(
    provider,
    tool_registry,
    mcp_client,
    config,
    capabilities,
);
let (response, events) = agent_loop.run(task, &mut memory).await?;
```

### 2. Shared LLM Provider

Needs shared provider architecture:

```rust
impl AgentTool {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        tool_registry: Arc<ToolRegistry>,
        config: Config,
    ) -> Self
}
```

### 3. Tool Filtering

`allowed_tools` field not yet enforced. Production would filter:

```rust
fn filter_tools(&self, allowed: &[String]) -> Vec<ToolDefinition> {
    registry
        .get_all_tools()
        .into_iter()
        .filter(|t| allowed.contains(&t.name))
        .collect()
}
```

### 4. Memory Store Integration

AgentMemory doesn't use persistent MemoryStore. Production would:

```rust
pub struct AgentMemory {
    store: MemoryStore,
    agent_id: String,
}
```

### 5. Context Management

No token budget tracking per agent. Production would:

```rust
pub struct AgentContext {
    agent_id: String,
    token_budget: usize,
    token_count: usize,
}
```

## Best Practices

### 1. Choose Appropriate Agent Type

```rust
// Good: Specific agent for specific task
spawn_agent("reviewer", "Review this code")

// Bad: Generic agent for specialized task
spawn_agent("generic", "Review this code")
```

### 2. Provide Clear Tasks

```json
{
  "agent_type": "coder",
  "task": "Implement OAuth2 authentication with Google provider, including token refresh and error handling"
}
```

### 3. Use Custom Prompts for Specialization

```json
{
  "agent_type": "coder",
  "prompt": "You are a Rust async/await expert. Focus on Send, Sync, and lifetime safety.",
  "task": "Review this async code"
}
```

### 4. Resume Agents for Long-Running Tasks

```json
// Initial task
{"agent_type": "planner", "task": "Create implementation plan"}
// Returns: agent_abc123

// Resume with next phase
{"agent_id": "agent_abc123", "task": "Execute phase 2: Design"}
```

### 5. Combine with TaskSystem

```rust
// Create task
let task = task_store.create("Plan feature", "...", None, None, vec![], vec![], HashMap::new())?;

// Delegate to agent
let agent = spawn_agent("planner", &task.description)?;

// Update task with agent results
task_store.update_description(&task.id, &agent_response)?;
```

## Troubleshooting

### Issue: Agent not responding as expected

**Cause**: Wrong agent type or unclear task

**Solution**:
```json
// Try different agent type
{"agent_type": "debugger", "task": "Investigate this error"}

// Or use custom prompt
{"prompt": "You are a security expert. Focus on auth vulnerabilities.", "task": "..."}
```

### Issue: Agent loses context

**Cause**: Agent memory is isolated per session

**Solution**: Use agent_id to resume:
```json
{"agent_id": "agent_abc123", "task": "Continue with previous context"}
```

### Issue: Agent can't access needed tools

**Cause**: Tool filtering too restrictive

**Solution**: Expand allowed_tools or remove restriction:
```json
{
  "allowed_tools": ["read_file", "write_file", "bash", "grep_code"]
}
```

## See Also

- **TaskSystem**: Persistent task management
- **PlanMode**: Structured workflow management
- **Tool Registry**: Available tools for agents
