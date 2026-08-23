use super::traits::AgentTool;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub struct EditTool {
    work_dir: PathBuf,
}

impl EditTool {
    pub fn new(work_dir: PathBuf) -> Self {
        Self { work_dir }
    }

    fn backup_file(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            return Ok(());
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
impl AgentTool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edit a file using exact or smart-whitespace text replacement. Supports single or multiple non-overlapping replacements in edits[]."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "edits": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "oldText": {
                                "type": "string",
                                "description": "Exact text to replace (incorporates smart whitespace matching if indentation is off)"
                            },
                            "newText": {
                                "type": "string",
                                "description": "Replacement text"
                            }
                        },
                        "required": ["oldText", "newText"]
                    }
                }
            },
            "required": ["path", "edits"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let path_str = args["path"].as_str().unwrap_or("");
        if path_str.is_empty() {
            bail!("Missing required parameter: 'path'");
        }

        let edits_array = match args["edits"].as_array() {
            Some(arr) => arr,
            None => bail!("Missing or invalid 'edits' array"),
        };

        let full_path = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            self.work_dir.join(path_str)
        };

        if !full_path.exists() {
            bail!("File not found: {:?}", full_path);
        }

        let mut content = fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file {:?}", full_path))?;

        let mut applied = 0;
        for (i, edit) in edits_array.iter().enumerate() {
            let old_text = edit["oldText"].as_str().unwrap_or("");
            let new_text = edit["newText"].as_str().unwrap_or("");

            // 1. Try exact match
            let count = content.matches(old_text).count();
            if count == 1 {
                content = content.replacen(old_text, new_text, 1);
                applied += 1;
                continue;
            }

            // 2. Try normalized line endings
            let normalized_content = content.replace("\r\n", "\n");
            let normalized_old = old_text.replace("\r\n", "\n");
            
            if normalized_content.matches(&normalized_old).count() == 1 {
                let normalized_new = new_text.replace("\r\n", "\n");
                content = normalized_content.replacen(&normalized_old, &normalized_new, 1);
                applied += 1;
                continue;
            }

            // 3. Try Smart Whitespace matching (Fuzzy Indentation)
            // Leading whitespaces on each line might be slightly wrong from LLM
            let fuzzy_old: String = normalized_old
                .lines()
                .map(|l| l.trim_start())
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            if fuzzy_old.is_empty() {
                bail!("Edit #{}: oldText is empty or invalid.", i + 1);
            }

            // Search content line by line to find a block that matches when trimmed
            let content_lines: Vec<&str> = normalized_content.lines().collect();
            let old_lines: Vec<&str> = fuzzy_old.lines().collect();
            
            let mut match_idx = None;
            let mut matches_found = 0;

            for start_idx in 0..=content_lines.len().saturating_sub(old_lines.len()) {
                let mut matches = true;
                for offset in 0..old_lines.len() {
                    if content_lines[start_idx + offset].trim_start() != old_lines[offset] {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    matches_found += 1;
                    match_idx = Some(start_idx);
                }
            }

            if matches_found == 1 {
                if let Some(start_idx) = match_idx {
                    let end_idx = start_idx + old_lines.len();
                    let before = content_lines[..start_idx].join("\n");
                    let after = content_lines[end_idx..].join("\n");
                    
                    // Reassemble
                    let mut new_content = before;
                    if !new_content.is_empty() { new_content.push('\n'); }
                    new_content.push_str(new_text);
                    if !after.is_empty() { new_content.push('\n'); new_content.push_str(&after); }
                    
                    content = new_content;
                    applied += 1;
                    continue;
                }
            } else if matches_found > 1 {
                bail!(
                    "Edit #{}: oldText matched {} times (fuzzily) in {:?}. Please provide more unique surrounding lines.",
                    i + 1, matches_found, path_str
                );
            }

            bail!(
                "Edit #{}: oldText not found in {:?} (even with smart whitespace matching).\nOld text was:\n{}",
                i + 1, path_str, old_text
            );
        }

        // Create a safety backup before overwriting
        let _ = self.backup_file(&full_path);

        fs::write(&full_path, content)
            .with_context(|| format!("Failed to write edited file {:?}", full_path))?;

        Ok(format!("Successfully applied {} smart edit(s) to {:?}", applied, path_str))
    }
}
