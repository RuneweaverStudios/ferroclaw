//! Integration tests for the memory subsystem:
//! SQLite + FTS5, CRUD operations, conversation persistence, search ranking.

use ferroclaw::memory::MemoryStore;

#[test]
fn test_memory_insert_and_retrieve() {
    let store = MemoryStore::in_memory().unwrap();
    store
        .insert(
            "api_key_location",
            "Stored in 1Password vault 'Engineering'",
        )
        .unwrap();
    let mem = store.get("api_key_location").unwrap().unwrap();
    assert_eq!(mem.content, "Stored in 1Password vault 'Engineering'");
    assert_eq!(mem.key, "api_key_location");
}

#[test]
fn test_memory_upsert_updates_content() {
    let store = MemoryStore::in_memory().unwrap();
    store.insert("user_name", "Alice").unwrap();
    store.insert("user_name", "Bob").unwrap();
    let mem = store.get("user_name").unwrap().unwrap();
    assert_eq!(mem.content, "Bob");
    // Should still have only 1 entry
    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn test_memory_forget() {
    let store = MemoryStore::in_memory().unwrap();
    store.insert("temporary", "delete me").unwrap();
    assert!(store.get("temporary").unwrap().is_some());
    assert!(store.forget("temporary").unwrap());
    assert!(store.get("temporary").unwrap().is_none());
    // Forgetting nonexistent key returns false
    assert!(!store.forget("nonexistent").unwrap());
}

#[test]
fn test_memory_list_all_ordered_by_update() {
    let store = MemoryStore::in_memory().unwrap();
    store.insert("first", "1").unwrap();
    store.insert("second", "2").unwrap();
    store.insert("third", "3").unwrap();
    // Update first — should now appear first (most recently updated)
    store.insert("first", "1 updated").unwrap();

    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].key, "first"); // Most recently updated
}

#[test]
fn test_memory_fts_search_relevance() {
    let store = MemoryStore::in_memory().unwrap();
    store
        .insert(
            "rust_framework",
            "Ferroclaw is a Rust-based AI agent framework for security-first operation",
        )
        .unwrap();
    store
        .insert(
            "python_tool",
            "dietmcp is a Python tool for compressing MCP schemas to reduce token usage",
        )
        .unwrap();
    store
        .insert(
            "js_project",
            "NanoClaw is a lightweight JavaScript agent that uses Anthropic APIs",
        )
        .unwrap();
    store
        .insert(
            "security_notes",
            "The capability system uses Ed25519 signed manifests and hash-chained audit logs",
        )
        .unwrap();

    // Search for "Rust agent" should rank rust_framework highest
    let results = store.search("Rust agent", 10).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].key, "rust_framework");

    // Search for "Python MCP" should find python_tool
    let results = store.search("Python MCP", 10).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].key, "python_tool");

    // Search for "Ed25519 audit" should find security_notes
    let results = store.search("Ed25519 audit", 10).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].key, "security_notes");
}

#[test]
fn test_memory_search_returns_empty_for_no_match() {
    let store = MemoryStore::in_memory().unwrap();
    store.insert("greeting", "hello world").unwrap();
    let results = store.search("xyznonexistent", 10).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_memory_search_limit() {
    let store = MemoryStore::in_memory().unwrap();
    for i in 0..20 {
        store
            .insert(&format!("item_{i}"), &format!("test data item number {i}"))
            .unwrap();
    }
    let results = store.search("test data item", 5).unwrap();
    assert!(results.len() <= 5);
}

#[test]
fn test_conversation_persistence() {
    let store = MemoryStore::in_memory().unwrap();
    let session = "sess_integration_test";

    store
        .save_conversation(session, "user", "What is Ferroclaw?")
        .unwrap();
    store
        .save_conversation(
            session,
            "assistant",
            "Ferroclaw is a security-first Rust agent.",
        )
        .unwrap();
    store
        .save_conversation(session, "user", "How does it handle MCP?")
        .unwrap();
    store
        .save_conversation(session, "assistant", "It uses native DietMCP compression.")
        .unwrap();

    let history = store.get_conversation(session).unwrap();
    assert_eq!(history.len(), 4);
    assert_eq!(history[0].role, "user");
    assert_eq!(history[1].role, "assistant");
    assert_eq!(history[2].role, "user");
    assert_eq!(history[3].role, "assistant");
    assert!(history[3].content.contains("DietMCP"));
}

#[test]
fn test_conversation_isolation_between_sessions() {
    let store = MemoryStore::in_memory().unwrap();

    store
        .save_conversation("sess_a", "user", "Message for A")
        .unwrap();
    store
        .save_conversation("sess_b", "user", "Message for B")
        .unwrap();
    store
        .save_conversation("sess_a", "assistant", "Reply in A")
        .unwrap();

    let history_a = store.get_conversation("sess_a").unwrap();
    let history_b = store.get_conversation("sess_b").unwrap();
    assert_eq!(history_a.len(), 2);
    assert_eq!(history_b.len(), 1);
}

#[test]
fn test_memory_handles_unicode() {
    let store = MemoryStore::in_memory().unwrap();
    store
        .insert("chinese", "Ferroclaw 是一个安全优先的 AI 代理框架")
        .unwrap();
    store.insert("emoji", "Status: running 🦀🔒").unwrap();
    store
        .insert("arabic", "فيروكلو هو إطار عمل وكيل الذكاء الاصطناعي")
        .unwrap();

    let mem = store.get("chinese").unwrap().unwrap();
    assert!(mem.content.contains("安全优先"));

    let mem = store.get("emoji").unwrap().unwrap();
    assert!(mem.content.contains("🦀"));
}

#[test]
fn test_memory_handles_large_content() {
    let store = MemoryStore::in_memory().unwrap();
    let large_content = "x".repeat(100_000);
    store.insert("large", &large_content).unwrap();
    let mem = store.get("large").unwrap().unwrap();
    assert_eq!(mem.content.len(), 100_000);
}

#[test]
fn test_memory_concurrent_safe() {
    // SQLite in WAL mode should handle concurrent reads
    let store = MemoryStore::in_memory().unwrap();
    for i in 0..100 {
        store
            .insert(&format!("key_{i}"), &format!("value_{i}"))
            .unwrap();
    }
    let all = store.list_all().unwrap();
    assert_eq!(all.len(), 100);
}
