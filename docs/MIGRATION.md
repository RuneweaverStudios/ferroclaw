# Migration Guide - Claude Code to Ferroclaw

## Overview

This guide helps you migrate from Claude Code to Ferroclaw, highlighting feature parity, command mappings, and key differences.

**Key Differences:**
- **Language**: Rust (Ferroclaw) vs TypeScript (Claude Code)
- **Architecture**: Single binary vs Node.js application
- **Memory**: Dual system (SQLite + files) vs file-based only
- **Tasks**: SQLite with dependencies vs in-memory with dependencies
- **Tools**: 80+ built-in tools vs 40+ tools

## Feature Parity Matrix

| Feature | Claude Code | Ferroclaw | Status |
|---------|-------------|-----------|--------|
| **File Editing** | EditTool | FileEditTool | ✅ Full parity |
| **Task Management** | TaskCreate/List/Update | TaskSystem | ✅ Enhanced (persistent) |
| **Memory Directory** | MemdirSystem | MemdirSystem | ✅ Full parity |
| **Plan Mode** | PlanMode | PlanMode | ✅ Full parity |
| **Git Commit** | CommitTool | Commit Command | ✅ Full parity |
| **Code Review** | ReviewTool | Review Command | ✅ Enhanced (quality scoring) |
| **Subagents** | AgentTool | AgentTool | ⚠️ Simplified (roadmap for full) |
| **Hooks** | PreToolUse/PostToolUse | HookSystem | ✅ Enhanced (6 lifecycle points) |
| **Memory Store** | MemoryStore | MemoryStore | ✅ Full parity (SQLite + FTS5) |
| **DietMCP** | DietMCP | Native MCP + DietMCP | ✅ Enhanced (70-93% compression) |

## Command Mapping

### File Editing

**Claude Code:**
```typescript
await editTool({
  file_path: "/path/to/file.rs",
  old_string: "let x = 1;",
  new_string: "let x = 42;"
});
```

**Ferroclaw:**
```json
{
  "name": "file_edit",
  "arguments": {
    "file_path": "/path/to/file.rs",
    "old_string": "let x = 1;",
    "new_string": "let x = 42;"
  }
}
```

**Key Differences:**
- Ferroclaw uses exact string matching (same as Claude Code)
- Both enforce uniqueness validation
- Ferroclaw uses atomic writes via tempfile

### Task Management

**Claude Code:**
```typescript
// Create task
await taskCreate({
  subject: "Implement feature",
  description: "Build the feature",
  active_form: "Implementing feature",
  owner: null,
  blocked_by: [],
  metadata: {}
});

// List tasks
await taskList({
  status: "pending",
  owner: null
});

// Update task
await taskUpdate({
  task_id: "task-123",
  status: "in_progress"
});
```

**Ferroclaw:**
```bash
# CLI
ferroclaw task create --subject "Implement feature" --description "Build the feature"
ferroclaw task list --status pending
ferroclaw task update task-123 --status in_progress

# Programmatic
use ferroclaw::tasks::{TaskStore, TaskStatus, TaskFilter};

let store = TaskStore::new(None)?;

// Create
let task = store.create(
    "Implement feature",
    "Build the feature",
    Some("Implementing feature".into()),
    None,
    vec![],
    vec![],
    HashMap::new(),
)?;

// List
let filter = TaskFilter {
    status: Some(TaskStatus::Pending),
    owner: None,
    blocked_by: None,
};
let pending = store.list(Some(filter))?;

// Update
store.set_status(&task.id, TaskStatus::InProgress)?;
```

**Key Differences:**
- **Persistence**: Ferroclaw uses SQLite (persistent), Claude Code uses in-memory
- **API**: Ferroclaw provides both CLI and programmatic APIs
- **Dependencies**: Both support bidirectional dependency tracking
- **Cycle Detection**: Both prevent circular dependencies

### Memory Directory

**Claude Code:**
```typescript
// Read memory
const memory = await memdir.loadMemoryPrompt();

// Write topic file
await memdir.writeTopicFile("project_context", "# Project\n\n...");

// List topics
const topics = await memdir.listTopicFiles();
```

