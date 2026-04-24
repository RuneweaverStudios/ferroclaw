# TaskSystem - Persistent Task Management

## Overview

TaskSystem provides SQLite-backed task management with dependency tracking, status workflows, and metadata support. Inspired by Claude Code's task tools, it enables complex multi-step project management with persistent storage.

**Key Features:**
- Persistent SQLite storage
- Dependency tracking with cycle detection
- Status workflow (pending → in_progress → completed)
- Rich metadata support
- CLI and programmatic APIs
- Full-text search capability

## Core Concepts

### Task Structure

Every task contains:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Auto | Unique task identifier (timestamp + random) |
| `subject` | string | Yes | Brief title (e.g., "Implement feature") |
| `description` | string | Yes | Detailed description of work |
| `active_form` | string | No | Present tense for progress (e.g., "Implementing") |
| `status` | enum | Yes | Current status (pending/in_progress/completed) |
| `owner` | string | No | Agent or user assigned to task |
| `blocks` | array[str] | No | Task IDs that depend on this task |
| `blocked_by` | array[str] | No | Task IDs this task depends on |
| `metadata` | object | No | Additional key-value data |
| `created_at` | timestamp | Auto | Creation time |
| `updated_at` | timestamp | Auto | Last update time |

### Status Workflow

```
┌─────────────┐
│   Pending   │ ◄──── Default status
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ InProgress  │ ◄──── Active work
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Completed  │ ◄──── Finished work
└─────────────┘
```

**Rules:**
- Tasks start as `pending`
- Any status transition is allowed
- Status affects `list()` filtering
- `blocked_by` tasks don't auto-block status (manual enforcement)

### Dependency Tracking

Two complementary fields:

- **`blocks`**: Tasks that depend on this task (forward reference)
- **`blocked_by`**: Tasks this task depends on (backward reference)

**Example:**
```rust
Task 1: "Design database" (blocks: [2, 3])
Task 2: "Create tables" (blocked_by: [1])
Task 3: "Write migrations" (blocked_by: [1])
```

**Bidirectional Management:**
- When you add `blocked_by: [1]` to Task 2, Task 1's `blocks` automatically includes Task 2
- When you add `blocks: [2, 3]` to Task 1, Tasks 2 and 3's `blocked_by` automatically include Task 1

### Cycle Detection

TaskSystem prevents circular dependencies:

```
❌ FORBIDDEN:
Task 1 → blocked_by: [2]
Task 2 → blocked_by: [1]
Result: Error - Cycle detected: 1 → 2 → 1

✅ ALLOWED:
Task 1 → blocks: [2]
Task 2 → blocked_by: [1]
Task 3 → blocked_by: [1, 2]
```

## CRUD Operations

### Create Task

```rust
use ferroclaw::tasks::TaskStore;
use std::collections::HashMap;

let store = TaskStore::new(None)?;

let task = store.create(
    "Implement authentication",  // subject
    "Add login and registration", // description
    Some("Implementing auth".into()), // active_form
    Some("agent-1".into()),       // owner
    vec![],                       // blocks (empty)
    vec![],                       // blocked_by (empty)
    HashMap::new(),               // metadata
)?;
```

**CLI:**
```bash
ferroclaw task create \
  --subject "Implement authentication" \
  --description "Add login and registration" \
  --active-form "Implementing auth" \
  --owner "agent-1"
```

### Read Task

```rust
// Get by ID
let task = store.get("1234567890-abc12345")?;

// Error if not found
let task = store.get("nonexistent")
    .expect_err("Task not found");
```

**CLI:**
```bash
ferroclaw task show 1234567890-abc12345
```

### Update Task

```rust
// Update individual fields
store.update_subject(&task.id, "New subject")?;
store.update_description(&task.id, "New description")?;
store.update_active_form(&task.id, Some("New active form".into()))?;
store.update_owner(&task.id, Some("new-owner".into()))?;

// Update status
store.set_status(&task.id, TaskStatus::Completed)?;

// Update metadata
let mut metadata = HashMap::new();
metadata.insert("priority".into(), json!("high"));
store.update_metadata(&task.id, metadata)?;

// Replace metadata
store.replace_metadata(&task.id, new_metadata)?;
```

**CLI:**
```bash
# Update status
ferroclaw task update <id> --status in_progress

# Update subject
ferroclaw task update <id> --subject "New subject"

# Update description
ferroclaw task update <id> --description "New description"
```

### Delete Task

