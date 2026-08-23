use super::openai::OpenAiProvider;
use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[derive(Clone)]
pub struct OpenRouterProvider {
    inner: OpenAiProvider,
}

impl OpenRouterProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: OpenAiProvider::new(
                api_key,
                Some("https://openrouter.ai/api/v1".to_string()),
                Some("openrouter".to_string()),
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    fn provider_name(&self) -> &str {
        "openrouter"
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
                id: "deepseek/deepseek-r1:free".to_string(),
                name: "OpenRouter DeepSeek R1 (100% Free)".to_string(),
                provider: "openrouter".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: true,
                context_window: 64_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "meta-llama/llama-3.3-70b-instruct:free".to_string(),
                name: "OpenRouter Llama 3.3 70B (100% Free)".to_string(),
                provider: "openrouter".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "anthropic/claude-3.7-sonnet".to_string(),
                name: "OpenRouter Claude 3.7 Sonnet".to_string(),
                provider: "openrouter".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: true,
                context_window: 200_000,
                input_cost_per_million: 3.00,
                output_cost_per_million: 15.00,
            },
        ]
    }
}
