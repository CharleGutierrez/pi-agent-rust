use super::traits::AgentTool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use ignore::{WalkBuilder, WalkState};
use regex::RegexBuilder;
use serde_json::json;
use std::path::PathBuf;

pub struct FindFilesTool {
    work_dir: PathBuf,
}

impl FindFilesTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }
}

#[async_trait]
impl AgentTool for FindFilesTool {
    fn name(&self) -> &str {
        "find_files"
    }

    fn description(&self) -> &str {
        "Find files matching a glob or regex pattern across the project while respecting .gitignore."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "File name pattern or regex to search for (e.g. '*.rs', 'auth', 'Cargo')"
                },
                "path": {
                    "type": "string",
                    "description": "Starting directory (default: current directory)"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of files to return (default 100)"
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
        let max_results = args["max_results"].as_u64().unwrap_or(100) as usize;

        let start_dir = self.work_dir.join(sub_path);
        if !start_dir.exists() {
            bail!("Path does not exist: {:?}", start_dir);
        }

        let regex_pattern = if pattern.contains('*') || pattern.contains('?') {
            pattern.replace('.', "\\.").replace('*', ".*").replace('?', ".")
        } else {
            pattern.to_string()
        };

        let regex = RegexBuilder::new(&regex_pattern)
            .case_insensitive(true)
            .build()?;

        let walker = WalkBuilder::new(&start_dir)
            .hidden(false)
            .git_ignore(true)
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != ".git" && name != "target" && name != "node_modules"
            })
            .build_parallel();

        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let results_clone = results.clone();
        let work_dir = self.work_dir.clone();

        let _ = tokio::task::spawn_blocking(move || {
            walker.run(|| {
                let results = results_clone.clone();
                let regex = regex.clone();
                let work_dir = work_dir.clone();

                Box::new(move |result| {
                    if results.lock().unwrap().len() >= max_results {
                        return WalkState::Quit;
                    }

                    if let Ok(entry) = result {
                        let path = entry.path();
                        let file_name = entry.file_name().to_string_lossy();

                        if regex.is_match(&file_name) || regex.is_match(&path.to_string_lossy()) {
                            let rel = path
                                .strip_prefix(&work_dir)
                                .unwrap_or(path)
                                .to_string_lossy()
                                .replace('\\', "/");

                            let is_dir = path.is_dir();
                            let display = if is_dir { format!("{}/", rel) } else { rel };
                            
                            let mut r = results.lock().unwrap();
                            if r.len() < max_results {
                                r.push(display);
                            } else {
                                return WalkState::Quit;
                            }
                        }
                    }
                    WalkState::Continue
                })
            });
        }).await;

        let final_results = results.lock().unwrap().clone();

        if final_results.is_empty() {
            return Ok(format!("No files matching pattern '{}' found in {:?}", pattern, sub_path));
        }

        Ok(final_results.join("\n"))
    }
}