```rust
let deleted = store.delete(&task.id)?;
assert!(deleted);  // true if deleted
```

**CLI:**
```bash
ferroclaw task delete <id>
```

## Dependency Management

### Create with Dependencies

```rust
// Task 2 depends on Task 1
let task1 = store.create("Task 1", "First task", None, None, vec![], vec![], HashMap::new())?;
let task2 = store.create(
    "Task 2",
    "Second task",
    None,
    None,
    vec![],           // blocks
    vec![task1.id()], // blocked_by
    HashMap::new(),
)?;

// Task 1's blocks now includes task2.id
assert!(store.get(&task1.id)?.blocks.contains(&task2.id));
```

**CLI:**
```bash
# Create task with dependencies
ferroclaw task create \
  --subject "Task 2" \
  --description "Second task" \
  --blocked-by 1234567890-abc12345
```

### Add Dependencies Later

```rust
// Add block relationship
store.add_block(&task1.id, &task2.id)?;

// Add blocked_by relationship
store.add_blocked_by(&task3.id, &task1.id)?;
```

**CLI:**
```bash
# Task 1 blocks Task 2
ferroclaw task add-block <task1-id> <task2-id>

# Task 3 is blocked by Task 1
ferroclaw task add-block <task1-id> <task3-id> --reverse
```

### Query Dependencies

```rust
// Get tasks that block this task
let blocking = store.get_blocking(&task.id)?;

// Get tasks that this task blocks
let blocked = store.get_blocked(&task.id)?;

// Check if task is blocked
let is_blocked = store.is_blocked(&task.id)?;
```

**CLI:**
```bash
# Show tasks blocking this task
ferroclaw task blocking <id>

# Show tasks blocked by this task
ferroclaw task blocked <id>
```

### Remove Dependencies

```rust
store.remove_block(&task1.id, &task2.id)?;
```

**CLI:**
```bash
ferroclaw task remove-block <task1-id> <task2-id>
```

## Listing and Filtering

### List All Tasks

```rust
let all_tasks = store.list(None)?;
```

**CLI:**
```bash
ferroclaw task list
```

### Filter by Status

```rust
use ferroclaw::tasks::{TaskFilter, TaskStatus};

let filter = TaskFilter {
    status: Some(TaskStatus::Pending),
    owner: None,
    blocked_by: None,
};

let pending_tasks = store.list(Some(filter))?;
```

**CLI:**
```bash
ferroclaw task list --status pending
ferroclaw task list --status in_progress
ferroclaw task list --status completed
```

### Filter by Owner

```rust
let filter = TaskFilter {
    status: None,
    owner: Some("agent-1".into()),
    blocked_by: None,
};

let my_tasks = store.list(Some(filter))?;
```

**CLI:**
```bash
ferroclaw task list --owner agent-1
```

### Filter by Dependencies

```rust
// Find tasks blocked by specific task
let filter = TaskFilter {
    status: None,
    owner: None,
    blocked_by: Some("1234567890-abc12345".into()),
};

let dependent_tasks = store.list(Some(filter))?;
```

## Metadata Operations

### Set Metadata

```rust
use serde_json::json;

let mut metadata = HashMap::new();
metadata.insert("priority".into(), json!("high"));
metadata.insert("estimated_hours".into(), json!(4));
metadata.insert("tags".into(), json!(["backend", "auth"]));

store.update_metadata(&task.id, metadata)?;
```

### Read Metadata

```rust
let task = store.get(&task.id)?;
let priority = task.metadata.get("priority")
    .and_then(|v| v.as_str());

if let Some("high") = priority {
    println!("High priority task!");
}
```

### Query Metadata

```rust
// Find high priority tasks
let all_tasks = store.list(None)?;
let high_priority: Vec<_> = all_tasks
    .into_iter()
    .filter(|t| {
        t.metadata
            .get("priority")
            .and_then(|v| v.as_str())
            == Some("high")
    })
    .collect();
```

## Programmatic API

### Complete Example