**Ferroclaw:**
```rust
use ferroclaw::memory::Memdir;

let memdir = Memdir::new()?;

// Read memory
let memory = memdir.load_memory_prompt()?;

// Write topic file
memdir.write_topic_file("project_context", "# Project\n\n...")?;

// List topics
let topics = memdir.list_topic_files()?;
```

**Key Differences:**
- **Identical API**: Nearly identical method signatures
- **Truncation**: Both use 200 lines / 25KB limits
- **File Organization**: Both use MEMORY.md + topic files structure

### Plan Mode

**Claude Code:**
```typescript
// Enter plan mode
await planMode.activate();

// Create steps
await planMode.createStep({
  subject: "Design API",
  description: "Design REST endpoints",
  acceptance_criteria: ["Endpoints documented"],
  depends_on: []
});

// Advance phase
await planMode.advancePhase();
```

**Ferroclaw:**
```rust
use ferroclaw::modes::plan::{PlanMode, PlanPhase};

let mut plan = PlanMode::new(None)?;

// Create steps
plan.add_step(
    "Design API",
    "Design REST endpoints",
    Some("Designing API".into()),
    vec![],
    vec![],
    vec!["Endpoints documented".into()],
    false,
)?;

// Advance phase
plan.advance_phase()?;
```

**Key Differences:**
- **Phases**: Both use Research, Planning, Implementation, Verification
- **Storage**: Ferroclaw uses TaskSystem (persistent), Claude Code uses in-memory
- **Waves**: Both support dependency-based wave execution

### Git Commit

**Claude Code:**
```typescript
await commitTool({
  yes: false,
  amend: false,
  repo_path: "."
});
```

**Ferroclaw:**
```bash
# CLI
ferroclaw commit
ferroclaw commit --yes
ferroclaw commit --amend

# Programmatic via tool
{
  "name": "commit",
  "arguments": {
    "yes": false,
    "amend": false,
    "repo_path": "."
  }
}
```

**Key Differences:**
- **Identical**: Both use conventional commits format
- **Integration**: Both use git2 for Git operations
- **Workflow**: Both provide interactive approval

### Code Review

**Claude Code:**
```typescript
await reviewTool({
  scope: "staged",
  min_severity: "high",
  file_pattern: "**/*.rs"
});
```

**Ferroclaw:**
```bash
# CLI
ferroclaw review --scope staged --severity high --pattern "**/*.rs"

# Programmatic via tool
{
  "name": "review",
  "arguments": {
    "scope": "staged",
    "min_severity": "high",
    "file_pattern": "**/*.rs"
  }
}
```

**Key Differences:**
- **Enhanced**: Ferroclaw adds quality scoring (0-100)
- **Categories**: Both support multiple issue categories
- **Output**: Both support text and JSON formats

### Subagents

**Claude Code:**
```typescript
await agentTool({
  agent_type: "planner",
  prompt: "You are a planning specialist...",
  task: "Create implementation plan",
  allowed_tools: ["read_file", "write_file"]
});
```

**Ferroclaw:**
```json
{
  "name": "agent",
  "arguments": {
    "agent_type": "planner",
    "prompt": "You are a planning specialist...",
    "task": "Create implementation plan",
    "allowed_tools": ["read_file", "write_file"]
  }
}
```

**Key Differences:**
- **⚠️ Simplified**: Ferroclaw's current implementation is simplified
- **Roadmap**: Full AgentLoop integration planned
- **Memory Isolation**: Both maintain isolated agent contexts

### Hooks

**Claude Code:**
```typescript
// Pre-tool use hook
hooks.register({
  preToolUse: async (tool, args) => {
    console.log(`Calling ${tool} with`, args);
    return { allowedTools: args.allowedTools };
  }
});

// Post-tool use hook
hooks.register({
  postToolUse: async (tool, result) => {
    console.log(`${tool} returned`, result);
  }
});
```

