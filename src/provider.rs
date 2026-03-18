use crate::error::Result;
use crate::types::{Message, ProviderResponse, ToolDefinition};
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// LLM provider interface.
///
/// Each provider (Anthropic, OpenAI, etc.) implements this trait to normalize
/// the request/response cycle. The agent loop calls `complete()` without
/// knowing which provider is behind it.
pub trait LlmProvider: Send + Sync {
    fn complete<'a>(
        &'a self,
        messages: &'a [Message],
        tools: &'a [ToolDefinition],
        model: &'a str,
        max_tokens: u32,
    ) -> BoxFuture<'a, Result<ProviderResponse>>;

    fn name(&self) -> &str;

    fn supports_model(&self, model: &str) -> bool;
}
