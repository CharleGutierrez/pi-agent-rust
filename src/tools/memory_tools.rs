use super::super::memory::{AttemptOutcome, MemoryEngine};
use super::traits::AgentTool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

pub struct MemoryToolWrapper {
    tool_name: String,
    description: String,
    schema: serde_json::Value,
    engine: Arc<MemoryEngine>,
}

impl MemoryToolWrapper {
    pub fn new(
        tool_name: impl Into<String>,
        description: impl Into<String>,
        schema: serde_json::Value,
        engine: Arc<MemoryEngine>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            description: description.into(),
            schema,
            engine,
        }
    }

    pub fn create_all(engine: Arc<MemoryEngine>) -> Vec<Box<dyn AgentTool>> {
        vec![
            // 1. log_issue
            Box::new(MemoryToolWrapper::new(
                "log_issue",
                "Open a new issue in project memory BEFORE writing fix code. Returns the issue ID.",
                json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "One-line description of the bug or unexpected behavior" },
                        "location": { "type": "string", "description": "Optional file path or component where the issue manifests" }
                    },
                    "required": ["summary"]
                }),
                engine.clone(),
            )),
            // 2. record_attempt
            Box::new(MemoryToolWrapper::new(
                "record_attempt",
                "Record a fix attempt on the current issue immediately after trying it. Outcome must be 'worked', 'failed', or 'partial'.",
                json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "One-line description of what you tried" },
                        "outcome": { "type": "string", "enum": ["worked", "failed", "partial"], "description": "Result of the attempt" },
                        "issue_id": { "type": "string", "description": "Optional issue ID (e.g. '0001')" },
                        "location": { "type": "string", "description": "Optional file path touched" }
                    },
                    "required": ["summary", "outcome"]
                }),
                engine.clone(),
            )),
            // 3. record_fix
            Box::new(MemoryToolWrapper::new(
                "record_fix",
                "Record a confirmed fix and close an issue after evidence (e.g. tests pass).",
                json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "One-line description of the confirmed fix" },
                        "issue_id": { "type": "string", "description": "Optional issue ID to close" },
                        "location": { "type": "string", "description": "Optional file path where the fix was applied" }
                    },
                    "required": ["summary"]
                }),
                engine.clone(),
            )),
            // 4. add_decision
            Box::new(MemoryToolWrapper::new(
                "add_decision",
                "Record an architectural or product decision permanently into memory.",
                json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "One-line description of the decision" },
                        "location": { "type": "string", "description": "Optional file path or scope" },
                        "supersedes": { "type": "string", "description": "Optional event ID of an older decision being replaced" }
                    },
                    "required": ["summary"]
                }),
                engine.clone(),
            )),
            // 5. add_note
            Box::new(MemoryToolWrapper::new(
                "add_note",
                "Record a gotcha, constraint, or environment detail.",
                json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "One-line description of the gotcha or note" },
                        "location": { "type": "string", "description": "Optional file path or library" }
                    },
                    "required": ["summary"]
                }),
                engine.clone(),
            )),
            // 6. precheck_file
            Box::new(MemoryToolWrapper::new(
                "precheck_file",
                "Check a file's failure history, open issues, and dead-ends BEFORE modifying it.",
                json!({
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Project-relative file path to check" }
                    },
                    "required": ["file_path"]
                }),
                engine.clone(),
            )),
            // 7. get_summary
            Box::new(MemoryToolWrapper::new(
                "get_summary",
                "Read the distilled project memory summary without re-scanning files.",
                json!({
                    "type": "object",
                    "properties": {}
                }),
                engine.clone(),
            )),
            // 8. get_project_map
            Box::new(MemoryToolWrapper::new(
                "get_project_map",
                "Read PROJECT_MAP.md to understand the repo structure, files, and relationships.",
                json!({
                    "type": "object",
                    "properties": {}
                }),
                engine.clone(),
            )),
            // 9. get_plan
            Box::new(MemoryToolWrapper::new(
                "get_plan",
                "Read plan.md (the project's intent: ideas, active plans, next items).",
                json!({
                    "type": "object",
                    "properties": {}
                }),
                engine.clone(),
            )),
            // 10. search_events
            Box::new(MemoryToolWrapper::new(
                "search_events",
                "Search across all logged persistent memory events using query keywords.",
                json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Query keyword to search for" },
                        "limit": { "type": "integer", "description": "Maximum events to return (default 10)" }
                    },
                    "required": ["query"]
                }),
                engine.clone(),
            )),
            // 11. get_score
            Box::new(MemoryToolWrapper::new(
                "get_score",
                "Get the failure-prevention score, ROI, hours saved, and dollars protected.",
                json!({
                    "type": "object",
                    "properties": {}
                }),
                engine.clone(),
            )),
            // 12. get_context
            Box::new(MemoryToolWrapper::new(
                "get_context",
                "Generate a token-budgeted memory context block for prompt injection.",
                json!({
                    "type": "object",
                    "properties": {
                        "tokens": { "type": "integer", "description": "Approximate token budget (default 2000)" },
                        "focus": { "type": "string", "description": "Optional focus keyword or path prefix" }
                    }
                }),
                engine.clone(),
            )),
        ]
    }
}

