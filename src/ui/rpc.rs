use crate::agent::AgentEngine;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

pub struct RpcServer;

impl RpcServer {
    pub async fn run(mut agent: AgentEngine, default_model: &str) -> Result<()> {
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        loop {
            let mut line = String::new();
            if handle.read_line(&mut line)? == 0 {
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let req: RpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: serde_json::Value::Null,
                        result: None,
                        error: Some(json!({ "code": -32700, "message": format!("Parse error: {}", e) })),
                    };
                    println!("{}", serde_json::to_string(&err_resp)?);
                    io::stdout().flush()?;
                    continue;
                }
            };

            let resp = match req.method.as_str() {
                "prompt" => {
                    let prompt = req.params["prompt"].as_str().unwrap_or("");
                    let model = req.params["model"].as_str().unwrap_or(default_model);
                    match agent.run_turn(prompt, model, |_chunk| {}).await {
                        Ok(ans) => RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: Some(json!({ "response": ans })),
                            error: None,
                        },
                        Err(e) => RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: None,
                            error: Some(json!({ "code": -32000, "message": e.to_string() })),
                        },
                    }
                }
                "get_memory" => match agent.memory.get_summary() {
                    Ok(summary) => RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!({ "summary": summary })),
                        error: None,
                    },
                    Err(e) => RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: None,
                        error: Some(json!({ "code": -32000, "message": e.to_string() })),
                    },
                },
                "get_score" => match agent.memory.get_score() {
                    Ok(score) => RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!(score)),
                        error: None,
                    },
                    Err(e) => RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: None,
                        error: Some(json!({ "code": -32000, "message": e.to_string() })),
                    },
                },
                "list_models" => {
                    let models = agent.router.list_all_models();
                    RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!(models)),
                        error: None,
                    }
                }
                _ => RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(json!({ "code": -32601, "message": format!("Method not found: {}", req.method) })),
                },
            };

            println!("{}", serde_json::to_string(&resp)?);
            io::stdout().flush()?;
        }

        Ok(())
    }
}
