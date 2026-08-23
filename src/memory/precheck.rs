use super::events::{AttemptOutcome, EventType, MemoryEvent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecheckReport {
    pub file_path: String,
    pub has_warnings: bool,
    pub open_issues: Vec<String>,
    pub failed_attempts: Vec<String>,
    pub decisions: Vec<String>,
    pub churn_count: usize,
    pub guidance: String,
}

pub struct Prechecker;

impl Prechecker {
    pub fn check_file(events: &[MemoryEvent], target_path: &str) -> PrecheckReport {
        let clean_target = target_path.replace('\\', "/").trim_start_matches("./").to_string();

        let mut open_issues = Vec::new();
        let mut resolved_issues = std::collections::HashSet::new();
        let mut failed_attempts = Vec::new();
        let mut decisions = Vec::new();
        let mut churn_count = 0;

        // First pass: find resolved issues
        for evt in events {
            if evt.event_type == EventType::Fix {
                if let Some(id) = &evt.issue_id {
                    resolved_issues.insert(id.clone());
                }
            }
        }

        // Second pass: match events with this target file
        for evt in events {
            let matches_location = evt.location.as_ref().map_or(false, |loc| {
                let clean_loc = loc.replace('\\', "/").trim_start_matches("./").to_string();
                clean_loc == clean_target
                    || clean_target.starts_with(&clean_loc)
                    || clean_loc.starts_with(&clean_target)
            });

            if matches_location {
                churn_count += 1;
                match evt.event_type {
                    EventType::Issue => {
                        let is_resolved = evt.issue_id.as_ref().map_or(false, |id| resolved_issues.contains(id));
                        if !is_resolved {
                            open_issues.push(format!(
                                "[Issue #{}] {}",
                                evt.issue_id.as_deref().unwrap_or("?"),
                                evt.summary
                            ));
                        }
                    }
                    EventType::Attempt => {
                        if evt.outcome == Some(AttemptOutcome::Failed) {
                            failed_attempts.push(evt.summary.clone());
                        }
                    }
                    EventType::Decision => {
                        decisions.push(evt.summary.clone());
                    }
                    _ => {}
                }
            }
        }

        let has_warnings = !open_issues.is_empty() || !failed_attempts.is_empty() || churn_count > 4;

        let guidance = if !has_warnings {
            format!("No failure warnings for `{}`. Clear to proceed.", clean_target)
        } else {
            let mut g = format!("⚠️ WARNINGS for `{}`:\n", clean_target);
            if !open_issues.is_empty() {
                g.push_str("  Open Issues:\n");
                for issue in &open_issues {
                    g.push_str(&format!("   - {}\n", issue));
                }
            }
            if !failed_attempts.is_empty() {
                g.push_str("  Failed Approaches (DO NOT REPEAT):\n");
                for fail in &failed_attempts {
                    g.push_str(&format!("   - ❌ {}\n", fail));
                }
            }
            if !decisions.is_empty() {
                g.push_str("  Applicable Decisions:\n");
                for dec in &decisions {
                    g.push_str(&format!("   - 💡 {}\n", dec));
                }
            }
            if churn_count > 4 {
                g.push_str(&format!("  High Churn: touched {} times recently.\n", churn_count));
            }
            g
        };

        PrecheckReport {
            file_path: clean_target,
            has_warnings,
            open_issues,
            failed_attempts,
            decisions,
            churn_count,
            guidance,
        }
    }
}
