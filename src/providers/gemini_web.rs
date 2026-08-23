use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, Role, StreamChunk, ToolDefinition};
use anyhow::{bail, Result};
use async_trait::async_trait;
use futures_util::stream::BoxStream;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use std::time::Duration;

#[derive(Clone)]
#[allow(dead_code)]
pub struct GeminiWebProvider {
    client: reqwest::Client,
    cookie_1psid: String,
    cookie_1psidts: Option<String>,
}

impl GeminiWebProvider {
    pub fn new(cookie: impl Into<String>) -> Self {
        let raw_cookie = cookie.into();
        
        // Basic parsing if user passes full cookie string
        let mut psid = raw_cookie.clone();
        let mut psidts = None;
        
        if raw_cookie.contains("__Secure-1PSID=") {
            for part in raw_cookie.split(';') {
                let trimmed = part.trim();
                if trimmed.starts_with("__Secure-1PSID=") {
                    psid = trimmed.replace("__Secure-1PSID=", "");
                } else if trimmed.starts_with("__Secure-1PSIDTS=") {
                    psidts = Some(trimmed.replace("__Secure-1PSIDTS=", ""));
                }
            }
        }

        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            cookie_1psid: psid,
            cookie_1psidts: psidts,
        }
    }

    #[allow(dead_code)]
    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let mut cookie_str = format!("__Secure-1PSID={}", self.cookie_1psid);
        if let Some(ts) = &self.cookie_1psidts {
            cookie_str.push_str(&format!("; __Secure-1PSIDTS={}", ts));
        }
        
        if let Ok(val) = HeaderValue::from_str(&cookie_str) {
            headers.insert(COOKIE, val);
        }
        headers
    }
    
    fn format_prompt(&self, messages: &[Message]) -> String {
        let mut full_prompt = String::new();
        for m in messages {
            let role = match m.role {
                Role::System => "System Instruction",
                Role::User => "User",
                Role::Assistant => "Assistant",
                Role::Tool => "Tool Result",
            };
            full_prompt.push_str(&format!("{}: {}\n\n", role, m.content));
        }
        full_prompt
    }
}

#[async_trait]
impl LlmProvider for GeminiWebProvider {
    fn provider_name(&self) -> &str {
        "gemini-web"
    }

    async fn complete(
        &self,
        _model: &str,
        messages: &[Message],
        _tools: &[ToolDefinition],
        _temperature: Option<f32>,
        _max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        if self.cookie_1psid.is_empty() {
            bail!("Missing GEMINI_WEB_COOKIE. To use the Gemini Web interface, you must provide your __Secure-1PSID cookie.");
        }

        let _prompt = self.format_prompt(messages);

        // Note: Fully reverse-engineering Google's batchexecute RPC (SNlM0e tokens, f.req nested arrays) 
        // in pure Rust without a headless browser is brittle as the internal protobuf arrays change weekly.
        // We simulate the failure/instruction here to guide the user to the free official API, 
        // while laying the architectural foundation for cookie-based web integrations.
        
        bail!("Gemini Web UI (Cookie-based) integration is architecturally scaffolded, but Google's undocumented batchexecute RPC format changes frequently.\n\n💡 TIP: You do NOT need to use cookies to get Gemini for free! Google provides official FREE API keys via Google AI Studio (gemini-2.0-flash is 100% free).\n\nAction: Get a free key at aistudio.google.com and set GEMINI_API_KEY instead.");
    }

    async fn stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        // Fallback to non-streaming for the error message
        let res = self.complete(model, messages, tools, temperature, max_tokens).await?;
        
        // If it somehow succeeds, return a stream of one chunk
        let stream = futures_util::stream::iter(vec![
            Ok(StreamChunk::ContentDelta(res.content)),
            Ok(StreamChunk::Done)
        ]);
        
        Ok(Box::pin(stream))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-web-latest".to_string(),
                name: "Gemini Web UI (Cookie based)".to_string(),
                provider: "gemini-web".to_string(),
                is_free: true,
                supports_tools: false, // Web UI scraping doesn't reliably support native tool calls
                supports_thinking: false,
                context_window: 32_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            }
        ]
    }
}
