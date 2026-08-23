use super::traits::AgentTool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub struct BashTool {
    work_dir: PathBuf,
}

impl BashTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }
}

#[async_trait]
impl AgentTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a shell/bash command in the project directory. Captures stdout and stderr, supports timeout, truncates large outputs."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional timeout in seconds (default 60)"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let cmd_str = match args["command"].as_str() {
            Some(c) => c,
            None => bail!("Missing required parameter: 'command'"),
        };

        let timeout_secs = args["timeout_secs"].as_u64().unwrap_or(60);

        #[cfg(target_os = "windows")]
        let child = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(cmd_str)
            .current_dir(&self.work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        #[cfg(not(target_os = "windows"))]
        let child = Command::new("bash")
            .arg("-c")
            .arg(cmd_str)
            .current_dir(&self.work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let result = timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut out = String::new();
                if exit_code != 0 {
                    out.push_str(&format!("[Process exited with status code {}]\n", exit_code));
                }

                if !stdout.is_empty() {
                    out.push_str(&stdout);
                }

                if !stderr.is_empty() {
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("[stderr]\n");
                    out.push_str(&stderr);
                }

                if out.is_empty() {
                    out.push_str("(Command finished with no output)");
                }

                // Truncate if output exceeds 60KB
                if out.len() > 60 * 1024 {
                    let truncated = &out[..60 * 1024];
                    return Ok(format!("{}\n... [Output truncated after 60KB]", truncated));
                }

                Ok(out)
            }
            Ok(Err(e)) => bail!("Command execution failed: {}", e),
            Err(_) => bail!("Command timed out after {} seconds", timeout_secs),
        }
    }
}
