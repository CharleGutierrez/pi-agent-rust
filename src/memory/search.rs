use super::events::MemoryEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub event: MemoryEvent,
    pub score: f32,
    pub snippet: String,
}

pub struct MemorySearchEngine;

impl MemorySearchEngine {
    /// Search memory events using weighted token matching & BM25-like scoring
    pub fn search(events: &[MemoryEvent], query: &str, limit: usize) -> Vec<SearchResult> {
        let query_terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if query_terms.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for event in events {
            let mut score = 0.0f32;
            let summary_lower = event.summary.to_lowercase();
            let notes_lower = event.notes.as_deref().unwrap_or("").to_lowercase();
            let location_lower = event.location.as_deref().unwrap_or("").to_lowercase();
            let issue_id_lower = event.issue_id.as_deref().unwrap_or("").to_lowercase();

            for term in &query_terms {
                // Exact matches in summary
                if summary_lower.contains(term) {
                    score += 5.0;
                }
                // Location matches
                if location_lower.contains(term) {
                    score += 4.0;
                }
                // Issue ID exact match
                if issue_id_lower == *term {
                    score += 10.0;
                }
                // Notes matches
                if notes_lower.contains(term) {
                    score += 2.0;
                }
            }

            if score > 0.0 {
                let snippet = format!(
                    "[{:?}] {} (loc: {})",
                    event.event_type,
                    event.summary,
                    event.location.as_deref().unwrap_or("global")
                );

                results.push(SearchResult {
                    event: event.clone(),
                    score,
                    snippet,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }
}
