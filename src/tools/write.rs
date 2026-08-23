use super::traits::AgentTool;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub struct WriteTool {
    work_dir: PathBuf,
}

impl WriteTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }

    fn backup_file(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            return Ok(()); // Nothing to backup
        }
        let backup_dir = self.work_dir.join(".pi/backups");
        fs::create_dir_all(&backup_dir)?;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
        let backup_path = backup_dir.join(format!("{}_{}.bak", timestamp, file_name));
        
        fs::copy(file_path, &backup_path)?;
        tracing::info!("Created backup of {:?} at {:?}", file_path, backup_path);
        Ok(())
    }
}

#[async_trait]
impl AgentTool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write full content to a file. Creates the file if it doesn't exist, overwrites if it does. Automatically creates parent directories."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write (relative or absolute)"
                },
                "content": {
                    "type": "string",
                    "description": "Full content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path_str = args["path"].as_str().unwrap_or("");
        if path_str.is_empty() {
            bail!("Missing required parameter: 'path'");
        }

        let content = match args["content"].as_str() {
            Some(c) => c,
            None => bail!("Missing required parameter: 'content'"),
        };

        let full_path = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            self.work_dir.join(path_str)
        };

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for {:?}", full_path))?;
        }

        // Create safety backup if file already exists
        let _ = self.backup_file(&full_path);

        let bytes = content.as_bytes().len();
        fs::write(&full_path, content)
            .with_context(|| format!("Failed to write to {:?}", full_path))?;

        Ok(format!("Successfully wrote {} bytes to {:?}", bytes, path_str))
    }
}