**Ferroclaw:**
```rust
use ferroclaw::hooks::{Hook, HookContext, HookResult};

struct LoggingHook;

impl Hook for LoggingHook {
    fn pre_tool(&self, _ctx: &HookContext, tool: &str, args: &Value) -> HookResult {
        println!("Calling {} with {:?}", tool, args);
        HookResult::Continue
    }

    fn post_tool(&self, _ctx: &HookContext, tool: &str, result: &ToolResult) -> HookResult {
        println!("{} returned {:?}", tool, result);
        HookResult::Continue
    }
}

// Register
registry.hooks().register(Box::new(LoggingHook));
```

**Key Differences:**
- **Enhanced**: Ferroclaw has 6 lifecycle points vs 2 in Claude Code
- **Control Flow**: Ferroclaw supports Halt, ModifyArguments, ModifyResult
- **Built-ins**: Ferroclaw provides 5 built-in hooks

## Configuration Migration

### Claude Code Configuration

```json
// ~/.claude/settings.json
{
  "alwaysThinkingEnabled": true,
  "defaultModel": "claude-sonnet-4-20250514",
  "allowedTools": ["read_file", "write_file"],
  " anthropicApiKey": "sk-ant-..."
}
```

### Ferroclaw Configuration

```toml
# ~/.config/ferroclaw/config.toml
[agent]
default_model = "claude-sonnet-4-20250514"
max_iterations = 30
token_budget = 200000

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"

[security]
default_capabilities = ["fs_read", "net_outbound", "memory_read", "memory_write"]
audit_enabled = true
```

**Key Differences:**
- **Format**: TOML (Ferroclaw) vs JSON (Claude Code)
- **Providers**: Ferroclaw supports 4 providers vs 1 in Claude Code
- **Security**: Ferroclaw has capability system, Claude Code uses tool allowlists

## Memory Migration

### Claude Code Memory Location

```
~/.claude/memory/
├── MEMORY.md
├── user_role.md
└── project_context.md
```

### Ferroclaw Memory Location

```
~/.local/share/ferroclaw/memory/
├── MEMORY.md
├── user_role.md
└── project_context.md
```

**Migration Script:**
```bash
#!/bin/bash
# Migrate Claude Code memory to Ferroclaw

CLAUDE_MEMORY="$HOME/.claude/memory"
FERRO_MEMORY="$HOME/.local/share/ferroclaw/memory"

# Create Ferroclaw memory directory
mkdir -p "$FERRO_MEMORY"

# Copy memory files
cp -r "$CLAUDE_MEMORY"/* "$FERRO_MEMORY/"

echo "Memory migrated to $FERRO_MEMORY"
```

## Task Migration

Claude Code uses in-memory tasks (lost on restart). Ferroclaw uses persistent SQLite storage.

**Export Claude Code Tasks:**
```typescript
// In Claude Code
const tasks = await taskList({});
console.log(JSON.stringify(tasks, null, 2));
```

**Import to Ferroclaw:**
```bash
#!/bin/bash
# Import tasks from Claude Code export

# Read exported tasks
TASKS=$(cat tasks.json)

# Create tasks in Ferroclaw
echo "$TASKS" | jq -r '.[] | [.subject, .description] | @tsv' | while IFS=$'\t' read -r subject desc; do
    ferroclaw task create --subject "$subject" --description "$desc"
done
```

## Quick Start Migration

### Step 1: Install Ferroclaw

```bash
git clone https://github.com/RuneweaverStudios/ferroclaw
cd ferroclaw
cargo build --release
```

### Step 2: Run Setup Wizard

```bash
./target/release/ferroclaw setup
```

This will:
- Configure providers (Anthropic, OpenAI, etc.)
- Set up security capabilities
- Initialize databases
- Create configuration files

### Step 3: Migrate Configuration

**Manual Migration:**
```bash
# Copy API keys
export ANTHROPIC_API_KEY="sk-ant-..."

# Or use provider-specific keys
export ZAI_API_KEY="..."
export OPENROUTER_API_KEY="..."
```

