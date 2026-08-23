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
pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    provider_id: String,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>, base_url: Option<String>, provider_id: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            provider_id: provider_id.unwrap_or_else(|| "openai".to_string()),
        }
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !self.api_key.is_empty() {
            let auth = format!("Bearer {}", self.api_key);
            if let Ok(val) = HeaderValue::from_str(&auth) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }

    fn serialize_messages(&self, messages: &[Message]) -> serde_json::Value {
        let mut arr = Vec::new();
        for m in messages {
            let role_str = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };

            let mut obj = json!({
                "role": role_str,
                "content": m.content,
            });

            if !m.tool_calls.is_empty() {
                let tool_calls_json: Vec<_> = m
                    .tool_calls
                    .iter()
                    .map(|tc| {
                        json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments.to_string(),
                            }
                        })
                    })
                    .collect();
                obj["tool_calls"] = json!(tool_calls_json);
            }

            if let Some(tool_call_id) = &m.tool_call_id {
                obj["tool_call_id"] = json!(tool_call_id);
            }

            arr.push(obj);
        }
        json!(arr)
    }

    fn serialize_tools(&self, tools: &[ToolDefinition]) -> Option<serde_json::Value> {
        if tools.is_empty() {
            return None;
        }
        let tools_json: Vec<_> = tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            })
            .collect();
        Some(json!(tools_json))
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn provider_name(&self) -> &str {
        &self.provider_id
    }

    async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        if self.api_key.is_empty() && !self.base_url.contains("127.0.0") && !self.base_url.contains("localhost") {
            bail!("Missing API key for {}. Please set the environment variable.", self.provider_id);
        }

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut body = json!({
            "model": model,
            "messages": self.serialize_messages(messages),
        });

        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }
        if let Some(temp) = temperature {
            // Some reasoning models (e.g. o1/o3-mini) don't accept temperature
            if !model.starts_with("o1") && !model.starts_with("o3") {
                body["temperature"] = json!(temp);
            }
        }
        if let Some(max) = max_tokens {
            if model.starts_with("o1") || model.starts_with("o3") {
                body["max_completion_tokens"] = json!(max);
            } else {
                body["max_tokens"] = json!(max);
            }
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
            bail!("OpenAI API error [{}]: {}", status, err_text);
        }

        let res_json: serde_json::Value = resp.json().await?;
        let choice = res_json["choices"]
            .get(0)
            .with_context(|| "Empty choices in OpenAI response")?;

        let message = &choice["message"];
        let content = message["content"].as_str().unwrap_or("").to_string();
        let thinking = message["reasoning_content"].as_str().map(|s| s.to_string());

        let mut tool_calls = Vec::new();
        if let Some(tc_array) = message["tool_calls"].as_array() {
            for tc in tc_array {
                let id = tc["id"].as_str().unwrap_or("").to_string();
                let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                let args_raw = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let arguments = serde_json::from_str(args_raw).unwrap_or(json!({}));
                tool_calls.push(ToolCall { id, name, arguments });
            }
        }

        let usage = UsageStats {
            prompt_tokens: res_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: res_json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: res_json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
            reasoning_tokens: res_json["usage"]["completion_tokens_details"]["reasoning_tokens"].as_u64().map(|v| v as u32),
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
        if self.api_key.is_empty() && !self.base_url.contains("127.0.0") && !self.base_url.contains("localhost") {
            bail!("Missing API key for {}. Please set the environment variable.", self.provider_id);
        }

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut body = json!({
            "model": model,
            "messages": self.serialize_messages(messages),
            "stream": true,
            "stream_options": { "include_usage": true }
        });

        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }
        if let Some(temp) = temperature {
            if !model.starts_with("o1") && !model.starts_with("o3") {
                body["temperature"] = json!(temp);
            }
        }
        if let Some(max) = max_tokens {
            if model.starts_with("o1") || model.starts_with("o3") {
                body["max_completion_tokens"] = json!(max);
            } else {
                body["max_tokens"] = json!(max);
            }
        }

        let resp = self
            .client
            .post(&url)
            .headers(self.build_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to send stream request to {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("OpenAI Streaming API error [{}]: {}", status, err_text);
        }

        let event_stream = resp.bytes_stream().eventsource();
        let mapped = event_stream.map(|event_res| {
            match event_res {
                Ok(event) => {
                    let data = event.data;
                    if data == "[DONE]" {
                        return Ok(StreamChunk::Done);
                    }
                    match serde_json::from_str::<serde_json::Value>(&data) {
                        Ok(v) => {
                            if let Some(usage_obj) = v.get("usage").and_then(|u| u.as_object()) {
                                if let Some(total) = usage_obj.get("total_tokens").and_then(|t| t.as_u64()) {
                                    return Ok(StreamChunk::Usage(UsageStats {
                                        prompt_tokens: usage_obj.get("prompt_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32,
                                        completion_tokens: usage_obj.get("completion_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32,
                                        total_tokens: total as u32,
                                        reasoning_tokens: None,
                                        estimated_cost_usd: None,
                                    }));
                                }
                            }

                            if let Some(choice) = v["choices"].get(0) {
                                let delta = &choice["delta"];
                                if let Some(thinking) = delta["reasoning_content"].as_str() {
                                    return Ok(StreamChunk::ThinkingDelta(thinking.to_string()));
                                }
                                if let Some(content) = delta["content"].as_str() {
                                    return Ok(StreamChunk::ContentDelta(content.to_string()));
                                }
                                if let Some(tc_array) = delta["tool_calls"].as_array() {
                                    if let Some(tc) = tc_array.get(0) {
                                        let index = tc["index"].as_u64().unwrap_or(0) as usize;
                                        let id = tc["id"].as_str().map(|s| s.to_string());
                                        let name = tc["function"]["name"].as_str().map(|s| s.to_string());
                                        let arguments_delta = tc["function"]["arguments"].as_str().unwrap_or("").to_string();
                                        return Ok(StreamChunk::ToolCallDelta {
                                            index,
                                            id,
                                            name,
                                            arguments_delta,
                                        });
                                    }
                                }
                            }
                            Ok(StreamChunk::ContentDelta(String::new()))
                        }
                        Err(_) => Ok(StreamChunk::ContentDelta(String::new())),
                    }
                }
                Err(e) => Err(anyhow::anyhow!("SSE Error: {}", e)),
            }
        });

        Ok(Box::pin(mapped))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "gpt-4o".to_string(),
                name: "GPT-4o (Omni)".to_string(),
                provider: "openai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 2.50,
                output_cost_per_million: 10.00,
            },
            ModelInfo {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o Mini".to_string(),
                provider: "openai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 0.15,
                output_cost_per_million: 0.60,
            },
            ModelInfo {
                id: "o3-mini".to_string(),
                name: "o3 Mini Reasoning".to_string(),
                provider: "openai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: true,
                context_window: 200_000,
                input_cost_per_million: 1.10,
                output_cost_per_million: 4.40,
            },
            ModelInfo {
                id: "gpt-4.5-preview".to_string(),
                name: "GPT-4.5 Preview".to_string(),
                provider: "openai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 75.00,
                output_cost_per_million: 150.00,
            },
            ModelInfo {
                id: "o1".to_string(),
                name: "o1 Advanced Reasoning".to_string(),
                provider: "openai".to_string(),
                is_free: false,
                supports_tools: true,
                supports_thinking: true,
                context_window: 200_000,
                input_cost_per_million: 15.00,
                output_cost_per_million: 60.00,
            },
        ]
    }
}
