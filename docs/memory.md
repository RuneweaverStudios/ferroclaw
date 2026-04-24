# MemdirSystem - File-Based Persistent Memory

## Overview

MemdirSystem provides file-based persistent memory organization, inspired by Claude Code's memory directory system. It combines a central index file (MEMORY.md) with topic-specific files, automatic truncation, and formatted prompt generation.

**Key Features:**
- File-based memory organization (human-readable, git-friendly)
- Automatic truncation at 200 lines or 25KB
- Topic file organization for structured memory
- Formatted prompt generation for LLM context
- Complements SQLite-based MemoryStore

## File Organization

### Directory Structure

```
~/.local/share/ferroclaw/memory/
├── MEMORY.md              # Entry point (index, max 200 lines / 25KB)
├── user_role.md           # Topic: User's role and context
├── project_context.md     # Topic: Project-specific context
├── feedback_testing.md    # Topic: Testing preferences
└── ...                    # Additional topic files
```

### MEMORY.md (Entry Point)

The index file that provides:
- Overview of all memory topics
- Quick reference pointers to topic files
- Truncated automatically when too large

**Example MEMORY.md:**
```markdown
# Project Memory

## User Context
See user_role.md

## Project Overview
See project_context.md

## Testing Preferences
See feedback_testing.md

## Quick Reference
- Language: Rust
- Framework: Ferroclaw
- Testing: cargo test
```

### Topic Files

Individual files for specific topics:
- **user_role.md** - User's background, goals, preferences
- **project_context.md** - Project structure, architecture, goals
- **coding_style.md** - Code conventions, patterns
- **feedback_history.md** - Past feedback and lessons learned

**Example topic file (user_role.md):**
```markdown
# User Role and Context

## Background
Senior Rust developer working on AI agent framework.

## Goals
- Build secure, performant agent system
- Maintain 80%+ test coverage
- Follow Rust best practices

## Preferences
- Prefers explicit error handling over unwrap()
- Values type safety over runtime checks
- Uses structured logging
```

## Automatic Truncation

### Limits

MEMORY.md is automatically truncated at:
- **200 lines** - Primary limit
- **25KB** - Secondary limit (catches long-line edge cases)

### Truncation Algorithm

```rust
1. Check line count (primary)
2. If lines > 200, truncate to 200 lines
3. Check byte count (secondary)
4. If bytes > 25KB, find last newline before limit
5. Truncate at that newline (avoid mid-line cuts)
6. Append warning message
```

### Warning Message

```markdown
> WARNING: MEMORY.md is 215 lines (limit: 200). Only part of it was loaded. Keep index entries to one line under ~200 chars; move detail into topic files.
```

**Or for byte limit:**
```markdown
> WARNING: MEMORY.md is 28435 bytes (limit: 25000 bytes) — index entries are too long. Only part of it was loaded. Keep index entries to one line under ~200 chars; move detail into topic files.
```

## Operations

### Initialize Memdir

```rust
use ferroclaw::memory::Memdir;

// Use default path (~/.local/share/ferroclaw/memory/)
let memdir = Memdir::new()?;

// Use custom path
let custom_path = PathBuf::from("/custom/memory/path");
let memdir = Memdir::with_path(custom_path);

// Ensure directory exists
memdir.ensure_dir_exists()?;
```

### Read MEMORY.md

```rust
// Read without truncation
let content = memdir.read_entrypoint()?;

// Read with automatic truncation
let truncated = memdir.read_entrypoint_truncated()?;
println!("Content: {}", truncated.content);
println!("Lines: {}", truncated.line_count);
println!("Bytes: {}", truncated.byte_count);
println!("Line truncated: {}", truncated.was_line_truncated);
println!("Byte truncated: {}", truncated.was_byte_truncated);
```

### Write MEMORY.md

```rust
let content = r#"# Project Memory

## User Context
- Senior Rust developer
- Working on AI agent framework

## Project Overview
See project_context.md
"#;

// Write directly
std::fs::write(memdir.entrypoint_path(), content)?;

// Or use truncate helper
let truncated = memdir.truncate_entrypoint(content);
// truncated.content is safe to write
```

### Topic File Operations

#### Create/Update Topic

```rust
let topic_content = r#"# Project Context

## Architecture
- Agent loop with ReAct pattern
- SQLite-backed task system
- MCP protocol for tools

## Tech Stack
- Rust 2024 edition
- Tokio for async
- SQLite for persistence
"#;

memdir.write_topic_file("project_context", topic_content)?;
```

