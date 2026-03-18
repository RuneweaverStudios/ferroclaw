//! Signal channel adapter.
//!
//! Uses the signal-cli REST API (https://github.com/bbernhard/signal-cli-rest-api)
//! for sending and receiving Signal messages. Requires signal-cli running as a
//! REST service, typically via Docker.

use crate::channels::{Channel, OutgoingMessage};
use crate::config::SignalConfig;
use crate::error::{FerroError, Result};
use std::future::Future;
use std::pin::Pin;

pub struct SignalChannel {
    api_url: String,
    phone_number: String,
    allowed_numbers: Vec<String>,
    client: reqwest::Client,
}

impl SignalChannel {
    pub fn from_config(config: &SignalConfig) -> Self {
        Self {
            api_url: config.api_url.trim_end_matches('/').to_string(),
            phone_number: config.phone_number.clone(),
            allowed_numbers: config.allowed_numbers.clone(),
            client: reqwest::Client::new(),
        }
    }

    /// Check if a phone number is allowed to interact.
    pub fn is_allowed(&self, number: &str) -> bool {
        self.allowed_numbers.is_empty() || self.allowed_numbers.contains(&number.to_string())
    }

    /// Send a message via signal-cli REST API.
    pub async fn send_message(&self, recipient: &str, message: &str) -> Result<()> {
        let url = format!("{}/v2/send", self.api_url);

        let body = serde_json::json!({
            "message": message,
            "number": self.phone_number,
            "recipients": [recipient],
        });

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("Signal send error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(FerroError::Channel(format!(
                "Signal API error {status}: {body}"
            )));
        }

        Ok(())
    }

    /// Receive pending messages via signal-cli REST API.
    pub async fn receive_messages(&self) -> Result<Vec<SignalIncoming>> {
        let url = format!("{}/v1/receive/{}", self.api_url, self.phone_number);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("Signal receive error: {e}")))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let messages: Vec<SignalIncoming> = resp
            .json()
            .await
            .map_err(|e| FerroError::Channel(format!("Signal parse error: {e}")))?;

        Ok(messages)
    }
}

impl Channel for SignalChannel {
    fn name(&self) -> &str {
        "signal"
    }

    fn is_configured(&self) -> bool {
        !self.api_url.is_empty() && !self.phone_number.is_empty()
    }

    fn send<'a>(
        &'a self,
        target: &'a str,
        message: OutgoingMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let text = if message.is_error {
                format!("[Error] {}", message.text)
            } else {
                message.text
            };

            self.send_message(target, &text).await
        })
    }
}

/// Incoming Signal message from the REST API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SignalIncoming {
    #[serde(default)]
    pub envelope: Option<SignalEnvelope>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SignalEnvelope {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(rename = "dataMessage", default)]
    pub data_message: Option<SignalDataMessage>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SignalDataMessage {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist() {
        let channel = SignalChannel {
            api_url: "http://localhost:8080".into(),
            phone_number: "+1234567890".into(),
            allowed_numbers: vec!["+1111111111".into()],
            client: reqwest::Client::new(),
        };
        assert!(channel.is_allowed("+1111111111"));
        assert!(!channel.is_allowed("+9999999999"));
    }

    #[test]
    fn test_empty_allowlist() {
        let channel = SignalChannel {
            api_url: "http://localhost:8080".into(),
            phone_number: "+1234567890".into(),
            allowed_numbers: vec![],
            client: reqwest::Client::new(),
        };
        assert!(channel.is_allowed("+anything"));
    }
}
