use super::traits::AgentTool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use ignore::{WalkBuilder, WalkState};
use regex::RegexBuilder;
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct GrepTool {
    work_dir: PathBuf,
}

impl GrepTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }
}

#[async_trait]
impl AgentTool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Fast ripgrep-style regex/plain-text search across codebase respecting .gitignore. Returns matching file paths, line numbers, and lines."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex or plain-text pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file path to search inside (default: current directory)"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Case-sensitive search (default false)"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum matching lines to return (default 100)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let pattern = match args["pattern"].as_str() {
            Some(p) => p,
            None => bail!("Missing required parameter: 'pattern'"),
        };

        let sub_path = args["path"].as_str().unwrap_or(".");
        let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
        let max_results = args["max_results"].as_u64().unwrap_or(100) as usize;

        let search_dir = self.work_dir.join(sub_path);
        if !search_dir.exists() {
            bail!("Search path does not exist: {:?}", search_dir);
        }

        let regex = RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .build()?;

        let walker = WalkBuilder::new(&search_dir)
            .hidden(false)
            .git_ignore(true)
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != ".git" && name != "target" && name != "node_modules"
            })
            .build_parallel();

        let matches = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let matches_clone = matches.clone();
        let work_dir = self.work_dir.clone();
        
        let _ = tokio::task::spawn_blocking(move || {
            walker.run(|| {
                let matches = matches_clone.clone();
                let regex = regex.clone();
                let work_dir = work_dir.clone();
                
                Box::new(move |result| {
                    // Stop if we hit the limit
                    if matches.lock().unwrap().len() >= max_results {
                        return WalkState::Quit;
                    }

                    if let Ok(entry) = result {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(file) = File::open(path) {
                                let reader = BufReader::new(file);
                                for (line_num, line_res) in reader.lines().enumerate() {
                                    if let Ok(line) = line_res {
                                        if regex.is_match(&line) {
                                            let rel_path = path
                                                .strip_prefix(&work_dir)
                                                .unwrap_or(path)
                                                .to_string_lossy()
                                                .replace('\\', "/");

                                            let mut m = matches.lock().unwrap();
                                            if m.len() < max_results {
                                                m.push(format!("{}:{}: {}", rel_path, line_num + 1, line.trim_end()));
                                            } else {
                                                return WalkState::Quit;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    WalkState::Continue
                })
            });
        }).await;

        let results = matches.lock().unwrap().clone();

        if results.is_empty() {
            return Ok(format!("No matches found for pattern '{}' in {:?}", pattern, sub_path));
        }

        Ok(results.join("\n"))
    }
}