**Automatic Migration:**
```bash
# Ferroclaw can read Claude Code config
./target/release/ferroclaw config import --from-claude
```

### Step 4: Migrate Memory

```bash
# Copy memory files
cp -r ~/.claude/memory/* ~/.local/share/ferroclaw/memory/
```

### Step 5: Test Migration

```bash
# Run Ferroclaw
./target/release/ferroclaw run

# Test basic operations
ferroclaw exec "List files in /tmp"
ferroclaw task list
ferroclaw review
```

## Common Migration Patterns

### Pattern 1: Replace EditTool Calls

**Before (Claude Code):**
```typescript
await editTool({
  file_path: "src/main.rs",
  old_string: "println!(\"Hello\");",
  new_string: "println!(\"Hello, World!\");"
});
```

**After (Ferroclaw):**
```json
{
  "name": "file_edit",
  "arguments": {
    "file_path": "src/main.rs",
    "old_string": "println!(\"Hello\");",
    "new_string": "println!(\"Hello, World!\");"
  }
}
```

### Pattern 2: Replace Task Management

**Before (Claude Code):**
```typescript
const task = await taskCreate({
  subject: "Fix bug",
  description: "Fix login error",
  active_form: "Fixing bug"
});
```

**After (Ferroclaw):**
```bash
ferroclaw task create \
  --subject "Fix bug" \
  --description "Fix login error" \
  --active-form "Fixing bug"
```

### Pattern 3: Replace Memory Operations

**Before (Claude Code):**
```typescript
await memdir.writeTopicFile("project", "# Project\n\n...");
```

**After (Ferroclaw):**
```rust
use ferroclaw::memory::Memdir;

let memdir = Memdir::new()?;
memdir.write_topic_file("project", "# Project\n\n...")?;
```

## Troubleshooting

### Issue: Tasks not persisting

**Claude Code**: Tasks are in-memory (lost on restart)

**Ferroclaw**: Tasks persist in SQLite

**Solution**: No action needed, tasks automatically persist

### Issue: Memory files not found

**Claude Code**: `~/.claude/memory/`

**Ferroclaw**: `~/.local/share/ferroclaw/memory/`

**Solution**: Copy memory files to new location

### Issue: Configuration format different

**Claude Code**: JSON

**Ferroclaw**: TOML

**Solution**: Use setup wizard or manually convert config

### Issue: Hooks not executing

**Claude Code**: `preToolUse`, `postToolUse`

**Ferroclaw**: `pre_tool`, `post_tool`, `permission_check`, etc.

**Solution**: Update hook method names and signatures

## Advanced Migration

### Custom Tools

**Claude Code**:
```typescript
tools.register({
  name: "custom_tool",
  description: "My custom tool",
  parameters: { ... },
  handler: async (args) => { ... }
});
```

**Ferroclaw**: Use skill system
```toml
[skill]
name = "custom_tool"
description = "My custom tool"
version = "0.1.0"

[skill.tool]
type = "bash"
command_template = "my-command {{arg1}}"

[skill.security]
required_capabilities = ["process_exec"]
```

### Custom Hooks

**Claude Code**:
```typescript
hooks.register({
  postToolUse: async (tool, result) => { ... }
});
```

**Ferroclaw**:
```rust
struct MyHook;
impl Hook for MyHook {
    fn post_tool(&self, ctx: &HookContext, tool: &str, result: &ToolResult) -> HookResult {
        // ...
    }
}
```

## Support

**Documentation**:
- Main features: [`FEATURES.md`](../FEATURES.md)
- Individual features: [`docs/`](./)
- Architecture: [`ARCHITECTURE.md`](ARCHITECTURE.md)

**Community**:
- Issues: [GitHub Issues](https://github.com/RuneweaverStudios/ferroclaw/issues)
- Discussions: [GitHub Discussions](https://github.com/RuneweaverStudios/ferroclaw/discussions)

**Getting Help**:
- Check individual feature docs for detailed examples
- Review troubleshooting sections
- Ask questions in GitHub Discussions
