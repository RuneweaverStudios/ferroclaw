# Ferroclaw - Feature Documentation

## Overview

Ferroclaw extends Claude Code's capabilities with 10 major features implemented across 3 development waves. This document provides a comprehensive reference for all features.

**What is Ferroclaw?**
Ferroclaw is a security-first, single-binary AI agent framework written in Rust. It provides powerful tools for task management, memory organization, code editing, and workflow automation.

## Quick Start

### Installation

```bash
# Build from source
git clone https://github.com/RuneweaverStudios/ferroclaw
cd ferroclaw
cargo build --release

# Run setup wizard
./target/release/ferroclaw setup
```

### Basic Usage

```bash
# Interactive mode
ferroclaw run

# One-shot command
ferroclaw exec "List files in /tmp"

# Review staged changes
ferroclaw review

# Commit with conventional commits
ferroclaw commit

# Task management
ferroclaw task list
ferroclaw task create --subject "Fix bug" --description "..."
```

## Feature Reference

### 1. FileEditTool - Exact String Replacement

**Purpose**: Safe file editing through exact string matching

**Key Capabilities**:
- Exact string replacement (no regex, no patterns)
- Uniqueness validation
- Atomic write operations
- Multi-line support

**Documentation**: [`docs/file_edit.md`](docs/file_edit.md)

**Quick Example**:
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

**Use Cases**:
- Precise code refactoring
- Configuration file updates
- Multi-line code block replacement
- Safe atomic file edits

---

### 2. TaskSystem - Persistent Task Management

**Purpose**: SQLite-backed task tracking with dependencies

**Key Capabilities**:
- Persistent storage (SQLite)
- Dependency tracking with cycle detection
- Status workflow (pending → in_progress → completed)
- Rich metadata support
- CLI and programmatic APIs

**Documentation**: [`docs/tasks.md`](docs/tasks.md)

**Quick Example**:
```rust
use ferroclaw::tasks::TaskStore;

let store = TaskStore::new(None)?;
let task = store.create(
    "Implement authentication",
    "Add OAuth2 login",
    Some("Implementing auth".into()),
    None,
    vec![],
    vec![],
    HashMap::new(),
)?;
```

**Use Cases**:
- Project task tracking
- Feature planning
- Bug tracking
- Sprint management

---

### 3. MemdirSystem - File-Based Persistent Memory

**Purpose**: Organize long-term memory in topic files

**Key Capabilities**:
- File-based memory organization
- Automatic truncation (200 lines / 25KB)
- Topic file categorization
- LLM prompt generation
- Complements SQLite MemoryStore

**Documentation**: [`docs/memory.md`](docs/memory.md)

**Quick Example**:
```rust
use ferroclaw::memory::Memdir;

let memdir = Memdir::new()?;
memdir.write_topic_file("project_context", "# Project Overview\n\n...")?;
let memory_prompt = memdir.load_memory_prompt()?;
```

**Use Cases**:
- Project documentation
- User preferences storage
- Long-form context for LLMs
- Persistent project memory

---

### 4. PlanMode - Structured Multi-Phase Planning

**Purpose**: Manage complex tasks through phases

**Key Capabilities**:
- Four-phase workflow (Research, Planning, Implementation, Verification)
- Dependency-based wave execution
- Approval gates for phase transitions
- Acceptance criteria per step
- Integration with TaskSystem

**Documentation**: [`docs/plan_mode.md`](docs/plan_mode.md)

**Quick Example**:
```rust
use ferroclaw::modes::plan::{PlanMode, PlanPhase};

let mut plan = PlanMode::new(None)?;
plan.add_step("Design DB", "Create schema", None, vec![], vec![], vec![], false)?;
plan.advance_phase()?; // Research → Planning
```

**Use Cases**:
- Complex feature development
- Multi-step projects
- Team coordination
- Quality gates

---

### 5. Commit Command - Conventional Commits

**Purpose**: Automated commit message generation

**Key Capabilities**:
- Conventional commit format
- Staged changes analysis
- Diff preview
- Interactive approval workflow
- Commit amendment support

