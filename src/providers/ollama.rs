use super::traits::{CompletionResponse, LlmProvider};
use super::types::{Message, ModelInfo, Role, StreamChunk, ToolCall, ToolDefinition, UsageStats};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

#[derive(Clone)]
pub struct OllamaProvider {
    client: reqwest::Client,
    host: String,
}

impl OllamaProvider {
    pub fn new(host: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            host: host.unwrap_or_else(|| "http://127.0.0.1:11434".to_string()),
        }
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
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments,
                            }
                        })
                    })
                    .collect();
                obj["tool_calls"] = json!(tool_calls_json);
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
impl LlmProvider for OllamaProvider {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    async fn complete(
        &self,
        model: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
        temperature: Option<f32>,
        _max_tokens: Option<u32>,
    ) -> Result<CompletionResponse> {
        let url = format!("{}/api/chat", self.host.trim_end_matches('/'));
        let mut body = json!({
            "model": model,
            "messages": self.serialize_messages(messages),
            "stream": false,
        });

        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }
        if let Some(temp) = temperature {
            body["options"] = json!({ "temperature": temp });
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to connect to Ollama at {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("Ollama error [{}]: {}", status, err_text);
        }

        let res_json: serde_json::Value = resp.json().await?;
        let message = &res_json["message"];
        let content = message["content"].as_str().unwrap_or("").to_string();

        let mut tool_calls = Vec::new();
        if let Some(tc_array) = message["tool_calls"].as_array() {
            for tc in tc_array {
                let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                let arguments = tc["function"]["arguments"].clone();
                let id = format!("call_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string());
                tool_calls.push(ToolCall { id, name, arguments });
            }
        }

        let usage = UsageStats {
            prompt_tokens: res_json["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
            completion_tokens: res_json["eval_count"].as_u64().unwrap_or(0) as u32,
            total_tokens: (res_json["prompt_eval_count"].as_u64().unwrap_or(0)
                + res_json["eval_count"].as_u64().unwrap_or(0)) as u32,
            reasoning_tokens: None,
            estimated_cost_usd: Some(0.0),
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
        _max_tokens: Option<u32>,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        let url = format!("{}/api/chat", self.host.trim_end_matches('/'));
        let mut body = json!({
            "model": model,
            "messages": self.serialize_messages(messages),
            "stream": true,
        });

        if let Some(tools_val) = self.serialize_tools(tools) {
            body["tools"] = tools_val;
        }
        if let Some(temp) = temperature {
            body["options"] = json!({ "temperature": temp });
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Failed to stream from Ollama at {}", url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            bail!("Ollama stream error [{}]: {}", status, err_text);
        }

        let byte_stream = resp.bytes_stream();
        let mapped = byte_stream.map(|chunk_res| {
            match chunk_res {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            if v["done"].as_bool().unwrap_or(false) {
                                return Ok(StreamChunk::Done);
                            }
                            if let Some(c) = v["message"]["content"].as_str() {
                                return Ok(StreamChunk::ContentDelta(c.to_string()));
                            }
                        }
                    }
                    Ok(StreamChunk::ContentDelta(String::new()))
                }
                Err(e) => Err(anyhow::anyhow!("Ollama stream error: {}", e)),
            }
        });

        Ok(Box::pin(mapped))
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "llama3.3:latest".to_string(),
                name: "Ollama Llama 3.3 70B (Local & Free)".to_string(),
                provider: "ollama".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: false,
                context_window: 128_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "deepseek-r1:latest".to_string(),
                name: "Ollama DeepSeek-R1 (Local & Free)".to_string(),
                provider: "ollama".to_string(),
                is_free: true,
                supports_tools: true,
                supports_thinking: true,
                context_window: 64_000,
                input_cost_per_million: 0.0,
                output_cost_per_million: 0.0,
            },
            ModelInfo {
                id: "qwen2.5-coder:latest".to_string(),
                name: "Ollama Qwen 2.5 Coder (Local & Free)".to_string(),
                provider: "ollama".to_string(),
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
