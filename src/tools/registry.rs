use super::bash::BashTool;
use super::edit::EditTool;
use super::find_files::FindFilesTool;
use super::git::GitTool;
use super::grep::GrepTool;
use super::memory_tools::MemoryToolWrapper;
use super::read::ReadTool;
use super::traits::AgentTool;
use super::web_fetch::WebFetchTool;
use super::write::WriteTool;
use crate::memory::MemoryEngine;
use crate::providers::types::{ToolCall, ToolDefinition, ToolResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn AgentTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn AgentTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Initialize default full tool suite for workspace
    pub fn init_standard(work_dir: PathBuf, memory_engine: Arc<MemoryEngine>) -> Self {
        let mut reg = Self::new();

        // 1. Filesystem & Terminal Tools
        reg.register(Box::new(ReadTool::new(work_dir.clone())));
        reg.register(Box::new(WriteTool::new(work_dir.clone())));
        reg.register(Box::new(EditTool::new(work_dir.clone())));
        reg.register(Box::new(BashTool::new(work_dir.clone())));
        reg.register(Box::new(GrepTool::new(work_dir.clone())));
        reg.register(Box::new(FindFilesTool::new(work_dir.clone())));
        reg.register(Box::new(GitTool::new(work_dir.clone())));
        reg.register(Box::new(WebFetchTool::new()));

        // 2. Persistent Memory MCP Tools
        for mem_tool in MemoryToolWrapper::create_all(memory_engine) {
            reg.register(mem_tool);
        }

        reg
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<_> = self.tools.values().map(|t| t.to_definition()).collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }

    pub async fn execute_call(&self, call: &ToolCall) -> ToolResult {
        let tool_name = &call.name;
        match self.tools.get(tool_name) {
            Some(tool) => match tool.execute(call.arguments.clone()).await {
                Ok(content) => ToolResult {
                    tool_call_id: call.id.clone(),
                    content,
                    is_error: false,
                },
                Err(err) => ToolResult {
                    tool_call_id: call.id.clone(),
                    content: format!("Error executing tool '{}': {}", tool_name, err),
                    is_error: true,
                },
            },
            None => ToolResult {
                tool_call_id: call.id.clone(),
                content: format!("Unknown tool '{}'", tool_name),
                is_error: true,
            },
        }
    }
}
