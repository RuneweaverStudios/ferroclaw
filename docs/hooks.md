# HookSystem - Extensibility Through Event Hooks

## Overview

HookSystem provides an event-driven extensibility framework that allows intercepting and modifying tool execution, permission checks, and session lifecycle events. Hooks enable cross-cutting concerns without modifying core code.

**Key Features:**
- Six lifecycle hook points
- Control flow modification (halt, modify args/results)
- Five built-in hooks
- Thread-safe concurrent execution
- Zero overhead when no hooks registered

## Hook Lifecycle Methods

### 1. pre_tool

**Called before**: Tool execution, capability checks

**Use cases**:
- Input validation
- Argument transformation
- Permission override
- Request logging

**Signature**:
```rust
fn pre_tool(&self, context: &HookContext, tool: &str, args: &Value) -> HookResult
```

**Example**:
```rust
fn pre_tool(&self, ctx: &HookContext, tool: &str, args: &Value) -> HookResult {
    if tool == "dangerous_operation" {
        return HookResult::Halt("Dangerous operation not allowed".into());
    }

    // Transform arguments
    if let Some(modified) = self.validate_args(args) {
        return HookResult::ModifyArguments(modified);
    }

    HookResult::Continue
}
```

### 2. post_tool

**Called after**: Tool execution completes

**Use cases**:
- Result logging/auditing
- Result transformation
- Metrics collection
- Response caching

**Signature**:
```rust
fn post_tool(&self, context: &HookContext, tool: &str, result: &ToolResult) -> HookResult
```

**Example**:
```rust
fn post_tool(&self, ctx: &HookContext, tool: &str, result: &ToolResult) -> HookResult {
    // Log all tool calls
    println!("Tool {} returned: {}", tool, result.content);

    // Transform result if needed
    if let Some(modified) = self.transform_result(result) {
        return HookResult::ModifyResult(modified);
    }

    HookResult::Continue
}
```

### 3. permission_check

**Called during**: Capability verification

**Use cases**:
- Dynamic permission grants
- User-specific overrides
- Time-based restrictions
- Context-aware authorization

**Signature**:
```rust
fn permission_check(&self, context: &HookContext, capability: Capability) -> HookResult
```

**Example**:
```rust
fn permission_check(&self, ctx: &HookContext, cap: Capability) -> HookResult {
    // Grant admin users all capabilities
    if ctx.user_id == "admin" {
        return HookResult::Continue; // Allow
    }

    // Deny dangerous operations during business hours
    if cap == Capability::ProcessExec && self.is_business_hours() {
        return HookResult::Halt("Process execution not allowed during business hours".into());
    }

    HookResult::Continue
}
```

### 4. config_change

**Called when**: Configuration is reloaded or modified

**Use cases**:
- Hot reload configuration
- Validate new config
- Update internal state
- Notify dependent systems

**Signature**:
```rust
fn config_change(&self, context: &HookContext, key: &str, value: &Value)
```

**Example**:
```rust
fn config_change(&self, ctx: &HookContext, key: &str, value: &Value) {
    if key == "log_level" {
        self.set_log_level(value);
    }
}
```

### 5. session_start

**Called when**: New session begins

**Use cases**:
- Session initialization
- Resource allocation
- Logging/tracing setup
- User authentication

**Signature**:
```rust
fn session_start(&self, context: &HookContext)
```

**Example**:
```rust
fn session_start(&self, ctx: &HookContext) {
    println!("Session started: {}", ctx.session_id);
    self.initialize_session_resources(ctx.session_id);
}
```

### 6. session_end

**Called when**: Session terminates

**Use cases**:
- Resource cleanup
- Session persistence
- Final logging
- Statistics aggregation

**Signature**:
```rust
fn session_end(&self, context: &HookContext)
```

**Example**:
```rust
fn session_end(&self, ctx: &HookContext) {
    println!("Session ended: {}", ctx.session_id);
    self.cleanup_session_resources(ctx.session_id);
    self.save_session_statistics(ctx.session_id);
}
```

## HookResult Control Flow

### Continue

Proceed with the operation.

```rust
HookResult::Continue
```

**Flow**: Operation proceeds normally.

### Halt

Stop execution with error message.

```rust
HookResult::Halt("Operation not allowed".into())
```

**Flow**: Operation stops, error returned to caller.

### ModifyArguments

Change tool input arguments.

```rust
HookResult::ModifyArguments(json!({"param": "modified_value"}))
```

**Flow**: Tool executes with modified arguments.

### ModifyResult

Change tool output result.

```rust
HookResult::ModifyResult(ToolResult {
    call_id: result.call_id,
    content: "Modified content".into(),
    is_error: false,
})
```

**Flow**: Modified result returned to caller.

## HookContext

Runtime information provided to all hooks:

```rust
pub struct HookContext {
    pub session_id: String,
    pub user_id: String,
    pub channel_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}
```

**Fields**:
- `session_id`: Unique session identifier
- `user_id`: User or agent ID
- `channel_id`: Channel (cli, telegram, etc.)
- `timestamp`: Hook execution time
- `metadata`: Additional context

