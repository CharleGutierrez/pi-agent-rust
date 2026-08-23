use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, Role, StreamChunk, ToolCall, ToolDefinition, UsageStats};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

#[derive(Clone)]
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    oauth_token: Option<String>,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            oauth_token: None,
        }
    }

    pub fn set_oauth_token(&mut self, token: String) {
        self.oauth_token = Some(token);
    }

    fn serialize_contents(&self, messages: &[Message]) -> (Option<serde_json::Value>, Vec<serde_json::Value>) {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for m in messages {
            match m.role {
                Role::System => {
                    system_instruction = Some(json!({
                        "parts": [{ "text": m.content }]
                    }));
                }
                Role::User => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [{ "text": m.content }]
                    }));
                }
                Role::Assistant => {
                    let mut parts = Vec::new();
                    if !m.content.is_empty() {
                        parts.push(json!({ "text": m.content }));
                    }
                    for tc in &m.tool_calls {
                        parts.push(json!({
                            "functionCall": {
                                "name": tc.name,
                                "args": tc.arguments
                            }
                        }));
                    }
                    contents.push(json!({
                        "role": "model",
                        "parts": parts
                    }));
                }
                Role::Tool => {
                    let tool_id = m.tool_call_id.as_deref().unwrap_or("unknown");
                    let content_json = serde_json::from_str::<serde_json::Value>(&m.content)
                        .unwrap_or_else(|_| json!({ "result": m.content }));
                    contents.push(json!({
                        "role": "user",
                        "parts": [{
                            "functionResponse": {
                                "name": tool_id,
                                "response": content_json
                            }
                        }]
                    }));
                }
            }
        }

        (system_instruction, contents)
    }

    fn serialize_tools(&self, tools: &[ToolDefinition]) -> Option<serde_json::Value> {
        if tools.is_empty() {
            return None;
        }
        let decls: Vec<_> = tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                })
            })
            .collect();
        Some(json!([{ "functionDeclarations": decls }]))
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn provider_name(&self) -> &str {
        "gemini"
    }

    async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        if self.api_key.is_empty() && self.oauth_token.is_none() {
            bail!("Missing API key or OAuth Token for Gemini. Please set GEMINI_API_KEY or run `pi-agent login gemini`.");
        }

        let url = if self.oauth_token.is_some() {
            format!("{}/models/{}:generateContent", self.base_url.trim_end_matches('/'), model)
        } else {
            format!("{}/models/{}:generateContent?key={}", self.base_url.trim_end_matches('/'), model, self.api_key)
        };

        let (sys_opt, contents) = self.serialize_contents(messages);
        let mut body = json!({
            "contents": contents,
        });

        if let Some(sys) = sys_opt {
            body["systemInstruction"] = sys;
        }
        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }

        let mut gen_config = json!({});
        if let Some(temp) = temperature {
            gen_config["temperature"] = json!(temp);
        }
        if let Some(max) = max_tokens {
            gen_config["maxOutputTokens"] = json!(max);
        }
        if !gen_config.as_object().unwrap().is_empty() {
            body["generationConfig"] = gen_config;
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(token) = &self.oauth_token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to Gemini {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("Gemini API error [{}]: {}", status, err_text);
        }

        let res_json: serde_json::Value = resp.json().await?;
        let candidate = res_json["candidates"]
            .get(0)
            .with_context(|| "No candidate returned from Gemini")?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(parts) = candidate["content"]["parts"].as_array() {
            for p in parts {
                if let Some(txt) = p["text"].as_str() {
                    content.push_str(txt);
                }
                if let Some(fc) = p.get("functionCall") {
                    let name = fc["name"].as_str().unwrap_or("").to_string();
                    let args = fc.get("args").cloned().unwrap_or(json!({}));
                    tool_calls.push(ToolCall {
                        id: format!("call_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string()),
                        name,
                        arguments: args,
                    });
                }
            }
        }

        let usage = UsageStats {
            prompt_tokens: res_json["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion_tokens: res_json["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total_tokens: res_json["usageMetadata"]["totalTokenCount"].as_u64().unwrap_or(0) as u32,
            reasoning_tokens: None,
            estimated_cost_usd: None,
        };

        Ok(CompletionResponse {
            content,
            thinking: None,
            tool_calls,
            usage,
        })
    }

    async fn stream(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        if self.api_key.is_empty() && self.oauth_token.is_none() {
            bail!("Missing API key or OAuth Token for Gemini. Please set GEMINI_API_KEY or run `pi-agent login gemini`.");
        }

        let url = if self.oauth_token.is_some() {
            format!("{}/models/{}:streamGenerateContent?alt=sse", self.base_url.trim_end_matches('/'), model)
        } else {
            format!("{}/models/{}:streamGenerateContent?alt=sse&key={}", self.base_url.trim_end_matches('/'), model, self.api_key)
        };

        let (sys_opt, contents) = self.serialize_contents(messages);
        let mut body = json!({
            "contents": contents,
        });

        if let Some(sys) = sys_opt {
            body["systemInstruction"] = sys;
        }
        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }

        let mut gen_config = json!({});
        if let Some(temp) = temperature {
            gen_config["temperature"] = json!(temp);
        }
        if let Some(max) = max_tokens {
            gen_config["maxOutputTokens"] = json!(max);
        }
        if !gen_config.as_object().unwrap().is_empty() {
            body["generationConfig"] = gen_config;
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(token) = &self.oauth_token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to stream request to Gemini {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("Gemini Streaming API error [{}]: {}", status, err_text);
        }

        let event_stream = resp.bytes_stream().eventsource();
        let mapped = event_stream.map(|event_res| {
            match event_res {
                Ok(event) => {
                    let data = event.data;
                    match serde_json::from_str::<serde_json::Value>(&data) {
                        Ok(v) => {
                            if let Some(candidates) = v["candidates"].as_array() {
                                if let Some(candidate) = candidates.get(0) {
                                    if let Some(parts) = candidate["content"]["parts"].as_array() {
                                        let mut combined_text = String::new();
                                        for p in parts {
                                            if let Some(txt) = p["text"].as_str() {
                                                combined_text.push_str(txt);
                                            }
                                            if let Some(fc) = p.get("functionCall") {
                                                let name = fc["name"].as_str().unwrap_or("").to_string();
                                                let args = fc.get("args").cloned().unwrap_or(json!({}));
                                                return Ok(StreamChunk::ToolCallDelta {
                                                    index: 0,
                                                    id: Some(format!("call_{}", uuid::Uuid::new_v4())),
                                                    name: Some(name),
                                                    arguments_delta: args.to_string(),
                                                });
                                            }
                                        }
                                        return Ok(StreamChunk::ContentDelta(combined_text));
                                    }
                                }
                            }
                            Ok(StreamChunk::ContentDelta(String::new()))
                        }
                        Err(_) => Ok(StreamChunk::ContentDelta(String::new())),
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Gemini SSE Error: {}", e)),
            }
        });

        Ok(Box::pin(mapped))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gemini-2.0-flash".to_string(),
                name: "Google Gemini 2.0 Flash (Free Tier)".to_string(),
                provider: "gemini".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: true,
                context_window: 1_000_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "gemini-1.5-pro".to_string(),
                name: "Google Gemini 1.5 Pro (2M context)".to_string(),
                provider: "gemini".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 2_000_000,
                input_cost_per_million: 1.25,
                output_cost_per_million: 5.00,
            },
        ]
    }
}
