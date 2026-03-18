//! Integration tests for the security subsystem:
//! capability enforcement, audit log integrity, and gateway safety.

use ferroclaw::security::audit::AuditLog;
use ferroclaw::security::capabilities::check_with_message;
use ferroclaw::tool::ToolRegistry;
use ferroclaw::tools::builtin::register_builtin_tools;
use ferroclaw::types::{Capability, CapabilitySet};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

// ── Capability Enforcement ──────────────────────────────────────────

#[tokio::test]
async fn test_builtin_tools_respect_capabilities() {
    let memory = ferroclaw::memory::MemoryStore::in_memory().unwrap();
    let memory = Arc::new(Mutex::new(memory));
    let mut registry = ToolRegistry::new();
    register_builtin_tools(&mut registry, memory);

    // read_file requires FsRead — should succeed with FsRead
    let read_caps = CapabilitySet::new([Capability::FsRead]);
    let result = registry
        .execute(
            "read_file",
            "tc_1",
            &serde_json::json!({"path": "/nonexistent"}),
            &read_caps,
        )
        .await;
    // Should get through capability check (file doesn't exist, but that's a tool error, not a capability error)
    assert!(result.is_ok());

    // write_file requires FsWrite — should fail with only FsRead
    let result = registry
        .execute(
            "write_file",
            "tc_2",
            &serde_json::json!({"path": "/tmp/test", "content": "x"}),
            &read_caps,
        )
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Capability denied"), "Expected capability denial, got: {err}");
}

#[tokio::test]
async fn test_bash_requires_process_exec() {
    let memory = ferroclaw::memory::MemoryStore::in_memory().unwrap();
    let memory = Arc::new(Mutex::new(memory));
    let mut registry = ToolRegistry::new();
    register_builtin_tools(&mut registry, memory);

    // Attempt bash with only FsRead — must be denied
    let safe_caps = CapabilitySet::new([Capability::FsRead, Capability::NetOutbound]);
    let result = registry
        .execute(
            "bash",
            "tc_3",
            &serde_json::json!({"command": "echo hello"}),
            &safe_caps,
        )
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Capability denied"));

    // With ProcessExec — should succeed
    let exec_caps = CapabilitySet::new([Capability::ProcessExec]);
    let result = registry
        .execute(
            "bash",
            "tc_4",
            &serde_json::json!({"command": "echo hello"}),
            &exec_caps,
        )
        .await;
    assert!(result.is_ok());
    let tr = result.unwrap();
    assert!(!tr.is_error);
    assert!(tr.content.contains("hello"));
}

#[tokio::test]
async fn test_web_fetch_requires_net_outbound() {
    let memory = ferroclaw::memory::MemoryStore::in_memory().unwrap();
    let memory = Arc::new(Mutex::new(memory));
    let mut registry = ToolRegistry::new();
    register_builtin_tools(&mut registry, memory);

    // Without NetOutbound
    let no_net = CapabilitySet::new([Capability::FsRead]);
    let result = registry
        .execute(
            "web_fetch",
            "tc_5",
            &serde_json::json!({"url": "https://example.com"}),
            &no_net,
        )
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Capability denied"));
}

#[tokio::test]
async fn test_memory_tools_require_memory_caps() {
    let memory = ferroclaw::memory::MemoryStore::in_memory().unwrap();
    let memory = Arc::new(Mutex::new(memory));
    let mut registry = ToolRegistry::new();
    register_builtin_tools(&mut registry, memory);

    // memory_store requires MemoryWrite
    let read_only = CapabilitySet::new([Capability::MemoryRead]);
    let result = registry
        .execute(
            "memory_store",
            "tc_6",
            &serde_json::json!({"key": "test", "content": "value"}),
            &read_only,
        )
        .await;
    assert!(result.is_err());

    // memory_search requires MemoryRead
    let write_only = CapabilitySet::new([Capability::MemoryWrite]);
    let result = registry
        .execute(
            "memory_search",
            "tc_7",
            &serde_json::json!({"query": "test"}),
            &write_only,
        )
        .await;
    assert!(result.is_err());
}

