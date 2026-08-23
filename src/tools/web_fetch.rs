use super::traits::AgentTool;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::json;

pub struct WebFetchTool {
    client: reqwest::Client,
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Pi-Coding-Agent-Rust/1.0")
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl AgentTool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch raw content, documentation, or JSON API data from a URL."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to fetch content from"
                },
                "max_length": {
                    "type": "integer",
                    "description": "Maximum character length to return (default 10000)"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let url = match args["url"].as_str() {
            Some(u) => u,
            None => bail!("Missing required parameter: 'url'"),
        };

        let max_length = args["max_length"].as_u64().unwrap_or(10_000) as usize;

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch URL: {}", url))?;

        if !resp.status().is_success() {
            bail!("HTTP error status: {}", resp.status());
        }

        let body = resp.text().await?;
        if body.len() > max_length {
            let truncated = &body[..max_length];
            Ok(format!("{}\n... [Content truncated after {} characters]", truncated, max_length))
        } else {
            Ok(body)
        }
    }
}