```rust
use ferroclaw::tasks::{TaskStore, TaskFilter, TaskStatus};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize store
    let store = TaskStore::new(None)?;

    // Create tasks with dependencies
    let task1 = store.create(
        "Design database schema",
        "Create ERD and table definitions",
        Some("Designing database".into()),
        None,
        vec![],
        vec![],
        HashMap::new(),
    )?;

    let task2 = store.create(
        "Implement user table",
        "Create users table with auth fields",
        Some("Implementing users table".into()),
        None,
        vec![],
        vec![task1.id()],
        {
            let mut meta = HashMap::new();
            meta.insert("priority".into(), serde_json::json!("high"));
            meta
        },
    )?;

    // Update status
    store.set_status(&task1.id, TaskStatus::Completed)?;

    // List pending tasks
    let filter = TaskFilter {
        status: Some(TaskStatus::Pending),
        owner: None,
        blocked_by: None,
    };

    let pending = store.list(Some(filter))?;
    println!("Pending tasks: {}", pending.len());

    // Query dependencies
    let blocking = store.get_blocking(&task2.id)?;
    println!("Task 2 is blocked by {} tasks", blocking.len());

    Ok(())
}
```

## CLI Usage

### Basic Commands

```bash
# Create task
ferroclaw task create --subject "Fix bug" --description "Login error"

# List all tasks
ferroclaw task list

# Show task details
ferroclaw task show <id>

# Update task
ferroclaw task update <id> --status in_progress

# Delete task
ferroclaw task delete <id>
```

### Dependency Commands

```bash
# Add block relationship
ferroclaw task add-block <blocker-id> <blocked-id>

# Remove block relationship
ferroclaw task remove-block <blocker-id> <blocked-id>

# Show blocking tasks
ferroclaw task blocking <id>

# Show blocked tasks
ferroclaw task blocked <id>
```

### Filtering

```bash
# Filter by status
ferroclaw task list --status pending

# Filter by owner
ferroclaw task list --owner agent-1

# Combine filters
ferroclaw task list --status in_progress --owner agent-1
```

## Best Practices

### 1. Use Descriptive Subjects

```rust
// Good
"Implement OAuth2 authentication flow"

// Bad
"Auth work"
```

### 2. Include Active Forms

```rust
// Shows in progress indicators
active_form: Some("Implementing OAuth2 authentication flow".into())
```

### 3. Break Down Large Tasks

```rust
// Instead of one massive task
"Build entire application"

// Create dependent tasks
let design = store.create("Design app", ...)?;
let frontend = store.create("Build frontend", ..., vec![design.id])?;
let backend = store.create("Build backend", ..., vec![design.id])?;
let integrate = store.create("Integrate", ..., vec![frontend.id(), backend.id()])?;
```

### 4. Use Metadata for Context

```rust
let mut metadata = HashMap::new();
metadata.insert("priority".into(), json!("high"));
metadata.insert("complexity".into(), json!(7));
metadata.insert("assigned_to".into(), json!("alice"));
metadata.insert("tags".into(), json!(["security", "auth"]));
```

### 5. Validate Dependencies

```rust
// Check if task can start
let blocking = store.get_blocking(&task.id)?;
if blocking.is_empty() {
    store.set_status(&task.id, TaskStatus::InProgress)?;
} else {
    println!("Task is blocked by {} tasks", blocking.len());
}
```

## Database Schema

```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    subject TEXT NOT NULL,
    description TEXT NOT NULL,
    active_form TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    owner TEXT,
    blocks TEXT DEFAULT '[]',           -- JSON array
    blocked_by TEXT DEFAULT '[]',       -- JSON array
    metadata TEXT DEFAULT '{}',         -- JSON object
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_owner ON tasks(owner);
```

## Troubleshooting

### Issue: Cycle detected error

**Cause**: You're creating circular dependencies

**Solution**: Check your dependency graph:
```rust
// Visualize dependencies
let task = store.get(&task_id)?;
println!("Blocked by: {:?}", task.blocked_by);
for blocker_id in &task.blocked_by {
    let blocker = store.get(blocker_id)?;
    println!("  {} blocked by: {:?}", blocker.subject, blocker.blocked_by);
}
```

### Issue: Task not found after creation

**Cause**: Database path mismatch or permissions

**Solution**:
```bash
# Check default location
ls -la ~/.local/share/ferroclaw/tasks.db

# Use custom path
let store = TaskStore::new(Some(PathBuf::from("/custom/path/tasks.db")))?;
```

### Issue: Dependencies not updating

**Cause**: Using `blocks` instead of `blocked_by` or vice versa

**Solution**: Remember bidirectional management:
```rust
// When creating Task 2 that depends on Task 1
store.create("Task 2", "...", None, None, vec![], vec![task1.id()], HashMap::new())?;

// Task 1's blocks is automatically updated
```

## See Also

- **PlanMode**: Advanced workflow management with phases
- **MemdirSystem**: File-based memory for project context
- **AgentTool**: Subagent spawning for task execution
