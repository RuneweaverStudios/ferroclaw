//! Comprehensive tests for the hook system.

use crate::hooks::*;
use crate::types::{Capability, ToolCall, ToolResult};
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, RwLock};

/// Test hook that can be configured to return specific results.
#[derive(Clone)]
struct TestHook {
    pre_tool_result: Arc<std::sync::Mutex<Option<HookResult>>>,
    post_tool_result: Arc<std::sync::Mutex<Option<HookResult>>>,
    permission_result: Arc<std::sync::Mutex<Option<HookResult>>>,
    config_change_called: Arc<AtomicBool>,
    session_start_called: Arc<AtomicBool>,
    session_end_called: Arc<AtomicBool>,
    call_count: Arc<AtomicI32>,
}

impl TestHook {
    fn new() -> Self {
        Self {
            pre_tool_result: Arc::new(std::sync::Mutex::new(None)),
            post_tool_result: Arc::new(std::sync::Mutex::new(None)),
            permission_result: Arc::new(std::sync::Mutex::new(None)),
            config_change_called: Arc::new(AtomicBool::new(false)),
            session_start_called: Arc::new(AtomicBool::new(false)),
            session_end_called: Arc::new(AtomicBool::new(false)),
            call_count: Arc::new(AtomicI32::new(0)),
        }
    }

    fn set_pre_tool_result(&self, result: HookResult) {
        *self.pre_tool_result.lock().unwrap() = Some(result);
    }

    fn set_post_tool_result(&self, result: HookResult) {
        *self.post_tool_result.lock().unwrap() = Some(result);
    }

    fn set_permission_result(&self, result: HookResult) {
        *self.permission_result.lock().unwrap() = Some(result);
    }

    fn was_config_change_called(&self) -> bool {
        self.config_change_called.load(Ordering::Relaxed)
    }

    fn was_session_start_called(&self) -> bool {
        self.session_start_called.load(Ordering::Relaxed)
    }

    fn was_session_end_called(&self) -> bool {
        self.session_end_called.load(Ordering::Relaxed)
    }

    fn call_count(&self) -> i32 {
        self.call_count.load(Ordering::Relaxed)
    }
}

impl Hook for TestHook {
    fn pre_tool(&self, _ctx: &HookContext, _call: &ToolCall) -> HookResult {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        self.pre_tool_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(HookResult::Continue)
    }

    fn post_tool(&self, _ctx: &HookContext, _call: &ToolCall, _result: &ToolResult) -> HookResult {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        self.post_tool_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(HookResult::Continue)
    }

    fn permission_check(
        &self,
        _ctx: &HookContext,
        _tool_name: &str,
        _required_caps: &[Capability],
    ) -> HookResult {
        self.permission_result
            .lock()
            .unwrap()
            .clone()
            .unwrap_or(HookResult::Continue)
    }

    fn config_change(&self, _ctx: &HookContext, _config_key: &str) {
        self.config_change_called.store(true, Ordering::Relaxed);
    }

    fn session_start(&self, _ctx: &HookContext) {
        self.session_start_called.store(true, Ordering::Relaxed);
    }

    fn session_end(&self, _ctx: &HookContext) {
        self.session_end_called.store(true, Ordering::Relaxed);
    }
}

#[test]
fn test_hook_registration_and_execution_order() {
    let call_order = Arc::new(RwLock::new(Vec::new()));

    #[derive(Clone)]
    struct OrderedHook {
        name: String,
        call_order: Arc<RwLock<Vec<String>>>,
    }

    impl Hook for OrderedHook {
        fn pre_tool(&self, _ctx: &HookContext, _call: &ToolCall) -> HookResult {
            self.call_order.write().unwrap().push(self.name.clone());
            HookResult::Continue
        }
    }

    let manager = HookManager::new();
    manager.register(Box::new(OrderedHook {
        name: "first".to_string(),
        call_order: call_order.clone(),
    }));
    manager.register(Box::new(OrderedHook {
        name: "second".to_string(),
        call_order: call_order.clone(),
    }));
    manager.register(Box::new(OrderedHook {
        name: "third".to_string(),
        call_order: call_order.clone(),
    }));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    manager.execute_pre_tool(&ctx, &call).unwrap();

    let order = call_order.read().unwrap();
    assert_eq!(order.as_slice(), &["first", "second", "third"]);
}

#[test]
fn test_pre_tool_hook_continue() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    manager.register(Box::new(hook.clone()));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({"original": "args"}),
    };

    let result = manager.execute_pre_tool(&ctx, &call).unwrap();
    assert_eq!(result, json!({"original": "args"}));
    assert_eq!(hook.call_count(), 1);
}

#[test]
fn test_pre_tool_hook_modify_arguments() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_pre_tool_result(HookResult::ModifyArguments(json!({"modified": "args"})));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({"original": "args"}),
    };

    let result = manager.execute_pre_tool(&ctx, &call).unwrap();
    assert_eq!(result, json!({"modified": "args"}));
}

