use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition, UsageStats};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub thinking: Option<String>,
    pub tool_calls: Vec<super::types::ToolCall>,
    pub usage: UsageStats,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Identifier of the provider (e.g., "openai", "anthropic", "gemini", "groq", "ollama")
    fn provider_name(&self) -> &str;

    /// Complete request non-streaming
    async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CompletionResponse>;

    /// Stream request producing chunk deltas
    async fn stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>>;

    /// List supported models
    fn supported_models(&self) -> Vec<ModelInfo>;
}
