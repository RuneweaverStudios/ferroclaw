//! Channel abstraction for multi-platform messaging.
//!
//! Each messaging platform implements the Channel trait.
//! Channels receive messages, route them to the agent loop, and send responses back.
//! All adapters use HTTP APIs via reqwest — no platform SDKs compiled in.

pub mod discord;
pub mod email;
pub mod homeassistant;
pub mod router;
pub mod signal;
pub mod slack;
pub mod whatsapp;

use crate::error::Result;
use std::future::Future;
use std::pin::Pin;

/// Incoming message from any channel.
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    /// Channel identifier (e.g. "telegram", "discord", "slack").
    pub channel: String,
    /// Platform-specific sender identifier.
    pub sender_id: String,
    /// The message text content.
    pub text: String,
    /// Session key for conversation continuity (channel + sender combo).
    pub session_key: String,
    /// Optional reply-to identifier for threading.
    pub reply_to: Option<String>,
}

/// Outgoing response to a channel.
#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    /// Response text content.
    pub text: String,
    /// Whether this is an error response.
    pub is_error: bool,
    /// Optional thread/reply ID for threading.
    pub thread_id: Option<String>,
}

/// Trait for messaging channel implementations.
///
/// Each channel adapter handles platform-specific:
/// - Authentication and connection setup
/// - Message format conversion
/// - Rate limiting and retries
/// - Allowlist enforcement
pub trait Channel: Send + Sync {
    /// Channel name (e.g. "discord", "slack").
    fn name(&self) -> &str;

    /// Whether this channel is configured and ready to start.
    fn is_configured(&self) -> bool;

    /// Send a message through this channel.
    fn send<'a>(
        &'a self,
        target: &'a str,
        message: OutgoingMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}

/// Channel status for health checks.
#[derive(Debug, Clone)]
pub struct ChannelStatus {
    pub name: String,
    pub connected: bool,
    pub message_count: u64,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}