#[test]
fn test_pre_tool_hook_halt() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_pre_tool_result(HookResult::Halt("Access denied".to_string()));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    let result = manager.execute_pre_tool(&ctx, &call);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Access denied"));
}

#[test]
fn test_post_tool_hook_continue() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    manager.register(Box::new(hook.clone()));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };
    let result = ToolResult {
        call_id: "call-1".to_string(),
        content: "original result".to_string(),
        is_error: false,
    };

    let final_result = manager.execute_post_tool(&ctx, &call, &result).unwrap();
    assert_eq!(final_result.content, "original result");
    assert_eq!(hook.call_count(), 1);
}

#[test]
fn test_post_tool_hook_modify_result() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_post_tool_result(HookResult::ModifyResult(ToolResult {
        call_id: "call-1".to_string(),
        content: "modified result".to_string(),
        is_error: false,
    }));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };
    let result = ToolResult {
        call_id: "call-1".to_string(),
        content: "original result".to_string(),
        is_error: false,
    };

    let final_result = manager.execute_post_tool(&ctx, &call, &result).unwrap();
    assert_eq!(final_result.content, "modified result");
}

#[test]
fn test_post_tool_hook_halt() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_post_tool_result(HookResult::Halt("Result suppressed".to_string()));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };
    let result = ToolResult {
        call_id: "call-1".to_string(),
        content: "result".to_string(),
        is_error: false,
    };

    let final_result = manager.execute_post_tool(&ctx, &call, &result);
    assert!(final_result.is_err());
    assert!(
        final_result
            .unwrap_err()
            .to_string()
            .contains("Result suppressed")
    );
}

#[test]
fn test_permission_check_hook_allow() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_permission_result(HookResult::Halt("allow".to_string()));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let result = manager
        .execute_permission_check(&ctx, "test_tool", &[Capability::FsRead])
        .unwrap();

    assert!(result, "Hook should have allowed the operation");
}

#[test]
fn test_permission_check_hook_deny() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_permission_result(HookResult::Halt("Access denied".to_string()));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let result = manager.execute_permission_check(&ctx, "test_tool", &[Capability::FsRead]);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Access denied"));
}

#[test]
fn test_permission_check_hook_continue() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let result = manager
        .execute_permission_check(&ctx, "test_tool", &[Capability::FsRead])
        .unwrap();

    assert!(!result, "Hook should have deferred to default check");
}

#[test]
fn test_config_change_hook() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    manager.register(Box::new(hook.clone()));

    let ctx = HookContext::new("test");
    manager.execute_config_change(&ctx, "test_key");

    assert!(hook.was_config_change_called());
}

#[test]
fn test_session_start_hook() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    manager.register(Box::new(hook.clone()));

    let ctx = HookContext::new("test");
    manager.execute_session_start(&ctx);

    assert!(hook.was_session_start_called());
}

#[test]
fn test_session_end_hook() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    manager.register(Box::new(hook.clone()));

    let ctx = HookContext::new("test");
    manager.execute_session_end(&ctx);

    assert!(hook.was_session_end_called());
}

