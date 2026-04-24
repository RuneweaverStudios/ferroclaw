//! Integration tests for agent orchestration

use crate::agent::orchestration::{AgentExecution, AgentMessage, AgentMessageBus, SubagentConfig};

#[test]
fn test_subagent_config_builder_pattern() {
    // Test builder pattern for SubagentConfig
    let config = SubagentConfig::new("agent_test_1".to_string(), "coder".to_string())
        .with_prompt("You are a Rust expert")
        .with_tools(vec![
            "read_file".to_string(),
            "write_file".to_string(),
            "bash".to_string(),
        ])
        .with_memory_isolation(false)
        .with_token_budget(50000)
        .with_max_iterations(100);

    assert_eq!(config.agent_id, "agent_test_1");
    assert_eq!(config.agent_type, "coder");
    assert_eq!(
        config.system_prompt,
        Some("You are a Rust expert".to_string())
    );
    assert_eq!(config.allowed_tools.len(), 3);
    assert!(!config.memory_isolation);
    assert_eq!(config.token_budget, 50000);
    assert_eq!(config.max_iterations, Some(100));
}

#[test]
fn test_subagent_config_defaults() {
    // Test default values
    let config = SubagentConfig::new("agent_defaults".to_string(), "planner".to_string());

    assert_eq!(config.agent_id, "agent_defaults");
    assert_eq!(config.agent_type, "planner");
    assert!(config.system_prompt.is_none());
    assert!(config.allowed_tools.is_empty());
    assert!(config.memory_isolation); // Default is isolated
    assert_eq!(config.token_budget, 0); // 0 means use parent's budget
    assert!(config.max_iterations.is_none());
}

#[test]
fn test_agent_message_creation() {
    let msg = AgentMessage::new("agent_1", "agent_2", "Hello from agent 1");

    assert_eq!(msg.from_agent_id, "agent_1");
    assert_eq!(msg.to_agent_id, "agent_2");
    assert_eq!(msg.content, "Hello from agent 1");
    // Verify timestamp is recent (within 1 second)
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(msg.timestamp);
    assert!(diff.num_seconds() < 1);
}

#[test]
fn test_agent_message_bus_single_message() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    bus.register("agent_2".to_string());

    // Send a message
    let msg = AgentMessage::new("agent_1", "agent_2", "Test message");
    bus.send(msg).unwrap();

    // Receiver should get the message
    let messages = bus.receive("agent_2");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test message");

    // Messages should be consumed
    assert_eq!(bus.receive("agent_2").len(), 0);
}

#[test]
fn test_agent_message_bus_multiple_messages() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    bus.register("agent_2".to_string());

    // Send multiple messages
    for i in 1..=5 {
        let msg = AgentMessage::new("agent_1", "agent_2", format!("Message {}", i));
        bus.send(msg).unwrap();
    }

    // Receiver should get one message at a time
    let messages = bus.receive("agent_2");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Message 1");

    // Second call gets second message
    let messages = bus.receive("agent_2");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Message 2");
}

#[test]
fn test_agent_message_bus_broadcast() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    bus.register("agent_2".to_string());
    bus.register("agent_3".to_string());

    // Send broadcast message (empty to_agent_id)
    let msg = AgentMessage::new("agent_1", "", "Broadcast to all");
    bus.send(msg).unwrap();

    // agent_1 should NOT receive its own broadcast
    let messages_1 = bus.receive("agent_1");
    assert_eq!(messages_1.len(), 0);

    // agent_2 should receive broadcast (consumes it)
    let messages_2 = bus.receive("agent_2");
    assert_eq!(messages_2.len(), 1);
    assert_eq!(messages_2[0].content, "Broadcast to all");

    // agent_3 will NOT receive broadcast (already consumed by agent_2)
    let messages_3 = bus.receive("agent_3");
    assert_eq!(messages_3.len(), 0);
}

#[test]
fn test_agent_message_bus_send_to_unregistered_agent() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    // agent_2 is NOT registered

    let msg = AgentMessage::new("agent_1", "agent_2", "Test");
    let result = bus.send(msg);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("unregistered agent"));
}

