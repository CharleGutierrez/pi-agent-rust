use super::super::providers::types::ToolDefinition;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Tool name identifier (e.g. "read", "edit", "bash")
    fn name(&self) -> &str;

    /// Detailed description for LLM system prompt & tool declaration
    fn description(&self) -> &str;

    /// JSON schema for parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool given parsed JSON arguments
    async fn execute(&self, args: Value) -> Result<String>;

    /// Convert to ToolDefinition for provider API requests
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
        }
    }
}
