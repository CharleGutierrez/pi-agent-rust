use super::traits::AgentTool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct ReadTool {
    work_dir: PathBuf,
}

impl ReadTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }
}

#[async_trait]
impl AgentTool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file contents with line numbering, offset, and limit. Truncates large outputs safely."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read (relative or absolute)"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-indexed, default 1)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read (default 2000)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path_str = args["path"].as_str().unwrap_or("");
        if path_str.is_empty() {
            bail!("Missing required parameter: 'path'");
        }

        let offset = args["offset"].as_u64().unwrap_or(1) as usize;
        let limit = args["limit"].as_u64().unwrap_or(2000) as usize;

        let full_path = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            self.work_dir.join(path_str)
        };

        if !full_path.exists() {
            bail!("File not found: {:?}", full_path);
        }

        if full_path.is_dir() {
            bail!("Path is a directory, not a file: {:?}", full_path);
        }

        let file = File::open(&full_path)?;
        let reader = BufReader::new(file);

        let mut lines_out = Vec::new();
        let mut total_lines = 0;
        let mut total_bytes = 0;

        for (idx, line_res) in reader.lines().enumerate() {
            let line_num = idx + 1;
            total_lines += 1;
            let line = line_res?;

            if line_num >= offset && line_num < offset + limit {
                total_bytes += line.len() + 1;
                // Cap output at 64KB for token safety
                if total_bytes > 64 * 1024 {
                    lines_out.push(format!("... [Output truncated at 64KB, file has {} total lines]", total_lines));
                    break;
                }
                lines_out.push(format!("{:4} | {}", line_num, line));
            }
        }

        if lines_out.is_empty() {
            return Ok(format!("File {:?} is empty or offset {} is beyond end of file (total lines: {}).", full_path, offset, total_lines));
        }

        Ok(lines_out.join("\n"))
    }
}
