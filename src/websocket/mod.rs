//! WebSocket server for real-time event broadcasting.
//!
//! Listens on ws://localhost:8420 and broadcasts:
//! - AGENT_STATE_UPDATE (when agent status changes)
//! - TOOL_CALL_START (when tool execution begins)
//! - TOOL_CALL_UPDATE (when tool state changes)
//! - TOOL_OUTPUT_CHUNK (streaming output)

use crate::error::{FerroError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::io::Error as IoError;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::Message;

/// WebSocket event types that can be broadcast to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    /// Agent state changed (idle, thinking, executing, error)
    #[serde(rename = "agent_state_update")]
    AgentStateUpdate {
        agent_id: String,
        state: AgentState,
        timestamp: i64,
    },

    /// Tool execution started
    #[serde(rename = "tool_call_start")]
    ToolCallStart {
        call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
        timestamp: i64,
    },

    /// Tool execution state updated
    #[serde(rename = "tool_call_update")]
    ToolCallUpdate {
        call_id: String,
        state: ToolState,
        timestamp: i64,
    },

    /// Streaming output chunk from tool execution
    #[serde(rename = "tool_output_chunk")]
    ToolOutputChunk {
        call_id: String,
        chunk: String,
        is_final: bool,
        timestamp: i64,
    },
}

/// Agent execution state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Idle,
    Thinking,
    Executing,
    Error,
}

/// Tool execution state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolState {
    Pending,
    Running,
    Completed,
    Failed,
}

impl WsEvent {
    /// Get timestamp as milliseconds since Unix epoch.
    fn timestamp() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    /// Create an agent state update event.
    pub fn agent_state(agent_id: String, state: AgentState) -> Self {
        Self::AgentStateUpdate {
            agent_id,
            state,
            timestamp: Self::timestamp(),
        }
    }

    /// Create a tool call start event.
    pub fn tool_start(call_id: String, tool_name: String, arguments: serde_json::Value) -> Self {
        Self::ToolCallStart {
            call_id,
            tool_name,
            arguments,
            timestamp: Self::timestamp(),
        }
    }

    /// Create a tool call update event.
    pub fn tool_update(call_id: String, state: ToolState) -> Self {
        Self::ToolCallUpdate {
            call_id,
            state,
            timestamp: Self::timestamp(),
        }
    }

    /// Create a tool output chunk event.
    pub fn tool_chunk(call_id: String, chunk: String, is_final: bool) -> Self {
        Self::ToolOutputChunk {
            call_id,
            chunk,
            is_final,
            timestamp: Self::timestamp(),
        }
    }

    /// Serialize to JSON for WebSocket transmission.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(FerroError::from)
    }
}

/// Shared broadcast channel for WebSocket events.
#[derive(Clone)]
pub struct WsBroadcaster {
    tx: broadcast::Sender<WsEvent>,
}

impl WsBroadcaster {
    /// Create a new broadcaster with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Broadcast an event to all connected clients.
    pub fn broadcast(&self, event: WsEvent) -> Result<()> {
        match self.tx.send(event) {
            Ok(_) => Ok(()),
            Err(broadcast::error::SendError(_)) => {
                // No receivers connected, not a fatal error
                Ok(())
            }
        }
    }

    /// Subscribe to events.
    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.tx.subscribe()
    }
}

/// WebSocket server state.
pub struct WsServer {
    bind_addr: String,
    port: u16,
    broadcaster: WsBroadcaster,
}

impl WsServer {
    /// Create a new WebSocket server.
    pub fn new(bind_addr: String, port: u16) -> Self {
        Self {
            bind_addr,
            port,
            broadcaster: WsBroadcaster::new(1000),
        }
    }

    /// Get the broadcaster for sending events.
    pub fn broadcaster(&self) -> WsBroadcaster {
        self.broadcaster.clone()
    }

    /// Get the listen address.
    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.bind_addr, self.port)
    }

    /// Start the WebSocket server.
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr()).await.map_err(|e| {
            FerroError::Io(IoError::new(
                std::io::ErrorKind::AddrInUse,
                format!("Failed to bind WebSocket server: {e}"),
            ))
        })?;

        tracing::info!("WebSocket server listening on ws://{}", self.listen_addr());

        while let Ok((stream, addr)) = listener.accept().await {
            let rx = self.broadcaster.subscribe();

            tokio::spawn(async move {
                match Self::handle_connection(stream, addr.to_string(), rx).await {
                    Ok(_) => {
                        tracing::debug!("WebSocket client disconnected: {}", addr);
                    }
                    Err(e) => {
                        tracing::warn!("WebSocket connection error from {}: {}", addr, e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Handle a single WebSocket connection.
    async fn handle_connection(
        stream: TcpStream,
        addr: String,
        mut rx: broadcast::Receiver<WsEvent>,
    ) -> Result<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await.map_err(|e| {
            FerroError::Io(IoError::new(
                std::io::ErrorKind::ConnectionReset,
                format!("WebSocket handshake failed: {e}"),
            ))
        })?;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        tracing::info!("WebSocket client connected: {}", addr);
        let addr_clone = addr.clone();

        // Spawn task to forward broadcast events to this client
        let send_task = tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let json = match event.to_json() {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!("Failed to serialize event: {}", e);
                        continue;
                    }
                };

                if let Err(e) = ws_sender.send(Message::Text(json)).await {
                    tracing::warn!("Failed to send message to client {}: {}", addr_clone, e);
                    break;
                }
            }
        });

        // Handle incoming messages from client (ping/pong/close)
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => {
                    tracing::debug!("Client {} requested close", addr);
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    // Respond to ping with pong
                    // Note: tungstenite handles this automatically, but we can log it
                    tracing::trace!("Received ping from {}", addr);
                }
                Ok(Message::Pong(_)) => {
                    tracing::trace!("Received pong from {}", addr);
                }
                Ok(Message::Text(text)) => {
                    tracing::debug!("Received text from {}: {}", addr, text);
                    // Clients can send commands here if needed
                }
                Err(e) => {
                    tracing::warn!("WebSocket error from {}: {}", addr, e);
                    break;
                }
                _ => {}
            }
        }

        // Clean up
        send_task.abort();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_event_serialization() {
        let event = WsEvent::AgentStateUpdate {
            agent_id: "test-agent".to_string(),
            state: AgentState::Thinking,
            timestamp: 12345,
        };

        let json = event.to_json().unwrap();
        assert!(json.contains("agent_state_update"));
        assert!(json.contains("thinking"));
    }

    #[test]
    fn test_broadcaster() {
        let broadcaster = WsBroadcaster::new(10);
        let event = WsEvent::agent_state("test".to_string(), AgentState::Idle);

        // Should not fail even with no receivers
        assert!(broadcaster.broadcast(event).is_ok());
    }

    #[test]
    fn test_ws_server_listen_addr() {
        let server = WsServer::new("127.0.0.1".to_string(), 8420);
        assert_eq!(server.listen_addr(), "127.0.0.1:8420");
    }
}