**Documentation**: [`docs/git_workflow.md`](docs/git_workflow.md#commit-command)

**Quick Example**:
```bash
# Interactive commit
ferroclaw commit

# Auto-approve
ferroclaw commit --yes

# Amend previous commit
ferroclaw commit --amend
```

**Use Cases**:
- Standardized commit messages
- Automated changelog generation
- Commit policy enforcement
- Code review workflow

---

### 6. Review Command - Code Review Automation

**Purpose**: Automated code quality analysis

**Key Capabilities**:
- Diff analysis at multiple scopes
- Quality scoring (0-100)
- Issue detection by category and severity
- Actionable recommendations
- Text and JSON output formats

**Documentation**: [`docs/git_workflow.md`](docs/git_workflow.md#review-command)

**Quick Example**:
```bash
# Review staged changes
ferroclaw review

# Review commit range
ferroclaw review --scope main..HEAD

# Filter by severity
ferroclaw review --severity high
```

**Use Cases**:
- Pre-commit code review
- Pull request analysis
- Quality metrics tracking
- CI/CD integration

---

### 7. AgentTool - Subagent Spawning

**Purpose**: Delegate work to specialized subagents

**Key Capabilities**:
- Six built-in agent types (planner, coder, reviewer, debugger, researcher, generic)
- Memory isolation between agents
- Agent resumption via agent_id
- Custom system prompts
- Tool filtering capabilities

**Documentation**: [`docs/agents.md`](docs/agents.md)

**Quick Example**:
```json
{
  "name": "agent",
  "arguments": {
    "agent_type": "planner",
    "task": "Create a plan for implementing OAuth2 authentication"
  }
}
```

**Use Cases**:
- Task delegation
- Parallel processing
- Specialized assistance
- Context isolation

---

### 8. HookSystem - Event-Driven Extensibility

**Purpose**: Intercept and modify tool execution

**Key Capabilities**:
- Six lifecycle hook points
- Control flow modification (halt, modify args/results)
- Five built-in hooks (Logging, Audit, RateLimit, Security, Metrics)
- Thread-safe concurrent execution
- Custom hook implementation

**Documentation**: [`docs/hooks.md`](docs/hooks.md)

**Quick Example**:
```rust
use ferroclaw::hooks::builtin::{LoggingHook, AuditHook};

registry.hooks().register(Box::new(LoggingHook::new(true, false)));
registry.hooks().register(Box::new(AuditHook::new()));
```

**Use Cases**:
- Request logging
- Security auditing
- Rate limiting
- Custom validation
- Metrics collection

---

## Examples

### Example 1: Complete Development Workflow

```bash
# 1. Plan feature
ferroclaw exec "Plan OAuth2 authentication implementation"

# 2. Create tasks
ferroclaw task create --subject "Design OAuth2 flow" --description "..."
ferroclaw task create --subject "Implement OAuth2 client" --description "..."

# 3. Write code (with automatic edits)
ferroclaw exec "Implement OAuth2 client in src/auth/oauth.rs"

# 4. Review changes
ferroclaw review --scope working

# 5. Commit changes
ferroclaw commit --yes
```

### Example 2: Code Review Workflow

```bash
# Stage changes
git add src/auth.rs

# Review before committing
ferroclaw review --scope staged --severity high

# If review passes, commit
ferroclaw commit --yes
```

### Example 3: Task-Based Development

```rust
use ferroclaw::tasks::TaskStore;

// Create plan
let store = TaskStore::new(None)?;

// Create tasks with dependencies
let design = store.create("Design DB", "...", None, None, vec![], vec![], HashMap::new())?;
let impl_tables = store.create("Create tables", "...", None, None, vec![], vec![design.id()], HashMap::new())?;
let impl_queries = store.create("Write queries", "...", None, None, vec![], vec![design.id()], HashMap::new())?;

// Update status
store.set_status(&design.id(), TaskStatus::Completed)?;

// Check ready tasks
let filter = TaskFilter {
    status: Some(TaskStatus::Pending),
    owner: None,
    blocked_by: None,
};
let ready = store.list(Some(filter))?;
```

### Example 4: Memory Organization

```rust
use ferroclaw::memory::{Memdir, MemoryStore};

// Initialize both systems
let store = MemoryStore::new(None)?;
let memdir = Memdir::new()?;

// Save quick facts to SQLite
store.insert("api_key", "sk-...")?;

// Save detailed context to files
memdir.write_topic_file("project_context", "# Project Overview\n\n...")?;
memdir.write_topic_file("coding_standards", "# Code Style\n\n...")?;

// Generate memory prompt for LLM
let memory_prompt = memdir.load_memory_prompt()?;
```

## API Reference

### Tool API

All tools follow the same JSON schema:

```json
{
  "name": "tool_name",
  "arguments": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

**Available Tools**:
- `file_edit` - Edit files with exact string replacement
- `commit` - Create conventional commits
- `review` - Automated code review
- `agent` - Spawn subagents
- `read_file` - Read file contents
- `write_file` - Write to files
- `bash` - Execute shell commands
- And 80+ more built-in tools

### CLI API

```bash
# Task management
ferroclaw task <subcommand> [options]

# Git workflow
ferroclaw commit [options]
ferroclaw review [options]

# Agent execution
ferroclaw run                    # Interactive
ferroclaw exec "<prompt>"        # One-shot
```

### Programmatic API

```rust
use ferroclaw::{Ferroclaw, Config};

// Initialize
let config = Config::load()?;
let ferroclaw = Ferroclaw::new(config)?;

// Execute prompt
let response = ferroclaw.execute("Analyze this codebase").await?;

// Access subsystems
let tasks = ferroclaw.tasks();
let memory = ferroclaw.memory();
let tools = ferroclaw.tools();
```

## Troubleshooting

### Common Issues

**Issue**: FileEditTool reports "string not found"
- **Solution**: Verify exact text including whitespace
- **Solution**: Check for hidden characters with `cat -A`

**Issue**: Task won't execute (blocked)
- **Solution**: Check dependencies with `ferroclaw task blocking <id>`
- **Solution**: Complete blocking tasks first

**Issue**: MEMORY.md keeps getting truncated
- **Solution**: Move detail to topic files, keep index concise
- **Solution**: Use `ferroclaw memory list` to see topics

**Issue**: Review command shows no issues
- **Solution**: Check if changes are staged (`git status`)
- **Solution**: Try `--scope working` instead of `--scope staged`

**Issue**: Agent loses context between calls
- **Solution**: Use `agent_id` to resume: `{"agent_id": "agent_abc", "task": "..."}`
- **Solution**: Provide more context in task description

### Getting Help

- **Documentation**: See individual feature docs in `docs/`
- **Examples**: Check `examples/` directory
- **Issues**: Report bugs at GitHub Issues
- **Community**: Join discussions in GitHub Discussions

## Feature Matrix

| Feature | Status | Storage | CLI | API | Docs |
|---------|--------|---------|-----|-----|------|
| FileEditTool | ✅ Complete | N/A | ✅ | ✅ | [`docs/file_edit.md`](docs/file_edit.md) |
| TaskSystem | ✅ Complete | SQLite | ✅ | ✅ | [`docs/tasks.md`](docs/tasks.md) |
| MemdirSystem | ✅ Complete | Files | ❌ | ✅ | [`docs/memory.md`](docs/memory.md) |
| PlanMode | ✅ Complete | SQLite | ❌ | ✅ | [`docs/plan_mode.md`](docs/plan_mode.md) |
| Commit Command | ✅ Complete | N/A | ✅ | ✅ | [`docs/git_workflow.md`](docs/git_workflow.md) |
| Review Command | ✅ Complete | N/A | ✅ | ✅ | [`docs/git_workflow.md`](docs/git_workflow.md) |
| AgentTool | ✅ Simplified | Memory | ✅ | ✅ | [`docs/agents.md`](docs/agents.md) |
| HookSystem | ✅ Complete | N/A | ❌ | ✅ | [`docs/hooks.md`](docs/hooks.md) |

## Roadmap

### Future Enhancements

**AgentTool**:
- Real AgentLoop integration
- Persistent MemoryStore integration
- Tool filtering enforcement
- Per-agent token budgets

**PlanMode**:
- Visual plan visualization
- Drag-and-drop plan editing
- Plan templates
- Export/import plans

**Review Command**:
- More issue categories
- Custom rule definitions
- Integration with linters
- Automatic fix suggestions

**TaskSystem**:
- Task templates
- Recurring tasks
- Task dependencies visualization
- Sprint management

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines on:
- Code style
- Testing requirements
- Documentation standards
- Pull request process

## License

MIT License - See [`LICENSE`](LICENSE) for details

## See Also

- **Main README**: [`README.md`](README.md)
- **Architecture**: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- **Benchmarks**: [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- **Security**: [`docs/SECURITY.md`](docs/SECURITY.md)
- **Migration Guide**: [`docs/MIGRATION.md`](docs/MIGRATION.md)
