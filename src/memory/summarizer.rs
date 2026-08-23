use super::events::{AttemptOutcome, EventType, MemoryEvent};
use super::project_map::ProjectMap;
use chrono::Utc;
use std::collections::HashMap;

pub struct MemorySummarizer;

impl MemorySummarizer {
    /// Generate complete summary.md from events and project map
    pub fn generate_summary(events: &[MemoryEvent], project_map: Option<&ProjectMap>) -> String {
        let mut out = String::new();
        let date_str = Utc::now().format("%Y-%m-%d").to_string();

        out.push_str(&format!("# Project Memory Summary\n\n_Last updated: {}_\n\n", date_str));

        out.push_str("## Project purpose\n");
        if let Some(pm) = project_map {
            if !pm.purpose.is_empty() {
                out.push_str(&format!("{}\n\n", pm.purpose));
            } else {
                out.push_str("Autonomous project managed by Pi Agent.\n\n");
            }
        } else {
            out.push_str("Autonomous project managed by Pi Agent.\n\n");
        }

        // Aggregate issues
        let mut issues: HashMap<String, (String, Option<String>, bool, Vec<String>)> = HashMap::new();
        let mut unassigned_fixes = Vec::new();
        let mut decisions = Vec::new();
        let mut superseded_decisions = std::collections::HashSet::new();
        let mut notes = Vec::new();

        // First pass: mark superseded decisions & closed issues
        for evt in events {
            if let Some(sup) = &evt.supersedes {
                superseded_decisions.insert(sup.clone());
            }
            if evt.event_type == EventType::Fix {
                if let Some(id) = &evt.issue_id {
                    if let Some(entry) = issues.get_mut(id) {
                        entry.2 = true;
                    }
                }
            }
        }

        for evt in events {
            match evt.event_type {
                EventType::Issue => {
                    let id = evt.issue_id.clone().unwrap_or_else(|| "0000".to_string());
                    let is_fixed = issues.get(&id).map_or(false, |e| e.2);
                    issues.insert(id, (evt.summary.clone(), evt.location.clone(), is_fixed, Vec::new()));
                }
                EventType::Attempt => {
                    if let Some(id) = &evt.issue_id {
                        if let Some(entry) = issues.get_mut(id) {
                            let outcome_str = match evt.outcome {
                                Some(AttemptOutcome::Worked) => "worked",
                                Some(AttemptOutcome::Failed) => "failed",
                                Some(AttemptOutcome::Partial) => "partial",
                                None => "attempt",
                            };
                            entry.3.push(format!("{}: {}", outcome_str, evt.summary));
                        }
                    }
                }
                EventType::Fix => {
                    if let Some(id) = &evt.issue_id {
                        if let Some(entry) = issues.get_mut(id) {
                            entry.2 = true;
                        }
                    } else {
                        unassigned_fixes.push(evt.summary.clone());
                    }
                }
                EventType::Decision => {
                    if !superseded_decisions.contains(&evt.id) {
                        let loc = evt.location.as_deref().unwrap_or("project");
                        decisions.push(format!("- {} [{}]", evt.summary, loc));
                    }
                }
                EventType::Note => {
                    let loc = evt.location.as_deref().unwrap_or("general");
                    notes.push(format!("- {} [{}]", evt.summary, loc));
                }
                _ => {}
            }
        }

        out.push_str("## Recent issues\n");
        if issues.is_empty() && unassigned_fixes.is_empty() {
            out.push_str("None logged yet.\n\n");
        } else {
            let mut sorted_keys: Vec<_> = issues.keys().collect();
            sorted_keys.sort();
            for key in sorted_keys {
                if let Some((sum, loc, is_done, attempts)) = issues.get(key) {
                    let status = if *is_done { "[DONE]" } else { "[OPEN]" };
                    let loc_str = loc.as_deref().unwrap_or("");
                    out.push_str(&format!("- {} #{} {} [{}]\n", status, key, sum, loc_str));
                    for att in attempts {
                        out.push_str(&format!("  * {}\n", att));
                    }
                }
            }
            for fix in unassigned_fixes {
                out.push_str(&format!("- [DONE] Fix: {}\n", fix));
            }
            out.push('\n');
        }

        out.push_str("## Decisions\n");
        if decisions.is_empty() {
            out.push_str("None logged yet.\n\n");
        } else {
            for dec in decisions {
                out.push_str(&format!("{}\n", dec));
            }
            out.push('\n');
        }

        out.push_str("## Notes & Gotchas\n");
        if notes.is_empty() {
            out.push_str("None logged yet.\n\n");
        } else {
            for note in notes {
                out.push_str(&format!("{}\n", note));
            }
            out.push('\n');
        }

        out
    }

    /// Generate token-budgeted memory context block for AI prompt injection
    pub fn build_prompt_context(
        events: &[MemoryEvent],
        project_map: Option<&ProjectMap>,
        target_tokens: usize,
        focus: Option<&str>,
    ) -> String {
        let mut block = String::new();
        block.push_str("<project_memory>\n");

        if let Some(pm) = project_map {
            block.push_str(&format!("Project Purpose: {}\n", pm.purpose));
            if !pm.stack.is_empty() {
                block.push_str(&format!("Stack: {}\n", pm.stack.join(", ")));
            }
        }

        // Get key decisions and recent issues
        let summary = Self::generate_summary(events, project_map);
        // Estimate 4 chars per token
        let max_chars = target_tokens * 4;
        if summary.len() > max_chars {
            let truncated = &summary[..max_chars];
            block.push_str(truncated);
            block.push_str("\n... [Memory summary truncated to fit token budget]\n");
        } else {
            block.push_str(&summary);
        }

        if let Some(f) = focus {
            block.push_str(&format!("\nFocus area: {}\n", f));
        }

        block.push_str("</project_memory>\n");
        block
    }
}