#[async_trait]
impl AgentTool for MemoryToolWrapper {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.schema.clone()
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        match self.tool_name.as_str() {
            "log_issue" => {
                let summary = args["summary"].as_str().unwrap_or("");
                let location = args["location"].as_str();
                let id = self.engine.log_issue(summary, location)?;
                Ok(format!("Opened issue #{}: '{}'", id, summary))
            }
            "record_attempt" => {
                let summary = args["summary"].as_str().unwrap_or("");
                let outcome_str = args["outcome"].as_str().unwrap_or("failed");
                let outcome = match outcome_str {
                    "worked" => AttemptOutcome::Worked,
                    "partial" => AttemptOutcome::Partial,
                    _ => AttemptOutcome::Failed,
                };
                let issue_id = args["issue_id"].as_str();
                let location = args["location"].as_str();
                let evt_id = self.engine.record_attempt(summary, outcome, issue_id, location)?;
                Ok(format!("Recorded attempt [{}] on issue {:?}: '{}' ({})", outcome_str, issue_id, summary, evt_id))
            }
            "record_fix" => {
                let summary = args["summary"].as_str().unwrap_or("");
                let issue_id = args["issue_id"].as_str();
                let location = args["location"].as_str();
                let evt_id = self.engine.record_fix(summary, issue_id, location)?;
                Ok(format!("Recorded fix and closed issue {:?}: '{}' ({})", issue_id, summary, evt_id))
            }
            "add_decision" => {
                let summary = args["summary"].as_str().unwrap_or("");
                let location = args["location"].as_str();
                let supersedes = args["supersedes"].as_str();
                let evt_id = self.engine.add_decision(summary, location, supersedes)?;
                Ok(format!("Recorded decision: '{}' ({})", summary, evt_id))
            }
            "add_note" => {
                let summary = args["summary"].as_str().unwrap_or("");
                let location = args["location"].as_str();
                let evt_id = self.engine.add_note(summary, location)?;
                Ok(format!("Recorded note: '{}' ({})", summary, evt_id))
            }
            "precheck_file" => {
                let file_path = args["file_path"].as_str().unwrap_or("");
                if file_path.is_empty() {
                    bail!("Missing 'file_path'");
                }
                let report = self.engine.precheck_file(file_path)?;
                Ok(report.guidance)
            }
            "get_summary" => {
                self.engine.get_summary()
            }
            "get_project_map" => {
                let map_path = self.engine.storage.memory_dir().join(super::super::memory::storage::MAP_FILE);
                if map_path.exists() {
                    Ok(std::fs::read_to_string(&map_path)?)
                } else {
                    Ok("No PROJECT_MAP.md found.".to_string())
                }
            }
            "get_plan" => {
                let plan_path = self.engine.storage.memory_dir().join(super::super::memory::storage::PLAN_FILE);
                if plan_path.exists() {
                    Ok(std::fs::read_to_string(&plan_path)?)
                } else {
                    Ok("No plan.md found.".to_string())
                }
            }
            "search_events" => {
                let query = args["query"].as_str().unwrap_or("");
                let limit = args["limit"].as_u64().unwrap_or(10) as usize;
                let results = self.engine.search(query, limit)?;
                if results.is_empty() {
                    return Ok(format!("No events matching query '{}'", query));
                }
                let mut out = format!("Found {} matching events for '{}':\n", results.len(), query);
                for r in results {
                    out.push_str(&format!("- (score {:.1}) {}\n", r.score, r.snippet));
                }
                Ok(out)
            }
            "get_score" => {
                let score = self.engine.get_score()?;
                Ok(score.formatted_report())
            }
            "get_context" => {
                let tokens = args["tokens"].as_u64().unwrap_or(2000) as usize;
                let focus = args["focus"].as_str();
                self.engine.get_context(tokens, focus)
            }
            _ => bail!("Unknown memory tool: {}", self.tool_name),
        }
    }
}
