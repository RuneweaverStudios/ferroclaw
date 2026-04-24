//! Slack channel adapter.
//!
//! Uses the Slack Web API for sending messages and Socket Mode / Events API
//! for receiving. No Slack SDK — just reqwest HTTP calls.

use crate::channels::{Channel, OutgoingMessage};
use crate::config::SlackConfig;
use crate::error::{FerroError, Result};
use std::future::Future;
use std::pin::Pin;

pub struct SlackChannel {
    bot_token: String,
    #[allow(dead_code)]
    app_token: Option<String>,
    allowed_channels: Vec<String>,
    client: reqwest::Client,
}

impl SlackChannel {
    pub fn from_config(config: &SlackConfig) -> Option<Self> {
        let bot_token = std::env::var(&config.bot_token_env).ok()?;
        let app_token = config
            .app_token_env
            .as_ref()
            .and_then(|env| std::env::var(env).ok());

        Some(Self {
            bot_token,
            app_token,
            allowed_channels: config.allowed_channels.clone(),
            client: reqwest::Client::new(),
        })
    }

    /// Check if a channel is allowed.
    pub fn is_allowed(&self, channel_id: &str) -> bool {
        self.allowed_channels.is_empty() || self.allowed_channels.contains(&channel_id.to_string())
    }

    /// Send a message to a Slack channel via Web API.
    pub async fn send_message(
        &self,
        channel_id: &str,
        text: &str,
        thread_ts: Option<&str>,
    ) -> Result<()> {
        let mut body = serde_json::json!({
            "channel": channel_id,
            "text": text,
        });

        if let Some(ts) = thread_ts {
            body["thread_ts"] = serde_json::Value::String(ts.to_string());
        }

        let resp = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("Slack send error: {e}")))?;

        let status = resp.status();
        let response_body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FerroError::Channel(format!("Slack response parse error: {e}")))?;

        if !status.is_success() || response_body.get("ok") != Some(&serde_json::Value::Bool(true)) {
            let error = response_body
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown");
            return Err(FerroError::Channel(format!("Slack API error: {error}")));
        }

        Ok(())
    }

    /// React to a message with an emoji.
    pub async fn add_reaction(&self, channel_id: &str, timestamp: &str, emoji: &str) -> Result<()> {
        let body = serde_json::json!({
            "channel": channel_id,
            "timestamp": timestamp,
            "name": emoji,
        });

        self.client
            .post("https://slack.com/api/reactions.add")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("Slack reaction error: {e}")))?;

        Ok(())
    }
}

impl Channel for SlackChannel {
    fn name(&self) -> &str {
        "slack"
    }

    fn is_configured(&self) -> bool {
        !self.bot_token.is_empty()
    }

    fn send<'a>(
        &'a self,
        target: &'a str,
        message: OutgoingMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let text = if message.is_error {
                format!(":x: *Error:* {}", message.text)
            } else {
                message.text
            };

            self.send_message(target, &text, message.thread_id.as_deref())
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist_empty_allows_all() {
        let channel = SlackChannel {
            bot_token: "xoxb-test".into(),
            app_token: None,
            allowed_channels: vec![],
            client: reqwest::Client::new(),
        };
        assert!(channel.is_allowed("C1234567"));
    }

    #[test]
    fn test_allowlist_enforced() {
        let channel = SlackChannel {
            bot_token: "xoxb-test".into(),
            app_token: None,
            allowed_channels: vec!["C111".into(), "C222".into()],
            client: reqwest::Client::new(),
        };
        assert!(channel.is_allowed("C111"));
        assert!(!channel.is_allowed("C999"));
    }
}
