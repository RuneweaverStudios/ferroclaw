pub mod anthropic;
pub mod openai;
pub mod openrouter;
pub mod streaming;
pub mod zai;

use crate::config::{Config, resolve_env_var};
use crate::error::{FerroError, Result};
use crate::provider::LlmProvider;

/// Select the appropriate provider for a model string.
///
/// Routing order:
/// 1. Zai GLM models (`glm-*`)
/// 2. OpenRouter models (`provider/model` format with `/`)
/// 3. Anthropic models (`claude-*`)
/// 4. OpenAI-compatible (fallback)
pub fn resolve_provider(model: &str, config: &Config) -> Result<Box<dyn LlmProvider>> {
    // Zai GLM models
    if zai::is_zai_model(model) {
        let zai_cfg = config
            .providers
            .zai
            .as_ref()
            .ok_or_else(|| FerroError::Config("Zai provider not configured".into()))?;
        let api_key = resolve_env_var(&zai_cfg.api_key_env)?;
        return Ok(Box::new(zai::ZaiProvider::new(
            api_key,
            zai_cfg.base_url.clone(),
        )));
    }

    // OpenRouter models (provider/model format)
    if openrouter::is_openrouter_model(model) {
        let or_cfg = config
            .providers
            .openrouter
            .as_ref()
            .ok_or_else(|| FerroError::Config("OpenRouter provider not configured".into()))?;
        let api_key = resolve_env_var(&or_cfg.api_key_env)?;
        return Ok(Box::new(openrouter::OpenRouterProvider::new(
            api_key,
            or_cfg.base_url.clone(),
            or_cfg.site_url.clone(),
            or_cfg.site_name.clone(),
        )));
    }

    // Anthropic models
    if model.starts_with("claude-") {
        let anthropic_cfg = config
            .providers
            .anthropic
            .as_ref()
            .ok_or_else(|| FerroError::Config("Anthropic provider not configured".into()))?;
        let api_key = resolve_env_var(&anthropic_cfg.api_key_env)?;
        return Ok(Box::new(anthropic::AnthropicProvider::new(
            api_key,
            anthropic_cfg.base_url.clone(),
            anthropic_cfg.max_tokens,
        )));
    }

    // OpenAI-compatible fallback
    if let Some(openai_cfg) = &config.providers.openai {
        let api_key = resolve_env_var(&openai_cfg.api_key_env)?;
        return Ok(Box::new(openai::OpenAiProvider::new(
            api_key,
            openai_cfg.base_url.clone(),
            openai_cfg.max_tokens,
        )));
    }

    Err(FerroError::Provider(format!(
        "No provider configured for model '{model}'"
    )))
}
