use super::openai::OpenAiProvider;
use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[derive(Clone)]
pub struct DeepSeekProvider {
    inner: OpenAiProvider,
}

impl DeepSeekProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: OpenAiProvider::new(
                api_key,
                Some("https://api.deepseek.com/v1".to_string()),
                Some("deepseek".to_string()),
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    fn provider_name(&self) -> &str {
        "deepseek"
    }

    async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        self.inner.complete(model, messages, tools, temperature, max_tokens).await
    }

    async fn stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        self.inner.stream(model, messages, tools, temperature, max_tokens).await
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "deepseek-chat".to_string(),
                name: "DeepSeek V3 (Chat / Coding)".to_string(),
                provider: "deepseek".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 64_000,
                input_cost_per_million: 0.14,
                output_cost_per_million: 0.28,
            },
            ModelInfo {
                id: "deepseek-reasoner".to_string(),
                name: "DeepSeek R1 (Deep Reasoning)".to_string(),
                provider: "deepseek".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: true,
                context_window: 64_000,
                input_cost_per_million: 0.55,
                output_cost_per_million: 2.19,
            },
        ]
    }
}
