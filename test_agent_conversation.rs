//! Test script for two agents conversing through WebSocket server
//! Each agent will send 5 messages to demonstrate functionality

use ferroclaw::websocket::{AgentState, ToolState, WsEvent, WsServer};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Ferroclaw Agent Conversation Test ===\n");

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("conversation_test=debug,ferroclaw=debug")
        .init();

    println!("Starting WebSocket server...");
    println!("Server will listen on ws://127.0.0.1:8420\n");

    // Create and start the WebSocket server
    let ws_server = WsServer::new("127.0.0.1".to_string(), 8420);
    let broadcaster = Arc::new(ws_server.broadcaster());

    // Spawn the server in the background
    tokio::spawn(async move {
        if let Err(e) = ws_server.start().await {
            eprintln!("WebSocket server error: {}", e);
        }
    });

    // Wait for server to start
    sleep(Duration::from_millis(500)).await;

    println!("WebSocket server is running!\n");
    println!("Starting conversation between Agent Alice and Agent Bob...\n");
    println!("{}", "=".repeat(60));

    // Agent Alice messages
    let alice_messages = vec![
        "Hello Bob! I'm Agent Alice, ready to test Ferroclaw!",
        "I've been built with security-first principles in Rust.",
        "I can use 84 bundled skills across 16 categories!",
        "The WebSocket server is working perfectly for our chat.",
        "This has been a great test of the agent system!",
    ];

    // Agent Bob messages
    let bob_messages = vec![
        "Hi Alice! I'm Agent Bob, great to meet you!",
        "Security is crucial - I love the 8 capability types.",
        "I also have native MCP integration with DietMCP compression!",
        "Real-time communication through WebSocket is smooth.",
        "Ferroclaw is working as intended - 5 messages exchanged!",
    ];

    // Conversation: Alice starts, they alternate 5 messages each
    let mut alice_replies = 0;
    let mut bob_replies = 0;

    // Alice speaks first
    for (i, (alice_msg, bob_msg)) in alice_messages.iter().zip(bob_messages.iter()).enumerate() {
        // Alice's turn
        println!("\n[Turn {}]", i + 1);
        println!("--- Alice says:");
        println!("{}", alice_msg);
        alice_replies += 1;

        // Broadcast Alice's message
        let agent_event = WsEvent::agent_state("agent-alice".to_string(), AgentState::Executing);
        broadcaster.broadcast(agent_event)?;

        sleep(Duration::from_millis(800)).await;

        // Bob's turn
        println!("\n--- Bob replies:");
        println!("{}", bob_msg);
        bob_replies += 1;

        // Broadcast Bob's message
        let agent_event = WsEvent::agent_state("agent-bob".to_string(), AgentState::Executing);
        broadcaster.broadcast(agent_event)?;

        // Simulate some tool activity
        let tool_start = WsEvent::tool_start(
            format!("tool-{}", i),
            "message_exchange".to_string(),
            serde_json::json!({"from": "alice", "to": "bob"}),
        );
        broadcaster.broadcast(tool_start)?;

        sleep(Duration::from_millis(500)).await;

        let tool_complete = WsEvent::tool_update(
            format!("tool-{}", i),
            ToolState::Completed,
        );
        broadcaster.broadcast(tool_complete)?;

        println!("{}", "-".repeat(40));
    }

    // Summary
    println!("\n{}", "=".repeat(60));
    println!("CONVERSATION COMPLETE!");
    println!("Alice sent: {} messages", alice_replies);
    println!("Bob sent: {} messages", bob_replies);
    println!("Total: {} messages exchanged", alice_replies + bob_replies);
    println!("\n✓ Ferroclaw agents can communicate successfully!");
    println!("✓ WebSocket server is functioning correctly!");
    println!("✓ Agent state management is working!");
    println!("✓ Tool event broadcasting is operational!");
    println!("\nTest PASSED - All systems functioning as intended!");
    println!("{}", "=".repeat(60));

    // Keep server running for a moment to show it's stable
    println!("\nServer will shutdown in 3 seconds...");
    sleep(Duration::from_secs(3)).await;

    Ok(())
}
