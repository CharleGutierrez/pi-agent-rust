use super::anthropic::AnthropicProvider;
use super::deepseek::DeepSeekProvider;
use super::gemini::GeminiProvider;
use super::gemini_web::GeminiWebProvider;
use super::groq::GroqProvider;
use super::mistral::MistralProvider;
use super::ollama::OllamaProvider;
use super::openai::OpenAiProvider;
use super::openrouter::OpenRouterProvider;
use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, StreamChunk, ToolDefinition, UsageStats};
use super::xai::XaiProvider;
use crate::config::AppConfig;
use crate::config::config::CustomModelConfig;
use anyhow::{Context, Result};
use futures_util::stream::BoxStream;
use std::collections::HashMap;
use std::sync::Arc;

struct CustomProviderWrapper {
    inner: Arc<dyn LlmProvider>,
    custom_model: CustomModelConfig,
}

#[async_trait::async_trait]
impl LlmProvider for CustomProviderWrapper {
    fn provider_name(&self) -> &str {
        &self.custom_model.provider_type
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
        vec![ModelInfo {
            id: self.custom_model.id.clone(),
            name: self.custom_model.name.clone(),
            provider: format!("custom-{}", self.custom_model.provider_type),
            is_free: false,
            supports_tools: self.custom_model.supports_tools,
            supports_thinking: self.custom_model.supports_thinking,
            context_window: self.custom_model.context_window,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        }]
    }
}

pub struct ProviderRouter {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    default_model: String,
    total_usage: std::sync::Mutex<UsageStats>,
    custom_model_aliases: HashMap<String, (String, String)>,
}

impl ProviderRouter {
    pub fn new(config: &AppConfig) -> Self {
        let mut providers: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();
        let mut custom_model_aliases = HashMap::new();

        // 1. Ollama (Always available locally)
        let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        providers.insert("ollama".to_string(), Arc::new(OllamaProvider::new(Some(ollama_host))));

        // 2. Groq (Free tier API)
        let groq_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
        providers.insert("groq".to_string(), Arc::new(GroqProvider::new(groq_key)));

        // 3. Gemini (Free tier & Paid)
        let gemini_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        let mut gemini_provider = GeminiProvider::new(gemini_key.clone(), None);
        // If API key is missing, see if we have an OAuth token
        if gemini_key.is_empty() {
            if let Ok(Some(mut token)) = crate::auth::GoogleAuth::load_token() {
                // Ignore errors on refresh during router init, it will be caught later
                let _ = futures_executor::block_on(crate::auth::GoogleAuth::refresh_token_if_needed(&mut token));
                gemini_provider.set_oauth_token(token.access_token);
            }
        }
        providers.insert("gemini".to_string(), Arc::new(gemini_provider));

        // 4. OpenRouter (Free tier models + Paid)
        let or_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
        providers.insert("openrouter".to_string(), Arc::new(OpenRouterProvider::new(or_key)));

        // 5. Anthropic
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        providers.insert("anthropic".to_string(), Arc::new(AnthropicProvider::new(anthropic_key, None)));

        // 6. OpenAI
        let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        providers.insert("openai".to_string(), Arc::new(OpenAiProvider::new(openai_key, None, None)));

        // 7. DeepSeek
        let deepseek_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
        providers.insert("deepseek".to_string(), Arc::new(DeepSeekProvider::new(deepseek_key)));

        // 8. xAI
        let xai_key = std::env::var("XAI_API_KEY").unwrap_or_default();
        providers.insert("xai".to_string(), Arc::new(XaiProvider::new(xai_key)));

        // 9. Mistral
        let mistral_key = std::env::var("MISTRAL_API_KEY").unwrap_or_default();
        providers.insert("mistral".to_string(), Arc::new(MistralProvider::new(mistral_key)));

        // 10. Gemini Web (Cookie based)
        if let Ok(cookie) = std::env::var("GEMINI_WEB_COOKIE") {
            if !cookie.is_empty() {
                providers.insert("gemini-web".to_string(), Arc::new(GeminiWebProvider::new(cookie)));
            }
        }

        // 11. Custom Models
        for cm in &config.custom_models {
            let api_key = cm.api_key_env.as_ref().and_then(|env_var| std::env::var(env_var).ok()).unwrap_or_default();
            
            let base_inner: Arc<dyn LlmProvider> = match cm.provider_type.to_lowercase().as_str() {
                "openai" => Arc::new(OpenAiProvider::new(api_key, cm.base_url.clone(), Some(cm.provider_type.clone()))),
                "anthropic" => Arc::new(AnthropicProvider::new(api_key, cm.base_url.clone())),
                "gemini" => Arc::new(GeminiProvider::new(api_key, cm.base_url.clone())),
                "ollama" => Arc::new(OllamaProvider::new(cm.base_url.clone())),
                _ => Arc::new(OpenAiProvider::new(api_key, cm.base_url.clone(), Some(cm.provider_type.clone()))), // Default to OpenAI compatible
            };

            let provider_id = format!("custom_{}", cm.id);
            providers.insert(
                provider_id.clone(),
                Arc::new(CustomProviderWrapper {
                    inner: base_inner,
                    custom_model: cm.clone(),
                }),
            );

            custom_model_aliases.insert(cm.id.clone(), (provider_id, cm.id.clone()));
        }

        let def = config.default_model.clone();

        Self {
            providers,
            default_model: def,
            total_usage: std::sync::Mutex::new(UsageStats::default()),
            custom_model_aliases,
        }
    }

