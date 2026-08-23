use super::openai::OpenAiProvider;
use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[derive(Clone)]
pub struct MistralProvider {
    inner: OpenAiProvider,
}

impl MistralProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: OpenAiProvider::new(
                api_key,
                Some("https://api.mistral.ai/v1".to_string()),
                Some("mistral".to_string()),
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for MistralProvider {
    fn provider_name(&self) -> &str {
        "mistral"
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
                id: "codestral-latest".to_string(),
                name: "Mistral Codestral (Coding Specialist)".to_string(),
                provider: "mistral".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 256_000,
                input_cost_per_million: 1.00,
                output_cost_per_million: 3.00,
            },
            ModelInfo {
                id: "mistral-large-latest".to_string(),
                name: "Mistral Large (Flagship)".to_string(),
                provider: "mistral".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 131_000,
                input_cost_per_million: 2.00,
                output_cost_per_million: 6.00,
            },
        ]
    }
}
