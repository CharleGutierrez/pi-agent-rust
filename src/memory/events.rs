use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Typed Event types for Persistent AI Coding Memory
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Discovered bug, regression, or unexpected behavior
    Issue,
    /// Fix attempt with outcome (worked, failed, partial)
    Attempt,
    /// Verified fix that resolves an issue
    Fix,
    /// Permanent architectural or product decision
    Decision,
    /// Gotcha, constraint, environment note, setup detail
    Note,
    /// Significant project milestone reached
    Milestone,
    /// Reverted change / rollback
    Revert,
    /// Context checkpoint / snapshot
    ContextSnapshot,
}

/// Attempt outcome
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Worked,
    Failed,
    Partial,
}

impl std::fmt::Display for AttemptOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttemptOutcome::Worked => write!(f, "worked"),
            AttemptOutcome::Failed => write!(f, "failed"),
            AttemptOutcome::Partial => write!(f, "partial"),
        }
    }
}

/// A structured persistent memory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// Unique event identifier (evt_uuid)
    pub id: String,
    /// Event type classification
    pub event_type: EventType,
    /// ISO-8601 UTC timestamp
    pub timestamp: DateTime<Utc>,
    /// One-line summary
    pub summary: String,
    /// Detailed notes or hypothesis
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// File path or component location
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Associated issue ID (e.g. 0001)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    /// Attempt outcome if event_type is Attempt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<AttemptOutcome>,
    /// Superseded event id if event_type is Decision
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    /// Cost or token estimate saved/spent
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_prevented: Option<u64>,
    /// Extra metadata
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl MemoryEvent {
    pub fn new(event_type: EventType, summary: impl Into<String>) -> Self {
        Self {
            id: format!("evt_{}", Uuid::new_v4().to_string().replace('-', "")[..12].to_string()),
            event_type,
            timestamp: Utc::now(),
            summary: summary.into(),
            notes: None,
            location: None,
            issue_id: None,
            outcome: None,
            supersedes: None,
            tokens_prevented: None,
            metadata: serde_json::Map::new(),
        }
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_issue_id(mut self, issue_id: impl Into<String>) -> Self {
        self.issue_id = Some(issue_id.into());
        self
    }

    pub fn with_outcome(mut self, outcome: AttemptOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    pub fn with_supersedes(mut self, old_id: impl Into<String>) -> Self {
        self.supersedes = Some(old_id.into());
        self
    }

    pub fn with_tokens_prevented(mut self, tokens: u64) -> Self {
        self.tokens_prevented = Some(tokens);
        self
    }
}
