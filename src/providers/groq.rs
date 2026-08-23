use super::openai::OpenAiProvider;
use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::stream::BoxStream;

#[derive(Clone)]
pub struct GroqProvider {
    inner: OpenAiProvider,
}

impl GroqProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            inner: OpenAiProvider::new(
                api_key,
                Some("https://api.groq.com/openai/v1".to_string()),
                Some("groq".to_string()),
            ),
        }
    }
}

#[async_trait]
impl LlmProvider for GroqProvider {
    fn provider_name(&self) -> &str {
        "groq"
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
                id: "llama-3.3-70b-versatile".to_string(),
                name: "Groq Llama 3.3 70B Versatile (Free Tier 300+ tok/s)".to_string(),
                provider: "groq".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "deepseek-r1-distill-llama-70b".to_string(),
                name: "Groq DeepSeek-R1 Distill 70B (Free Tier)".to_string(),
                provider: "groq".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: true,
                context_window: 128_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "qwen-2.5-coder-32b".to_string(),
                name: "Groq Qwen 2.5 Coder 32B (Free Tier)".to_string(),
                provider: "groq".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
        ]
    }
}