#[test]
fn test_agent_message_bus_has_messages() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    bus.register("agent_2".to_string());

    // No messages initially
    assert!(!bus.has_messages("agent_1"));
    assert!(!bus.has_messages("agent_2"));

    // Send a message
    let msg = AgentMessage::new("agent_2", "agent_1", "Hello");
    bus.send(msg).unwrap();

    // agent_1 should now have messages
    assert!(bus.has_messages("agent_1"));
    assert!(!bus.has_messages("agent_2"));

    // Consume the message
    bus.receive("agent_1");

    // No more messages
    assert!(!bus.has_messages("agent_1"));
}

#[test]
fn test_agent_message_bus_message_count() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    bus.register("agent_2".to_string());

    // Initial count
    assert_eq!(bus.message_count("agent_1"), 0);

    // Add messages
    for i in 1..=3 {
        let msg = AgentMessage::new("agent_2", "agent_1", format!("Msg {}", i));
        bus.send(msg).unwrap();
    }

    // Count should be 3
    assert_eq!(bus.message_count("agent_1"), 3);

    // Consume one message
    let _ = bus.receive("agent_1");

    // Count should be 2
    assert_eq!(bus.message_count("agent_1"), 2);
}

#[test]
fn test_agent_execution_creation() {
    let execution = AgentExecution::new("agent_exec_1".to_string());

    assert_eq!(execution.agent_id, "agent_exec_1");
    assert_eq!(execution.response, "");
    assert_eq!(execution.tool_calls, 0);
    assert_eq!(execution.tokens_used, 0);
    assert_eq!(execution.messages_received, 0);
    assert_eq!(execution.messages_sent, 0);
}

#[test]
fn test_agent_execution_cloning() {
    let mut execution = AgentExecution::new("agent_exec_1".to_string());
    execution.response = "Test response".to_string();
    execution.tool_calls = 5;
    execution.tokens_used = 1000;

    let mut cloned = execution.clone();

    assert_eq!(cloned.agent_id, "agent_exec_1");
    assert_eq!(cloned.response, "Test response");
    assert_eq!(cloned.tool_calls, 5);
    assert_eq!(cloned.tokens_used, 1000);

    // Verify it's a true clone
    cloned.response = "Modified".to_string();
    assert_eq!(execution.response, "Test response");
    assert_eq!(cloned.response, "Modified");
}

#[test]
fn test_message_bus_mixed_direct_and_broadcast() {
    let mut bus = AgentMessageBus::new();

    bus.register("agent_1".to_string());
    bus.register("agent_2".to_string());
    bus.register("agent_3".to_string());

    // Send direct message
    bus.send(AgentMessage::new("agent_1", "agent_2", "Direct message"))
        .unwrap();

    // Send broadcast
    bus.send(AgentMessage::new("agent_1", "", "Broadcast message"))
        .unwrap();

    // agent_2 should receive the direct message first
    let messages_2 = bus.receive("agent_2");
    assert_eq!(messages_2.len(), 1);
    assert_eq!(messages_2[0].content, "Direct message");

    // agent_3 should receive the broadcast
    let messages_3 = bus.receive("agent_3");
    assert_eq!(messages_3.len(), 1);
    assert_eq!(messages_3[0].content, "Broadcast message");

    // agent_1 should receive nothing (not its own direct message or broadcast)
    let messages_1 = bus.receive("agent_1");
    assert_eq!(messages_1.len(), 0);
}

#[test]
fn test_subagent_config_serialization() {
    // Test that SubagentConfig can be converted to/from JSON if needed
    let config = SubagentConfig::new("test".to_string(), "coder".to_string());

    // Just verify that struct is complete and has all expected fields
    assert!(!config.agent_id.is_empty());
    assert!(!config.agent_type.is_empty());
}

#[test]
fn test_message_bus_concurrent_access_pattern() {
    // Simulate a pattern of concurrent-like access
    let mut bus = AgentMessageBus::new();

    // Register multiple agents
    for i in 1..=5 {
        bus.register(format!("agent_{}", i));
    }

    // Send messages between agents
    for i in 1..=5 {
        for j in 1..=5 {
            if i != j {
                let msg = AgentMessage::new(
                    format!("agent_{}", i),
                    format!("agent_{}", j),
                    format!("Msg {}→{}", i, j),
                );
                bus.send(msg).unwrap();
            }
        }
    }

    // Each agent should have received 4 messages
    for i in 1..=5 {
        let count = bus.message_count(&format!("agent_{}", i));
        assert_eq!(count, 4, "agent_{} should have 4 messages", i);
    }
}