    pub fn resolve_alias(&self, alias_or_name: &str) -> (String, String) {
        let clean = alias_or_name.trim().to_lowercase();
        
        // Check custom model aliases first
        if let Some(resolved) = self.custom_model_aliases.get(&clean) {
            return resolved.clone();
        }

        match clean.as_str() {
            "sonnet" | "claude" | "claude-3-7" => ("anthropic".to_string(), "claude-3-7-sonnet-latest".to_string()),
            "sonnet-3.5" | "claude-3-5" => ("anthropic".to_string(), "claude-3-5-sonnet-latest".to_string()),
            "haiku" => ("anthropic".to_string(), "claude-3-5-haiku-latest".to_string()),
            "4o" | "gpt4" | "gpt-4o" => ("openai".to_string(), "gpt-4o".to_string()),
            "4o-mini" | "mini" => ("openai".to_string(), "gpt-4o-mini".to_string()),
            "o3-mini" | "o3" => ("openai".to_string(), "o3-mini".to_string()),
            "o1" => ("openai".to_string(), "o1".to_string()),
            "4.5" | "gpt-4.5" | "gpt-4.5-preview" => ("openai".to_string(), "gpt-4.5-preview".to_string()),
            "flash" | "gemini" | "gemini-flash" => ("gemini".to_string(), "gemini-2.0-flash".to_string()),
            "gemini-pro" => ("gemini".to_string(), "gemini-1.5-pro".to_string()),
            "gemini-web" => ("gemini-web".to_string(), "gemini-web-latest".to_string()),
            "groq" | "groq-llama" => ("groq".to_string(), "llama-3.3-70b-versatile".to_string()),
            "groq-r1" => ("groq".to_string(), "deepseek-r1-distill-llama-70b".to_string()),
            "groq-qwen" => ("groq".to_string(), "qwen-2.5-coder-32b".to_string()),
            "free-r1" | "openrouter-r1" => ("openrouter".to_string(), "deepseek/deepseek-r1:free".to_string()),
            "free-llama" => ("openrouter".to_string(), "meta-llama/llama-3.3-70b-instruct:free".to_string()),
            "opus" | "claude-opus" => ("anthropic".to_string(), "claude-3-opus-latest".to_string()),
            "grok" | "grok-2" => ("xai".to_string(), "grok-2-latest".to_string()),
            "codestral" | "mistral-code" => ("mistral".to_string(), "codestral-latest".to_string()),
            "mistral-large" => ("mistral".to_string(), "mistral-large-latest".to_string()),
            "r1" | "deepseek-r1" => {
                if self.providers.contains_key("deepseek") {
                    ("deepseek".to_string(), "deepseek-reasoner".to_string())
                } else if self.providers.contains_key("groq") {
                    ("groq".to_string(), "deepseek-r1-distill-llama-70b".to_string())
                } else {
                    ("ollama".to_string(), "deepseek-r1:latest".to_string())
                }
            }
            "deepseek" | "v3" => ("deepseek".to_string(), "deepseek-chat".to_string()),
            "ollama" | "local" => ("ollama".to_string(), "llama3.3:latest".to_string()),
            _ => {
                // Infer by provider prefix
                if clean.starts_with("claude") {
                    ("anthropic".to_string(), alias_or_name.to_string())
                } else if clean.starts_with("gpt") || clean.starts_with("o1") || clean.starts_with("o3") {
                    ("openai".to_string(), alias_or_name.to_string())
                } else if clean.starts_with("gemini") {
                    ("gemini".to_string(), alias_or_name.to_string())
                } else if clean.contains("groq") {
                    ("groq".to_string(), alias_or_name.to_string())
                } else if clean.contains('/') {
                    ("openrouter".to_string(), alias_or_name.to_string())
                } else if clean.starts_with("deepseek") {
                    ("deepseek".to_string(), alias_or_name.to_string())
                } else {
                    ("ollama".to_string(), alias_or_name.to_string())
                }
            }
        }
    }

    pub fn list_all_models(&self) -> Vec<ModelInfo> {
        let mut list = Vec::new();
        for p in self.providers.values() {
            list.extend(p.supported_models());
        }
        list
    }

    pub async fn complete(
        &self,
        model_spec: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        let (provider_id, model_name) = self.resolve_alias(model_spec);
        let provider = self
            .providers
            .get(&provider_id)
            .with_context(|| format!("Provider '{}' is not configured or missing API key", provider_id))?;

        let res = provider
            .complete(&model_name, messages, tools, temperature, max_tokens)
            .await?;

        // Update usage
        let mut u = self.total_usage.lock().unwrap();
        u.prompt_tokens += res.usage.prompt_tokens;
        u.completion_tokens += res.usage.completion_tokens;
        u.total_tokens += res.usage.total_tokens;

        Ok(res)
    }

    pub async fn stream(
        &self,
        model_spec: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        let (provider_id, model_name) = self.resolve_alias(model_spec);
        let provider = self
            .providers
            .get(&provider_id)
            .with_context(|| format!("Provider '{}' is not configured or missing API key", provider_id))?;

        provider
            .stream(&model_name, messages, tools, temperature, max_tokens)
            .await
    }

    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    pub fn set_default_model(&mut self, model: impl Into<String>) {
        self.default_model = model.into();
    }

    pub fn get_total_usage(&self) -> UsageStats {
        self.total_usage.lock().unwrap().clone()
    }
}
