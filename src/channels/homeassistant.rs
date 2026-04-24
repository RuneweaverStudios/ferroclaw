//! Home Assistant channel adapter.
//!
//! Integrates with Home Assistant's Conversation and REST API.
//! Allows Ferroclaw to act as a conversation agent in HA and
//! interact with smart home devices.

use crate::channels::{Channel, OutgoingMessage};
use crate::config::HomeAssistantConfig;
use crate::error::{FerroError, Result};
use std::future::Future;
use std::pin::Pin;

pub struct HomeAssistantChannel {
    api_url: String,
    token: String,
    entity_id: Option<String>,
    client: reqwest::Client,
}

impl HomeAssistantChannel {
    pub fn from_config(config: &HomeAssistantConfig) -> Option<Self> {
        let token = std::env::var(&config.token_env).ok()?;

        Some(Self {
            api_url: config.api_url.trim_end_matches('/').to_string(),
            token,
            entity_id: config.entity_id.clone(),
            client: reqwest::Client::new(),
        })
    }

    /// Send a notification via Home Assistant notification service.
    pub async fn send_notification(&self, title: &str, message: &str) -> Result<()> {
        let url = format!(
            "{}/api/services/notify/persistent_notification",
            self.api_url
        );

        let body = serde_json::json!({
            "title": title,
            "message": message,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("Home Assistant send error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(FerroError::Channel(format!(
                "Home Assistant API error {status}: {body}"
            )));
        }

        Ok(())
    }

    /// Process a conversation turn via HA Conversation API.
    pub async fn conversation_process(&self, text: &str) -> Result<String> {
        let url = format!("{}/api/conversation/process", self.api_url);

        let mut body = serde_json::json!({
            "text": text,
            "language": "en",
        });

        if let Some(ref entity) = self.entity_id {
            body["agent_id"] = serde_json::Value::String(entity.clone());
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("HA conversation error: {e}")))?;

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| FerroError::Channel(format!("HA response parse error: {e}")))?;

        let speech = result
            .pointer("/response/speech/plain/speech")
            .and_then(|s| s.as_str())
            .unwrap_or("(no response)")
            .to_string();

        Ok(speech)
    }

    /// Call a Home Assistant service (e.g. turn on a light).
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        let url = format!("{}/api/services/{}/{}", self.api_url, domain, service);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("HA service call error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(FerroError::Channel(format!(
                "HA service error {status}: {body}"
            )));
        }

        Ok(())
    }

    /// Get the state of a Home Assistant entity.
    pub async fn get_entity_state(&self, entity_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/states/{}", self.api_url, entity_id);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| FerroError::Channel(format!("HA state error: {e}")))?;

        resp.json()
            .await
            .map_err(|e| FerroError::Channel(format!("HA state parse error: {e}")))
    }
}

impl Channel for HomeAssistantChannel {
    fn name(&self) -> &str {
        "homeassistant"
    }

    fn is_configured(&self) -> bool {
        !self.api_url.is_empty() && !self.token.is_empty()
    }

    fn send<'a>(
        &'a self,
        _target: &'a str,
        message: OutgoingMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let title = if message.is_error {
                "Ferroclaw Error"
            } else {
                "Ferroclaw"
            };

            self.send_notification(title, &message.text).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name() {
        let channel = HomeAssistantChannel {
            api_url: "http://homeassistant.local:8123".into(),
            token: "test-token".into(),
            entity_id: None,
            client: reqwest::Client::new(),
        };
        assert_eq!(channel.name(), "homeassistant");
        assert!(channel.is_configured());
    }
}