#[test]
fn test_hook_manager_clear() {
    let manager = HookManager::new();
    manager.register(Box::new(TestHook::new()));
    manager.register(Box::new(TestHook::new()));

    assert_eq!(manager.len(), 2);

    manager.clear();

    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

#[test]
fn test_multiple_hooks_execution() {
    let manager = HookManager::new();
    let hook1 = TestHook::new();
    let hook2 = TestHook::new();
    let hook3 = TestHook::new();

    manager.register(Box::new(hook1.clone()));
    manager.register(Box::new(hook2.clone()));
    manager.register(Box::new(hook3.clone()));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    manager.execute_pre_tool(&ctx, &call).unwrap();

    // All three hooks should have been called
    assert_eq!(hook1.call_count(), 1);
    assert_eq!(hook2.call_count(), 1);
    assert_eq!(hook3.call_count(), 1);
}

#[test]
fn test_hook_halts_subsequent_hooks() {
    let manager = HookManager::new();
    let hook1 = TestHook::new();
    let hook2 = TestHook::new();
    let hook3 = TestHook::new();

    // Second hook halts execution
    hook2.set_pre_tool_result(HookResult::Halt("Stop here".to_string()));

    manager.register(Box::new(hook1.clone()));
    manager.register(Box::new(hook2.clone()));
    manager.register(Box::new(hook3.clone()));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    let result = manager.execute_pre_tool(&ctx, &call);
    assert!(result.is_err());

    // First two hooks should have been called, but not the third
    assert_eq!(hook1.call_count(), 1);
    assert_eq!(hook2.call_count(), 1);
    assert_eq!(hook3.call_count(), 0);
}

#[test]
fn test_hook_context_with_metadata() {
    let ctx = HookContext::new("session-123")
        .with_metadata("user_id", "user-456")
        .with_metadata("channel", "slack")
        .with_metadata("ip", "192.168.1.1");

    assert_eq!(ctx.session_id, "session-123");
    assert_eq!(ctx.metadata.len(), 3);
    assert_eq!(ctx.metadata.get("user_id"), Some(&"user-456".to_string()));
    assert_eq!(ctx.metadata.get("channel"), Some(&"slack".to_string()));
    assert_eq!(ctx.metadata.get("ip"), Some(&"192.168.1.1".to_string()));
}

#[test]
fn test_hook_result_should_continue() {
    assert!(HookResult::Continue.should_continue());
    assert!(HookResult::ModifyArguments(json!({})).should_continue());
    assert!(
        HookResult::ModifyResult(ToolResult {
            call_id: "test".to_string(),
            content: "test".to_string(),
            is_error: false,
        })
        .should_continue()
    );
    assert!(!HookResult::Halt("error".to_string()).should_continue());
}

#[test]
fn test_hook_result_error_message() {
    assert!(HookResult::Continue.error_message().is_none());
    assert!(
        HookResult::Halt("Access denied".to_string())
            .error_message()
            .is_some()
    );
    assert_eq!(
        HookResult::Halt("Access denied".to_string()).error_message(),
        Some("Access denied")
    );
}

#[test]
fn test_hook_manager_default() {
    let manager = HookManager::default();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_hook_context_timestamp() {
    let before = chrono::Utc::now();
    let ctx = HookContext::new("test");
    let after = chrono::Utc::now();

    assert!(ctx.timestamp >= before);
    assert!(ctx.timestamp <= after);
}

#[test]
fn test_multiple_permission_hooks() {
    let manager = HookManager::new();
    let hook1 = TestHook::new();
    let hook2 = TestHook::new();
    let hook3 = TestHook::new();

    // First hook continues, second allows, third should not be called
    hook2.set_permission_result(HookResult::Halt("allow".to_string()));

    manager.register(Box::new(hook1));
    manager.register(Box::new(hook2));
    manager.register(Box::new(hook3));

    let ctx = HookContext::new("test");
    let result = manager
        .execute_permission_check(&ctx, "test_tool", &[Capability::FsRead])
        .unwrap();

    assert!(result, "Should be allowed by second hook");
}

#[test]
fn test_invalid_hook_result_in_pre_tool() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_pre_tool_result(HookResult::ModifyResult(ToolResult {
        call_id: "test".to_string(),
        content: "test".to_string(),
        is_error: false,
    }));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    let result = manager.execute_pre_tool(&ctx, &call);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("ModifyResult is not valid")
    );
}

#[test]
fn test_invalid_hook_result_in_post_tool() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_post_tool_result(HookResult::ModifyArguments(json!({})));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };
    let result = ToolResult {
        call_id: "call-1".to_string(),
        content: "test".to_string(),
        is_error: false,
    };

    let final_result = manager.execute_post_tool(&ctx, &call, &result);
    assert!(final_result.is_err());
    assert!(
        final_result
            .unwrap_err()
            .to_string()
            .contains("ModifyArguments is not valid")
    );
}

#[test]
fn test_invalid_hook_result_in_permission_check() {
    let manager = HookManager::new();
    let hook = TestHook::new();
    hook.set_permission_result(HookResult::ModifyArguments(json!({})));
    manager.register(Box::new(hook));

    let ctx = HookContext::new("test");
    let result = manager.execute_permission_check(&ctx, "test_tool", &[Capability::FsRead]);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Modify operations are not valid")
    );
}

#[test]
fn test_hook_execution_isolation() {
    // Test that hooks don't interfere with each other
    let manager = HookManager::new();

    #[derive(Clone)]
    struct IsolationHook {
        value: Arc<std::sync::Mutex<i32>>,
    }

    impl Hook for IsolationHook {
        fn pre_tool(&self, _ctx: &HookContext, _call: &ToolCall) -> HookResult {
            let mut value = self.value.lock().unwrap();
            *value += 1;
            HookResult::Continue
        }
    }

    let value1 = Arc::new(std::sync::Mutex::new(0));
    let value2 = Arc::new(std::sync::Mutex::new(0));

    manager.register(Box::new(IsolationHook {
        value: value1.clone(),
    }));
    manager.register(Box::new(IsolationHook {
        value: value2.clone(),
    }));

    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    manager.execute_pre_tool(&ctx, &call).unwrap();

    assert_eq!(*value1.lock().unwrap(), 1);
    assert_eq!(*value2.lock().unwrap(), 1);
}

#[test]
fn test_hook_manager_thread_safety() {
    // Test that HookManager can be safely cloned and used across threads
    let manager = HookManager::new();
    manager.register(Box::new(TestHook::new()));

    let manager_clone = manager.clone();
    let ctx = HookContext::new("test");
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "test_tool".to_string(),
        arguments: json!({}),
    };

    // Spawn a thread and execute hooks
    let handle = std::thread::spawn(move || manager_clone.execute_pre_tool(&ctx, &call).unwrap());

    let result = handle.join().unwrap();
    assert_eq!(result, json!({}));
}
