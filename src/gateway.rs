//! HTTP gateway — binds 127.0.0.1 ONLY by default.
//!
//! Provides a REST API for external integrations.
//! Token-authenticated to prevent unauthorized access.

use crate::config::Config;
use crate::error::Result;

/// Gateway server configuration derived from the main config.
pub struct Gateway {
    pub bind_addr: String,
    pub port: u16,
    pub bearer_token: Option<String>,
}

impl Gateway {
    pub fn from_config(config: &Config) -> Self {
        let bearer_token = config
            .gateway
            .bearer_token
            .clone()
            .or_else(|| {
                config
                    .gateway
                    .bearer_token_env
                    .as_ref()
                    .and_then(|env_var| std::env::var(env_var).ok())
            });

        Self {
            bind_addr: config.gateway.bind.clone(),
            port: config.gateway.port,
            bearer_token,
        }
    }

    pub fn listen_addr(&self) -> String {
        format!("{}:{}", self.bind_addr, self.port)
    }

    /// Validate that we're not binding to 0.0.0.0 without explicit opt-in.
    pub fn validate_bind_safety(&self) -> Result<()> {
        if self.bind_addr == "0.0.0.0" {
            tracing::warn!(
                "Gateway binding to 0.0.0.0 (all interfaces). \
                 This exposes the agent to the network. \
                 Use 127.0.0.1 for local-only access."
            );
            if self.bearer_token.is_none() {
                return Err(crate::error::FerroError::Security(
                    "Refusing to bind to 0.0.0.0 without bearer_token authentication. \
                     Set gateway.bearer_token or gateway.bearer_token_env in config."
                        .into(),
                ));
            }
        }
        Ok(())
    }
}

/// Placeholder for the HTTP server. A full implementation would use
/// axum, warp, or similar. For now, we validate configuration.
pub async fn start_gateway(config: &Config) -> Result<()> {
    let gateway = Gateway::from_config(config);
    gateway.validate_bind_safety()?;

    tracing::info!("Gateway would start on {}", gateway.listen_addr());
    tracing::info!(
        "Auth: {}",
        if gateway.bearer_token.is_some() {
            "enabled"
        } else {
            "disabled (local only)"
        }
    );

    // TODO: Implement HTTP routes using a lightweight framework
    // Routes:
    //   POST /v1/chat     — send a message, get response
    //   GET  /v1/tools    — list available tools
    //   GET  /v1/health   — health check
    //   GET  /v1/memories — search memories

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_gateway_from_default_config() {
        let config = Config::default();
        let gw = Gateway::from_config(&config);
        assert_eq!(gw.bind_addr, "127.0.0.1");
        assert_eq!(gw.port, 8420);
    }

    #[test]
    fn test_gateway_blocks_unsafe_bind() {
        let mut config = Config::default();
        config.gateway.bind = "0.0.0.0".into();
        config.gateway.bearer_token = None;
        let gw = Gateway::from_config(&config);
        assert!(gw.validate_bind_safety().is_err());
    }

    #[test]
    fn test_gateway_allows_safe_bind() {
        let config = Config::default();
        let gw = Gateway::from_config(&config);
        assert!(gw.validate_bind_safety().is_ok());
    }

    #[test]
    fn test_gateway_allows_0000_with_token() {
        let mut config = Config::default();
        config.gateway.bind = "0.0.0.0".into();
        config.gateway.bearer_token = Some("secret".into());
        let gw = Gateway::from_config(&config);
        assert!(gw.validate_bind_safety().is_ok());
    }
}