#### Read Topic

```rust
let content = memdir.load_topic_file("project_context")?;
println!("{}", content);
```

#### Check Topic Exists

```rust
if memdir.topic_file_exists("project_context") {
    println!("Topic file exists");
}
```

#### List Topics

```rust
let topics = memdir.list_topic_files()?;
// Returns: ["project_context", "user_role", "feedback_testing"]

for topic in topics {
    println!("Topic: {}", topic);
}
```

#### Delete Topic

```rust
let deleted = memdir.delete_topic_file("old_topic")?;
if deleted {
    println!("Topic deleted");
} else {
    println!("Topic didn't exist");
}
```

### Prompt Generation

Generate formatted memory prompt for LLM context:

```rust
let memory_prompt = memdir.load_memory_prompt()?;
```

**Output format:**
```markdown
# User Preferences and Project Context

## CONTEXT_SUMMARY
You are working with a senior Rust developer on an AI agent framework called Ferroclaw.

## MEMORY_CONTENTS
- User role and context: See user_role.md
- Project context: See project_context.md
- Testing preferences: See feedback_testing.md

## MEMORY
<Contents of MEMORY.md (truncated to 200 lines / 25KB)>

## TOPIC: user_role
<Contents of user_role.md>

## TOPIC: project_context
<Contents of project_context.md>

[... additional topic files ...]
```

## Integration with MemoryStore

### Complementary Systems

**MemoryStore (SQLite):**
- FTS5 full-text search
- Conversation history
- Structured key-value data
- Timestamp-based queries

**Memdir (File-based):**
- Long-form memory organization
- Topic-based categorization
- LLM prompt generation
- Human-readable, git-friendly

### When to Use Each

**Use MemoryStore for:**
```rust
// Quick key-value storage
store.insert("api_key", "sk-...")?;
let key = store.get("api_key")?;

// Search across all memory
let results = store.search("authentication", 10)?;

// Conversation history
store.save_conversation("user-123", &messages)?;
```

**Use Memdir for:**
```rust
// Long-form context
memdir.write_topic_file("project_architecture", long_architecture_doc)?;

// System prompt generation
let memory_prompt = memdir.load_memory_prompt()?;

// Persistent project memory
memdir.write_topic_file("lessons_learned", lessons_md)?;
```

### Combined Usage Example

```rust
use ferroclaw::memory::{MemoryStore, Memdir};

let store = MemoryStore::new(None)?;
let memdir = Memdir::new()?;

// Save quick fact to SQLite
store.insert("user_preference_theme", "dark")?;

// Save detailed context to file
memdir.write_topic_file("ui_preferences", r#"
# UI Preferences

## Theme
- Dark mode preferred
- High contrast for accessibility

## Font Sizes
- Code: 14pt
- Text: 12pt
"#)?;

// Generate memory prompt for LLM
let memory_prompt = memdir.load_memory_prompt()?;

// Search for specific facts
let theme = store.get("user_preference_theme")?;
```

## Usage Examples

### Example 1: Project Onboarding

```rust
use ferroclaw::memory::Memdir;

fn onboarding(memdir: &Memdir) -> Result<(), Box<dyn std::error::Error>> {
    // Create user role file
    memdir.write_topic_file("user_role", r#"
# User Role

## Background
Senior software engineer with 10 years experience.

## Expertise
- Rust, Python, TypeScript
- Distributed systems
- Machine learning

## Goals
- Build production-ready AI agent
- Maintain high code quality
- Follow best practices
"#)?;

    // Create project context file
    memdir.write_topic_file("project_context", r#"
# Project Context

## Overview
Ferroclaw is a security-first AI agent framework.

## Architecture
- Agent loop with ReAct pattern
- SQLite for persistence
- MCP protocol for extensibility

## Tech Stack
- Rust 2024
- Tokio async runtime
- Git2 for version control
"#)?;

    // Update MEMORY.md index
    memdir.write_topic_file(MEMORY.md, r#"
# Project Memory

## Quick Start
- User: See user_role.md
- Project: See project_context.md

## Commands
- Build: cargo build --release
- Test: cargo test
- Run: cargo run
"#)?;

    Ok(())
}
```

### Example 2: Feedback Tracking

