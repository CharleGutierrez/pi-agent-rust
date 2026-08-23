use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, Role, StreamChunk, ToolCall, ToolDefinition, UsageStats};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

#[derive(Clone)]
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
        }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        if !self.api_key.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&self.api_key) {
                headers.insert("x-api-key", val);
            }
        }
        headers
    }

    fn extract_system_and_messages(&self, messages: &[Message]) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_prompt = String::new();
        let mut anthropic_messages = Vec::new();

        for m in messages {
            match m.role {
                Role::System => {
                    if !system_prompt.is_empty() {
                        system_prompt.push('\n');
                    }
                    system_prompt.push_str(&m.content);
                }
                Role::User => {
                    anthropic_messages.push(json!({
                        "role": "user",
                        "content": m.content
                    }));
                }
                Role::Assistant => {
                    let mut content_blocks = Vec::new();
                    if !m.content.is_empty() {
                        content_blocks.push(json!({
                            "type": "text",
                            "text": m.content
                        }));
                    }
                    for tc in &m.tool_calls {
                        content_blocks.push(json!({
                            "type": "tool_use",
                            "id": tc.id,
                            "name": tc.name,
                            "input": tc.arguments
                        }));
                    }
                    anthropic_messages.push(json!({
                        "role": "assistant",
                        "content": content_blocks
                    }));
                }
                Role::Tool => {
                    let tool_id = m.tool_call_id.as_deref().unwrap_or("unknown");
                    anthropic_messages.push(json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": tool_id,
                            "content": m.content
                        }]
                    }));
                }
            }
        }

        let sys = if system_prompt.is_empty() { None } else { Some(system_prompt) };
        (sys, anthropic_messages)
    }

    fn serialize_tools(&self, tools: &[ToolDefinition]) -> Option<serde_json::Value> {
        if tools.is_empty() {
            return None;
        }
        let tools_json: Vec<_> = tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                })
            })
            .collect();
        Some(json!(tools_json))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_name(&self) -> &str {
        "anthropic"
    }

    async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        if self.api_key.is_empty() {
            bail!("Missing API key for Anthropic. Please set ANTHROPIC_API_KEY.");
        }

        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));
        let (system_opt, chat_messages) = self.extract_system_and_messages(messages);

        let mut body = json!({
            "model": model,
            "messages": chat_messages,
            "max_tokens": max_tokens.unwrap_or(4096),
        });

        if let Some(sys) = system_opt {
            body["system"] = json!(sys);
        }
        if let Some(temp) = temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }

        let resp = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send request to {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("Anthropic API error [{}]: {}", status, err_text);
        }

        let res_json: serde_json::Value = resp.json().await?;
        let mut content = String::new();
        let mut thinking = None;
        let mut tool_calls = Vec::new();

        if let Some(blocks) = res_json["content"].as_array() {
            for b in blocks {
                match b["type"].as_str() {
                    Some("text") => {
                        content.push_str(b["text"].as_str().unwrap_or(""));
                    }
                    Some("thinking") => {
                        thinking = b["thinking"].as_str().map(|s| s.to_string());
                    }
                    Some("tool_use") => {
                        let id = b["id"].as_str().unwrap_or("").to_string();
                        let name = b["name"].as_str().unwrap_or("").to_string();
                        let arguments = b["input"].clone();
                        tool_calls.push(ToolCall { id, name, arguments });
                    }
                    _ => {}
                }
            }
        }

        let usage = UsageStats {
            prompt_tokens: res_json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: res_json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (res_json["usage"]["input_tokens"].as_u64().unwrap_or(0)
                + res_json["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
            reasoning_tokens: None,
            estimated_cost_usd: None,
        };

        Ok(CompletionResponse {
            content,
            thinking,
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
        if self.api_key.is_empty() {
            bail!("Missing API key for Anthropic. Please set ANTHROPIC_API_KEY.");
        }

        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));
        let (system_opt, chat_messages) = self.extract_system_and_messages(messages);

        let mut body = json!({
            "model": model,
            "messages": chat_messages,
            "max_tokens": max_tokens.unwrap_or(4096),
            "stream": true,
        });

        if let Some(sys) = system_opt {
            body["system"] = json!(sys);
        }
        if let Some(temp) = temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }

        let resp = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send stream to {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("Anthropic Streaming API error [{}]: {}", status, err_text);
        }

        let event_stream = resp.bytes_stream().eventsource();
        let mapped = event_stream.map(|event_res| {
            match event_res {
                Ok(event) => {
                    let data = event.data;
                    match serde_json::from_str::<serde_json::Value>(&data) {
                        Ok(v) => {
                            let event_type = v["type"].as_str().unwrap_or("");
                            match event_type {
                                "content_block_delta" => {
                                    let delta = &v["delta"];
                                    match delta["type"].as_str() {
                                        Some("text_delta") => {
                                            let txt = delta["text"].as_str().unwrap_or("").to_string();
                                            Ok(StreamChunk::ContentDelta(txt))
                                        }
                                        Some("thinking_delta") => {
                                            let txt = delta["thinking"].as_str().unwrap_or("").to_string();
                                            Ok(StreamChunk::ThinkingDelta(txt))
                                        }
                                        Some("input_json_delta") => {
                                            let index = v["index"].as_u64().unwrap_or(0) as usize;
                                            let partial_json = delta["partial_json"].as_str().unwrap_or("").to_string();
                                            Ok(StreamChunk::ToolCallDelta {
                                                index,
                                                id: None,
                                                name: None,
                                                arguments_delta: partial_json,
                                            })
                                        }
                                        _ => Ok(StreamChunk::ContentDelta(String::new())),
                                    }
                                }
                                "content_block_start" => {
                                    let block = &v["content_block"];
                                    if block["type"] == "tool_use" {
                                        let index = v["index"].as_u64().unwrap_or(0) as usize;
                                        let id = block["id"].as_str().map(|s| s.to_string());
                                        let name = block["name"].as_str().map(|s| s.to_string());
                                        Ok(StreamChunk::ToolCallDelta {
                                            index,
                                            id,
                                            name,
                                            arguments_delta: String::new(),
                                        })
                                    } else {
                                        Ok(StreamChunk::ContentDelta(String::new()))
                                    }
                                }
                                "message_stop" => Ok(StreamChunk::Done),
                                _ => Ok(StreamChunk::ContentDelta(String::new())),
                            }
                        }
                        Err(_) => Ok(StreamChunk::ContentDelta(String::new())),
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Anthropic SSE Error: {}", e)),
            }
        });

        Ok(Box::pin(mapped))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-3-7-sonnet-latest".to_string(),
                name: "Claude 3.7 Sonnet (Hybrid Reasoning)".to_string(),
                provider: "anthropic".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: true,
                context_window: 200_000,
                input_cost_per_million: 3.00,
                output_cost_per_million: 15.00,
            },
            ModelInfo {
                id: "claude-3-5-sonnet-latest".to_string(),
                name: "Claude 3.5 Sonnet v2".to_string(),
                provider: "anthropic".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 200_000,
                input_cost_per_million: 3.00,
                output_cost_per_million: 15.00,
            },
            ModelInfo {
                id: "claude-3-5-haiku-latest".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                provider: "anthropic".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 200_000,
                input_cost_per_million: 0.80,
                output_cost_per_million: 4.00,
            },
            ModelInfo {
                id: "claude-3-opus-latest".to_string(),
                name: "Claude 3 Opus".to_string(),
                provider: "anthropic".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 200_000,
                input_cost_per_million: 15.00,
                output_cost_per_million: 75.00,
            },
        ]
    }
}
