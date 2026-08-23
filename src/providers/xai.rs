use super::openai::OpenAiProvider;
use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[derive(Clone)]
pub struct XaiProvider {
    inner: OpenAiProvider,
}

impl XaiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: OpenAiProvider::new(
                api_key,
                Some("https://api.x.ai/v1".to_string()),
                Some("xai".to_string()),
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for XaiProvider {
    fn provider_name(&self) -> &str {
        "xai"
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
                id: "grok-2-latest".to_string(),
                name: "xAI Grok 2 (Latest)".to_string(),
                provider: "xai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 131_072,
                input_cost_per_million: 2.00,
                output_cost_per_million: 10.00,
            },
            ModelInfo {
                id: "grok-2-vision-latest".to_string(),
                name: "xAI Grok 2 Vision".to_string(),
                provider: "xai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 32_768,
                input_cost_per_million: 2.00,
                output_cost_per_million: 10.00,
            },
        ]
    }
}
