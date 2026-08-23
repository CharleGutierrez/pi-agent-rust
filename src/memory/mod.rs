pub mod events;
pub mod intent_plan;
pub mod precheck;
pub mod project_map;
pub mod scoring;
pub mod search;
pub mod storage;
pub mod summarizer;

pub use events::{AttemptOutcome, EventType, MemoryEvent};
pub use intent_plan::IntentPlan;
pub use precheck::{PrecheckReport, Prechecker};
pub use project_map::ProjectMap;
pub use scoring::FailurePreventionScore;
pub use search::{MemorySearchEngine, SearchResult};
pub use storage::MemoryStorage;
pub use summarizer::MemorySummarizer;

use anyhow::Result;
use std::fs;
use std::path::Path;

/// High-level AI Coding Memory Engine
#[derive(Debug, Clone)]
pub struct MemoryEngine {
    pub storage: MemoryStorage,
}

impl MemoryEngine {
    pub fn new(root_dir: &Path) -> Result<Self> {
        let storage = MemoryStorage::discover_or_create(root_dir)?;
        let engine = Self { storage };
        // Ensure summary and map exist
        engine.auto_sync()?;
        Ok(engine)
    }

    /// Automatically sync summary.md and project_map.md
    pub fn auto_sync(&self) -> Result<()> {
        let events = self.storage.load_events()?;
        let map_path = self.storage.memory_dir().join(storage::MAP_FILE);
        
        let map = if map_path.exists() {
            ProjectMap::load_from_file(&map_path)?
        } else {
            let auto_map = ProjectMap::auto_scan(self.storage.root_dir());
            auto_map.save_to_file(&map_path)?;
            Some(auto_map)
        };

        // Regenerate summary.md
        let summary_content = MemorySummarizer::generate_summary(&events, map.as_ref());
        let summary_path = self.storage.memory_dir().join(storage::SUMMARY_FILE);
        fs::write(&summary_path, summary_content)?;

        // Ensure plan.md exists
        let plan_path = self.storage.memory_dir().join(storage::PLAN_FILE);
        if !plan_path.exists() {
            let initial_plan = IntentPlan::new();
            initial_plan.save_to_file(&plan_path)?;
        }

        Ok(())
    }

    pub fn log_issue(&self, summary: &str, location: Option<&str>) -> Result<String> {
        let issue_id = self.storage.next_issue_id()?;
        let mut evt = MemoryEvent::new(EventType::Issue, summary).with_issue_id(&issue_id);
        if let Some(loc) = location {
            evt = evt.with_location(loc);
        }
        self.storage.append_event(&evt)?;
        self.auto_sync()?;
        Ok(issue_id)
    }

    pub fn record_attempt(
        &self,
        summary: &str,
        outcome: AttemptOutcome,
        issue_id: Option<&str>,
        location: Option<&str>,
    ) -> Result<String> {
        let resolved_issue_id = match issue_id {
            Some(id) => id.to_string(),
            None => self
                .storage
                .latest_open_issue_id()?
                .unwrap_or_else(|| "0001".to_string()),
        };

        let mut evt = MemoryEvent::new(EventType::Attempt, summary)
            .with_issue_id(&resolved_issue_id)
            .with_outcome(outcome);

        if let Some(loc) = location {
            evt = evt.with_location(loc);
        }

        self.storage.append_event(&evt)?;
        self.auto_sync()?;
        Ok(evt.id)
    }

    pub fn record_fix(&self, summary: &str, issue_id: Option<&str>, location: Option<&str>) -> Result<String> {
        let resolved_issue_id = match issue_id {
            Some(id) => Some(id.to_string()),
            None => self.storage.latest_open_issue_id()?,
        };

        let mut evt = MemoryEvent::new(EventType::Fix, summary);
        if let Some(id) = resolved_issue_id {
            evt = evt.with_issue_id(id);
        }
        if let Some(loc) = location {
            evt = evt.with_location(loc);
        }

        self.storage.append_event(&evt)?;
        self.auto_sync()?;
        Ok(evt.id)
    }

    pub fn add_decision(&self, summary: &str, location: Option<&str>, supersedes: Option<&str>) -> Result<String> {
        let mut evt = MemoryEvent::new(EventType::Decision, summary);
        if let Some(loc) = location {
            evt = evt.with_location(loc);
        }
        if let Some(sup) = supersedes {
            evt = evt.with_supersedes(sup);
        }

        self.storage.append_event(&evt)?;
        self.auto_sync()?;
        Ok(evt.id)
    }

    pub fn add_note(&self, summary: &str, location: Option<&str>) -> Result<String> {
        let mut evt = MemoryEvent::new(EventType::Note, summary);
        if let Some(loc) = location {
            evt = evt.with_location(loc);
        }

        self.storage.append_event(&evt)?;
        self.auto_sync()?;
        Ok(evt.id)
    }

    pub fn precheck_file(&self, target_path: &str) -> Result<PrecheckReport> {
        let events = self.storage.load_events()?;
        Ok(Prechecker::check_file(&events, target_path))
    }

    pub fn get_summary(&self) -> Result<String> {
        let events = self.storage.load_events()?;
        let map_path = self.storage.memory_dir().join(storage::MAP_FILE);
        let map = ProjectMap::load_from_file(&map_path)?;
        Ok(MemorySummarizer::generate_summary(&events, map.as_ref()))
    }

    pub fn get_score(&self) -> Result<FailurePreventionScore> {
        let events = self.storage.load_events()?;
        Ok(FailurePreventionScore::compute(&events))
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let events = self.storage.load_events()?;
        Ok(MemorySearchEngine::search(&events, query, limit))
    }

    pub fn get_context(&self, target_tokens: usize, focus: Option<&str>) -> Result<String> {
        let events = self.storage.load_events()?;
        let map_path = self.storage.memory_dir().join(storage::MAP_FILE);
        let map = ProjectMap::load_from_file(&map_path)?;
        Ok(MemorySummarizer::build_prompt_context(
            &events,
            map.as_ref(),
            target_tokens,
            focus,
        ))
    }
}