## Built-in Hooks

### 1. LoggingHook

Logs all tool calls and results.

**Features**:
- Configurable argument/result inclusion
- Session lifecycle logging
- Timestamp tracking

**Usage**:
```rust
use ferroclaw::hooks::builtin::LoggingHook;

// Log with arguments but not results
let hook = LoggingHook::new(true, false);
registry.hooks().register(Box::new(hook));
```

**Configuration**:
```rust
LoggingHook::new(
    include_arguments: bool,  // Include tool arguments in logs
    include_results: bool,    // Include tool results in logs
)
```

### 2. AuditHook

In-memory audit log of all tool executions.

**Features**:
- Tracks timestamp, session, tool, success/error
- Queryable log snapshots
- Clear and count operations

**Usage**:
```rust
use ferroclaw::hooks::builtin::AuditHook;

let hook = AuditHook::new();
registry.hooks().register(Box::new(hook));

// Query audit log
let log = hook.get_log();
for entry in log {
    println!("{}: {} - {}", entry.timestamp, entry.tool, entry.success);
}
```

### 3. RateLimitHook

Per-session rate limiting.

**Features**:
- Configurable max calls per time window
- Automatic cleanup on session end
- Call count tracking

**Usage**:
```rust
use ferroclaw::hooks::builtin::RateLimitHook;
use std::time::Duration;

// Limit to 10 calls per minute
let hook = RateLimitHook::new(10, Duration::from_secs(60));
registry.hooks().register(Box::new(hook));
```

### 4. SecurityHook

Tool allowlist/denylist and capability overrides.

**Features**:
- Tool allowlist/denylist
- User-specific capability overrides
- Dynamic capability grants/revokes

**Usage**:
```rust
use ferroclaw::hooks::builtin::SecurityHook;
use ferroclaw::security::Capability;

let mut hook = SecurityHook::new();

// Deny dangerous tools
hook.deny_tool("dangerous_operation");

// Grant specific capability to user
hook.grant_capability("user-123", Capability::ProcessExec);

registry.hooks().register(Box::new(hook));
```

### 5. MetricsHook

Tracks usage metrics.

**Features**:
- Total tool calls counter
- Total errors counter
- Thread-safe atomic operations
- Reset capability

**Usage**:
```rust
use ferroclaw::hooks::builtin::MetricsHook;

let hook = MetricsHook::new();
registry.hooks().register(Box::new(hook));

// Get metrics
println!("Total calls: {}", hook.total_calls());
println!("Total errors: {}", hook.total_errors());

// Reset metrics
hook.reset();
```

## Custom Hook Implementation

### Example: Caching Hook

```rust
use ferroclaw::hooks::{Hook, HookContext, HookResult};
use ferroclaw::types::{ToolResult, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CachingHook {
    cache: Arc<Mutex<HashMap<String, ToolResult>>>,
}

impl CachingHook {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn cache_key(&self, tool: &str, args: &Value) -> String {
        format!("{}:{}", tool, args)
    }
}

impl Hook for CachingHook {
    fn pre_tool(&self, _ctx: &HookContext, tool: &str, args: &Value) -> HookResult {
        let key = self.cache_key(tool, args);

        // Check cache
        if let Some(cached) = self.cache.lock().unwrap().get(&key) {
            return HookResult::ModifyResult(cached.clone());
        }

        HookResult::Continue
    }

    fn post_tool(&self, _ctx: &HookContext, tool: &str, args: &Value, result: &ToolResult) -> HookResult {
        // Cache successful results
        if !result.is_error {
            let key = self.cache_key(tool, args);
            self.cache.lock().unwrap().insert(key, result.clone());
        }

        HookResult::Continue
    }
}

// Register
let hook = CachingHook::new();
registry.hooks().register(Box::new(hook));
```

### Example: Validation Hook

```rust
pub struct ValidationHook;

impl Hook for ValidationHook {
    fn pre_tool(&self, _ctx: &HookContext, tool: &str, args: &Value) -> HookResult {
        // Validate file paths are absolute
        if let Some(path) = args.get("file_path") {
            if let Some(path_str) = path.as_str() {
                if !path_str.starts_with('/') {
                    return HookResult::Halt(format!(
                        "File path must be absolute: {}", path_str
                    ));
                }
            }
        }

        HookResult::Continue
    }
}
```

## Usage Examples

### Example 1: Register Multiple Hooks

```rust
use ferroclaw::tool::ToolRegistry;
use ferroclaw::hooks::builtin::{LoggingHook, AuditHook, MetricsHook};

let registry = ToolRegistry::new();

// Register multiple hooks
registry.hooks().register(Box::new(LoggingHook::new(true, false)));
registry.hooks().register(Box::new(AuditHook::new()));
registry.hooks().register(Box::new(MetricsHook::new()));

// Hooks execute in registration order
```

### Example 2: Conditional Hook Execution

```rust
pub struct ConditionalHook {
    enabled_tools: Vec<String>,
}

impl Hook for ConditionalHook {
    fn pre_tool(&self, _ctx: &HookContext, tool: &str, _args: &Value) -> HookResult {
        // Only run for specific tools
        if self.enabled_tools.contains(&tool.to_string()) {
            // Custom logic here
        }

        HookResult::Continue
    }
}
```