#[test]
fn test_capability_all_set() {
    let all = CapabilitySet::all();
    assert!(all.has(Capability::FsRead));
    assert!(all.has(Capability::FsWrite));
    assert!(all.has(Capability::NetOutbound));
    assert!(all.has(Capability::NetListen));
    assert!(all.has(Capability::ProcessExec));
    assert!(all.has(Capability::MemoryRead));
    assert!(all.has(Capability::MemoryWrite));
    assert!(all.has(Capability::BrowserControl));
}

#[test]
fn test_check_with_message_produces_actionable_error() {
    let caps = CapabilitySet::new([Capability::FsRead]);
    let err = check_with_message(&caps, &[Capability::FsWrite], "write_file").unwrap_err();
    assert!(err.contains("write_file"));
    assert!(err.contains("fs_write"));
    assert!(err.contains("config.toml"));
}

// ── Audit Log ───────────────────────────────────────────────────────

#[test]
fn test_audit_chain_integrity_with_100_entries() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("audit.jsonl");
    let mut log = AuditLog::new(path.clone(), true);

    for i in 0..100 {
        log.log_tool_call(
            &format!("tool_{i}"),
            &format!("{{\"arg\": {i}}}"),
            &format!("result_{i}"),
            i % 10 == 0,
        );
    }

    let result = log.verify().unwrap();
    assert!(result.valid);
    assert_eq!(result.entries, 100);
}

#[test]
fn test_audit_detects_deletion() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("audit.jsonl");
    let mut log = AuditLog::new(path.clone(), true);

    log.log_tool_call("tool_a", "{}", "ok", false);
    log.log_tool_call("tool_b", "{}", "ok", false);
    log.log_tool_call("tool_c", "{}", "ok", false);

    // Delete the middle line
    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let tampered = format!("{}\n{}\n", lines[0], lines[2]);
    std::fs::write(&path, tampered).unwrap();

    let log2 = AuditLog::new(path, true);
    let result = log2.verify().unwrap();
    assert!(!result.valid);
}

#[test]
fn test_audit_detects_insertion() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("audit.jsonl");
    let mut log = AuditLog::new(path.clone(), true);

    log.log_tool_call("tool_a", "{}", "ok", false);
    log.log_tool_call("tool_b", "{}", "ok", false);

    // Insert a fake entry between them
    let content = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let fake = r#"{"timestamp":"2026-01-01T00:00:00Z","tool_name":"evil","arguments_hash":"0000","result_hash":"0000","is_error":false,"previous_hash":"","entry_hash":"fake"}"#;
    let tampered = format!("{}\n{fake}\n{}\n", lines[0], lines[1]);
    std::fs::write(&path, tampered).unwrap();

    let log2 = AuditLog::new(path, true);
    let result = log2.verify().unwrap();
    assert!(!result.valid);
}

#[test]
fn test_audit_resumes_from_existing_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("audit.jsonl");

    // First session
    {
        let mut log = AuditLog::new(path.clone(), true);
        log.log_tool_call("session1_tool", "{}", "ok", false);
    }

    // Second session — should pick up the chain
    {
        let mut log = AuditLog::new(path.clone(), true);
        log.log_tool_call("session2_tool", "{}", "ok", false);
    }

    let log = AuditLog::new(path, true);
    let result = log.verify().unwrap();
    assert!(result.valid);
    assert_eq!(result.entries, 2);
}

// ── Tool Not Found ──────────────────────────────────────────────────

#[tokio::test]
async fn test_execute_nonexistent_tool() {
    let memory = ferroclaw::memory::MemoryStore::in_memory().unwrap();
    let memory = Arc::new(Mutex::new(memory));
    let mut registry = ToolRegistry::new();
    register_builtin_tools(&mut registry, memory);

    let caps = CapabilitySet::all();
    let result = registry
        .execute("nonexistent_tool", "tc_99", &serde_json::json!({}), &caps)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}
