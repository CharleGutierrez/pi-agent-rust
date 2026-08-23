use super::traits::AgentTool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use tokio::process::Command;

pub struct GitTool {
    work_dir: PathBuf,
}

impl GitTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }
}

#[async_trait]
impl AgentTool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Inspect git status, diffs, log history, branches, or stage and commit changes."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "subcommand": {
                    "type": "string",
                    "description": "Git subcommand: status, diff, log, branch, add, commit, rollback",
                    "enum": ["status", "diff", "log", "branch", "add", "commit", "rollback"]
                },
                "args": {
                    "type": "string",
                    "description": "Optional additional arguments or commit message (e.g. 'src/main.rs' or 'feat: add ai tuner')"
                }
            },
            "required": ["subcommand"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let sub = match args["subcommand"].as_str() {
            Some(s) => s,
            None => bail!("Missing required parameter: 'subcommand'"),
        };

        let extra_args = args["args"].as_str().unwrap_or("");

        let mut cmd = Command::new("git");
        cmd.current_dir(&self.work_dir);

        match sub {
            "status" => {
                cmd.args(&["status", "--short", "--branch"]);
            }
            "diff" => {
                if extra_args.is_empty() {
                    cmd.arg("diff");
                } else {
                    cmd.arg("diff").arg(extra_args);
                }
            }
            "log" => {
                cmd.args(&["log", "--oneline", "-n", "15"]);
            }
            "branch" => {
                cmd.args(&["branch", "-a"]);
            }
            "add" => {
                let target = if extra_args.is_empty() { "." } else { extra_args };
                cmd.args(&["add", target]);
            }
            "commit" => {
                if extra_args.is_empty() {
                    bail!("Commit message is required in 'args'");
                }
                cmd.args(&["commit", "-m", extra_args]);
            }
            "rollback" => {
                let target = if extra_args.is_empty() { "." } else { extra_args };
                cmd.args(&["checkout", "--", target]);
            }
            _ => bail!("Unsupported git subcommand: {}", sub),
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut res = String::new();
        if !stdout.is_empty() {
            res.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !res.is_empty() {
                res.push('\n');
            }
            res.push_str(&stderr);
        }
        if res.is_empty() {
            res.push_str("(Git command completed successfully)");
        }

        Ok(res)
    }
}