### Example 3: Session-Based State

```rust
pub struct SessionStateHook {
    state: Arc<Mutex<HashMap<String, SessionData>>>,
}

impl Hook for SessionStateHook {
    fn session_start(&self, ctx: &HookContext) {
        let mut state = self.state.lock().unwrap();
        state.insert(ctx.session_id.clone(), SessionData::new());
    }

    fn session_end(&self, ctx: &HookContext) {
        let mut state = self.state.lock().unwrap();
        state.remove(&ctx.session_id);
    }
}
```

## Best Practices

### 1. Keep Hooks Focused

```rust
// Good: Single responsibility
pub struct LoggingHook;
pub struct AuditHook;

// Bad: Multiple responsibilities
pub struct MegaHook {
    // Logging + auditing + metrics + ...
}
```

### 2. Use HookResult Correctly

```rust
// Good: Explicit control flow
if should_deny {
    return HookResult::Halt("Not allowed".into());
}
HookResult::Continue

// Bad: Silent failures
if should_deny {
    // Do nothing, let it proceed
}
HookResult::Continue
```

### 3. Handle Errors Gracefully

```rust
fn pre_tool(&self, ctx: &HookContext, tool: &str, args: &Value) -> HookResult {
    match self.validate(args) {
        Ok(_) => HookResult::Continue,
        Err(e) => HookResult::Halt(format!("Validation failed: {}", e)),
    }
}
```

### 4. Consider Performance

```rust
// Good: Minimal overhead in hook
fn pre_tool(&self, _ctx: &HookContext, _tool: &str, _args: &Value) -> HookResult {
    if !self.enabled {
        return HookResult::Continue;
    }
    // Expensive logic here
}

// Bad: Always expensive
fn pre_tool(&self, _ctx: &HookContext, _tool: &str, _args: &Value) -> HookResult {
    let expensive_result = expensive_computation();
    // ...
}
```

### 5. Use Thread-Safe Patterns

```rust
use std::sync::{Arc, Mutex};

pub struct ThreadSafeHook {
    state: Arc<Mutex<State>>,
}

impl Hook for ThreadSafeHook {
    fn pre_tool(&self, _ctx: &HookContext, _tool: &str, _args: &Value) -> HookResult {
        let mut state = self.state.lock().unwrap();
        // Access state safely
    }
}
```

## Performance Considerations

### Execution Order

Hooks execute in registration order:

```rust
registry.hooks().register(Box::new(hook1)); // Runs first
registry.hooks().register(Box::new(hook2)); // Runs second
```

### Early Exit

Any `Halt` result stops execution immediately:

```rust
// Hook 1: Checks auth
HookResult::Halt("Not authenticated".into()) // Stops here

// Hook 2: Never reached
HookResult::Continue
```

### Zero Overhead

When no hooks registered, overhead is negligible:

```rust
// No hooks = minimal overhead
let result = registry.execute(tool, args).await?;
```

### Benchmark Results

- Hook registration: ~100ns
- Hook execution (empty): ~50ns per hook
- Hook execution (with logic): varies by implementation

## Thread Safety

All hooks must be thread-safe:

```rust
// Good: Arc + Mutex
pub struct SafeHook {
    data: Arc<Mutex<Vec<String>>>,
}

// Bad: Unsync mutable state
pub struct UnsafeHook {
    data: Vec<String>, // !Sync
}
```

**Thread-safe built-in hooks**:
- LoggingHook: ✅ Thread-safe
- AuditHook: ✅ Thread-safe
- RateLimitHook: ✅ Thread-safe
- SecurityHook: ✅ Thread-safe
- MetricsHook: ✅ Thread-safe

## Troubleshooting

### Issue: Hook not executing

**Cause**: Hook not registered or wrong lifecycle method

**Solution**:
```rust
// Ensure hook is registered
registry.hooks().register(Box::new(my_hook));

// Check you're implementing the right method
impl Hook for MyHook {
    fn pre_tool(&self, ...) { } // Not post_tool
}
```

### Issue: Hook causes deadlock

**Cause**: Mutex lock ordering or recursive hook calls

**Solution**:
```rust
// Avoid locking in hooks that might trigger other hooks
fn pre_tool(&self, ctx: &HookContext, ...) -> HookResult {
    // Don't call registry.execute() here - might deadlock
    HookResult::Continue
}
```

### Issue: Hook slows down execution

**Cause**: Expensive operations in hot path

**Solution**:
```rust
fn pre_tool(&self, ctx: &HookContext, tool: &str, args: &Value) -> HookResult {
    // Cache expensive results
    if let Some(cached) = self.cache.get(&(tool, args)) {
        return *cached;
    }

    // Or use async for I/O
    // But note: hooks are synchronous currently
}
```

## See Also

- **ToolRegistry**: Tool execution and hook integration
- **SecuritySystem**: Capability checks
- **AgentLoop**: Hook integration in agent workflow