```rust
fn save_feedback(memdir: &Memdir, feedback: &str) -> Result<()> {
    // Load existing feedback
    let existing = if memdir.topic_file_exists("feedback_history") {
        memdir.load_topic_file("feedback_history")?
    } else {
        String::from("# Feedback History\n\n")
    };

    // Append new feedback
    let timestamp = chrono::Utc::now().to_rfc3339();
    let new_feedback = format!("{}## {}\n\n{}\n\n", existing, timestamp, feedback);

    // Save updated feedback
    memdir.write_topic_file("feedback_history", &new_feedback)?;

    Ok(())
}
```

### Example 3: LLM Context Generation

```rust
async fn generate_response(memdir: &Memdir, user_query: &str) -> Result<String> {
    // Load memory prompt
    let memory_prompt = memdir.load_memory_prompt()?;

    // Combine with user query
    let full_prompt = format!(
        "{}\n\n## USER QUERY\n{}\n\n## RESPONSE",
        memory_prompt, user_query
    );

    // Send to LLM
    let response = llm_provider.generate(&full_prompt).await?;

    Ok(response)
}
```

## Best Practices

### 1. Keep MEMORY.md Concise

```markdown
# Good MEMORY.md
## User
See user_role.md

## Project
See project_context.md

# Bad MEMORY.md (violates best practices)
## User
John is a senior developer with 10 years of experience
working on distributed systems... [200 lines of detail]
```

### 2. Use Descriptive Topic Names

```rust
// Good
memdir.write_topic_file("project_architecture", ...)?;
memdir.write_topic_file("coding_conventions", ...)?;

// Bad
memdir.write_topic_file("stuff", ...)?;
memdir.write_topic_file("notes", ...)?;
```

### 3. Organize Topics Logically

```
memory/
├── MEMORY.md           # Index
├── user/               # User-related
│   ├── role.md
│   └── preferences.md
├── project/            # Project-related
│   ├── architecture.md
│   └── goals.md
└── history/            # Historical
    ├── feedback.md
    └── lessons.md
```

### 4. Update MEMORY.md When Adding Topics

```rust
// After creating new topic
memdir.write_topic_file("new_feature", content)?;

// Update MEMORY.md to reference it
let entrypoint = memdir.read_entrypoint()?;
let updated = format!("{}\n\n## New Feature\nSee new_feature.md", entrypoint);
memdir.write_topic_file(MEMORY.md, &updated)?;
```

### 5. Use Markdown Formatting

```markdown
# Topic Title

## Sections
Use ### for subsections

- Bullet points
- For lists

1. Numbered lists
2. For sequences

```code
Code blocks
```

**Bold** for emphasis
*Italic* for subtle points
```

## Troubleshooting

### Issue: MEMORY.md keeps getting truncated

**Cause**: Too much detail in index file

**Solution**: Move detail to topic files:
```rust
// Before (MEMORY.md too large)
let content = r#"
# Memory

## User
John is a senior developer with 10 years experience...
[50 lines of detail]

## Project
Ferroclaw is an AI agent framework...
[100 lines of detail]
"#;

// After (concise index)
let content = r#"
# Memory

## User
See user_role.md

## Project
See project_context.md
"#;

memdir.write_topic_file("user_role", user_detail)?;
memdir.write_topic_file("project_context", project_detail)?;
memdir.write_topic_file(MEMORY.md, content)?;
```

### Issue: Topic file not found

**Cause**: File doesn't exist or wrong topic name

**Solution**:
```rust
// Check if exists first
if !memdir.topic_file_exists("my_topic") {
    // Create it
    memdir.write_topic_file("my_topic", "# My Topic\n\nInitial content")?;
}

// List available topics
let topics = memdir.list_topic_files()?;
println!("Available topics: {:?}", topics);
```

### Issue: Memory prompt too long

**Cause**: Too many topic files or very large topics

**Solution**:
```rust
// Select only relevant topics
let memory_prompt = memdir.load_memory_prompt()?;

// Or load specific topics manually
let user_context = memdir.load_topic_file("user_role")?;
let project_context = memdir.load_topic_file("project_context")?;
let custom_prompt = format!("{}\n\n{}", user_context, project_context);
```

## See Also

- **MemoryStore**: SQLite-based searchable memory
- **TaskSystem**: Persistent task management
- **PlanMode**: Structured workflow management
