use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Item in plan.md
#[derive(Debug, Clone)]
pub struct PlanItem {
    pub text: String,
    pub completed: bool,
    pub section: String,
}

/// Project Intent / Plan Representation
#[derive(Debug, Clone)]
pub struct IntentPlan {
    pub ideas: Vec<String>,
    pub active_plans: Vec<PlanItem>,
    pub next_plans: Vec<PlanItem>,
    pub shipped: Vec<String>,
}

impl IntentPlan {
    pub fn new() -> Self {
        Self {
            ideas: Vec::new(),
            active_plans: Vec::new(),
            next_plans: Vec::new(),
            shipped: Vec::new(),
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        Ok(Some(Self::parse(&content)))
    }

    pub fn parse(content: &str) -> Self {
        let mut plan = Self::new();
        let mut current_section = "";

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("## Ideas") {
                current_section = "ideas";
                continue;
            } else if trimmed.starts_with("## Active plans") {
                current_section = "active";
                continue;
            } else if trimmed.starts_with("## Next") {
                current_section = "next";
                continue;
            } else if trimmed.starts_with("## Shipped") {
                current_section = "shipped";
                continue;
            } else if trimmed.starts_with("## ") {
                current_section = "";
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with('>') || trimmed.starts_with('_') {
                continue;
            }

            match current_section {
                "ideas" => {
                    let idea = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
                    if !idea.is_empty() {
                        plan.ideas.push(idea.to_string());
                    }
                }
                "active" => {
                    let (completed, text) = Self::parse_checkbox(trimmed);
                    if !text.is_empty() {
                        plan.active_plans.push(PlanItem {
                            text,
                            completed,
                            section: "active".to_string(),
                        });
                    }
                }
                "next" => {
                    let (completed, text) = Self::parse_checkbox(trimmed);
                    if !text.is_empty() {
                        plan.next_plans.push(PlanItem {
                            text,
                            completed,
                            section: "next".to_string(),
                        });
                    }
                }
                "shipped" => {
                    let item = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
                    if !item.is_empty() {
                        plan.shipped.push(item.to_string());
                    }
                }
                _ => {}
            }
        }

        plan
    }

    fn parse_checkbox(line: &str) -> (bool, String) {
        let trimmed = line.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
        if trimmed.starts_with("[x]") || trimmed.starts_with("[X]") {
            (true, trimmed[3..].trim().to_string())
        } else if trimmed.starts_with("[ ]") {
            (false, trimmed[3..].trim().to_string())
        } else {
            (false, trimmed.to_string())
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Project Plan & Intent\n\n");
        out.push_str("> Editable intent file: ideas + plans. Separate from event history.\n\n");

        out.push_str("## Ideas\n");
        if self.ideas.is_empty() {
            out.push_str("_Loose thoughts, not yet committed to._\n\n");
        } else {
            for idea in &self.ideas {
                out.push_str(&format!("- {}\n", idea));
            }
            out.push('\n');
        }

        out.push_str("## Active plans\n");
        if self.active_plans.is_empty() {
            out.push_str("_What we're working toward now._\n\n");
        } else {
            for item in &self.active_plans {
                let mark = if item.completed { "[x]" } else { "[ ]" };
                out.push_str(&format!("- {} {}\n", mark, item.text));
            }
            out.push('\n');
        }

        out.push_str("## Next\n");
        if self.next_plans.is_empty() {
            out.push_str("_Queued, but not started._\n\n");
        } else {
            for item in &self.next_plans {
                let mark = if item.completed { "[x]" } else { "[ ]" };
                out.push_str(&format!("- {} {}\n", mark, item.text));
            }
            out.push('\n');
        }

        out.push_str("## Shipped\n");
        if self.shipped.is_empty() {
            out.push_str("_Move completed items here._\n\n");
        } else {
            for s in &self.shipped {
                out.push_str(&format!("- {}\n", s));
            }
            out.push('\n');
        }

        out
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_markdown())
            .with_context(|| format!("Failed to save plan to {:?}", path))?;
        Ok(())
    }
}
