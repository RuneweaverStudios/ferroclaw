//! Demonstration of the HookSystem for Ferroclaw.
//!
//! This example shows how to use hooks for logging, auditing, rate limiting,
//! and custom behavior modification.

use ferroclaw::hooks::builtin::{AuditHook, LoggingHook, MetricsHook, RateLimitHook};
use ferroclaw::hooks::{Hook, HookContext, HookResult};
use ferroclaw::types::{Capability, ToolCall};
use serde_json::json;

/// Custom hook that adds a timestamp to all tool arguments.
#[derive(Clone)]
struct TimestampHook;

impl Hook for TimestampHook {
    fn pre_tool(&self, _ctx: &HookContext, call: &ToolCall) -> HookResult {
        let mut args = call.arguments.clone();
        if let Some(obj) = args.as_object_mut() {
            obj.insert(
                "_timestamp".to_string(),
                json!(chrono::Utc::now().to_rfc3339()),
            );
        }
        HookResult::ModifyArguments(args)
    }
}

/// Custom hook that blocks tools with "delete" in the name.
#[derive(Clone)]
struct SafetyHook;

impl Hook for SafetyHook {
    fn permission_check(
        &self,
        _ctx: &HookContext,
        tool_name: &str,
        _required_caps: &[Capability],
    ) -> HookResult {
        if tool_name.to_lowercase().contains("delete") {
            return HookResult::Halt(
                "For safety, tools with 'delete' in their name are blocked".to_string(),
            );
        }
        HookResult::Continue
    }
}

fn main() {
    println!("=== Ferroclaw HookSystem Demo ===\n");

    // Create a hook manager
    let manager = ferroclaw::hooks::HookManager::new();

    // Register built-in hooks
    println!("Registering built-in hooks...");
    manager.register(Box::new(LoggingHook::new(true, false)));
    manager.register(Box::new(AuditHook::new()));
    manager.register(Box::new(RateLimitHook::new(5, 60)));
    manager.register(Box::new(MetricsHook::new()));

    // Register custom hooks
    println!("Registering custom hooks...");
    manager.register(Box::new(TimestampHook));
    manager.register(Box::new(SafetyHook));

    println!("\nTotal hooks registered: {}\n", manager.len());

    // Create test context
    let ctx = HookContext::new("demo-session")
        .with_metadata("user", "demo-user")
        .with_metadata("environment", "demo");

    // Test 1: Normal tool call
    println!("--- Test 1: Normal Tool Call ---");
    let call1 = ToolCall {
        id: "call-1".to_string(),
        name: "read_file".to_string(),
        arguments: json!({"path": "/tmp/file.txt"}),
    };

    match manager.execute_pre_tool(&ctx, &call1) {
        Ok(modified_args) => {
            println!("✓ Tool allowed with modified args: {}", modified_args);
        }
        Err(e) => {
            println!("✗ Tool blocked: {}", e);
        }
    }

    // Test 2: Blocked tool call
    println!("\n--- Test 2: Blocked Tool Call ---");
    let call2 = ToolCall {
        id: "call-2".to_string(),
        name: "delete_file".to_string(),
        arguments: json!({"path": "/tmp/file.txt"}),
    };

    match manager.execute_permission_check(&ctx, &call2.name, &[Capability::FsWrite]) {
        Ok(true) => println!("✓ Tool explicitly allowed by hook"),
        Ok(false) => println!("✓ Tool deferred to default checks"),
        Err(e) => println!("✗ Tool blocked by hook: {}", e),
    }

    // Test 3: Rate limiting
    println!("\n--- Test 3: Rate Limiting ---");
    let call3 = ToolCall {
        id: "call-3".to_string(),
        name: "fast_tool".to_string(),
        arguments: json!({}),
    };

    for i in 1..=7 {
        match manager.execute_pre_tool(&ctx, &call3) {
            Ok(_) => println!("Call {}: ✓", i),
            Err(e) => {
                println!("Call {}: ✗ {}", i, e);
                break;
            }
        }
    }

    // Test 4: Session lifecycle
    println!("\n--- Test 4: Session Lifecycle ---");
    manager.execute_session_start(&ctx);
    println!("Session started");

    // Do some work...
    println!("Performing work...");

    manager.execute_session_end(&ctx);
    println!("Session ended");

    // Test 5: Metrics
    println!("\n--- Test 5: Metrics ---");
    // The MetricsHook tracks all tool calls
    // In a real scenario, you'd retrieve the hook and check metrics
    println!("Metrics are being tracked by MetricsHook");

    println!("\n=== Demo Complete ===");
}
