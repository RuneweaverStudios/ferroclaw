# TaskSystem Examples

## Basic Usage

```rust
use ferroclaw::tasks::{TaskStore, TaskStatus};
use std::collections::HashMap;

// Create an in-memory store for testing
let store = TaskStore::in_memory()?;

// Create a new task
let task = store.create(
    "Implement feature X",
    "Build feature X with full test coverage",
    Some("Implementing".into()),  // active_form
    None,                          // no owner
    vec![],                        // blocks nothing
    vec![],                        // blocked by nothing
    HashMap::new(),                // no metadata
)?;

println!("Created task: {}", task.id);

// Update task status
let updated = store.set_status(&task.id, TaskStatus::InProgress)?;
println!("Status: {}", updated.unwrap().status.as_str());

// Get task details
let task = store.get(&task.id)?.unwrap();
println!("Task: {} - {}", task.subject, task.description);
```

## Task Dependencies

```rust
use ferroclaw::tasks::TaskStore;

let store = TaskStore::in_memory()?;

// Create tasks
let task1 = store.create("Task 1", "First task", None, None, vec![], vec![], HashMap::new())?;
let task2 = store.create("Task 2", "Second task", None, None, vec![], vec![], HashMap::new())?;

// task1 blocks task2 (task2 depends on task1)
store.add_block(&task1.id, &task2.id)?;

// Get tasks that task1 is blocking
let blocked = store.get_blocked(&task1.id)?;
println!("Task 1 is blocking {} tasks", blocked.len());

// Get tasks that are blocking task2
let blocking = store.get_blocking(&task2.id)?;
println!("Task 2 is blocked by {} tasks", blocking.len());
```

## Listing with Filters

```rust
use ferroclaw::tasks::{TaskStore, TaskFilter, TaskStatus};

let store = TaskStore::new(None)?;

// List only pending tasks
let filter = TaskFilter {
    status: Some(TaskStatus::Pending),
    owner: None,
    blocked_by: None,
};
let pending_tasks = store.list(Some(filter))?;

// List tasks for a specific owner
let owner_filter = TaskFilter {
    status: None,
    owner: Some("agent1".into()),
    blocked_by: None,
};
let my_tasks = store.list(Some(owner_filter))?;
```

## CLI Usage

```bash
# Create a task
ferroclaw task create --subject "Fix bug" --description "Fix the critical bug"

# List all tasks
ferroclaw task list

# List pending tasks
ferroclaw task list --status pending

# Show task details
ferroclaw task show <task-id>

# Update task status
ferroclaw task update <task-id> --status in_progress

# Add dependency
ferroclaw task add-block <task-id> <blocks-id>

# Show what a task is blocking
ferroclaw task blocked <task-id>

# Delete a task
ferroclaw task delete <task-id>
```
