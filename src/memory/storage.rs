use super::events::{EventType, MemoryEvent};
use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Directory and file names for AI coding memory
pub const MEMORY_DIR: &str = ".pi/memory";
pub const LEGACY_DIR: &str = ".projectmem";
pub const EVENTS_FILE: &str = "events.jsonl";
pub const SUMMARY_FILE: &str = "summary.md";
pub const MAP_FILE: &str = "PROJECT_MAP.md";
pub const PLAN_FILE: &str = "plan.md";

#[derive(Debug, Clone)]
pub struct MemoryStorage {
    root_dir: PathBuf,
    memory_dir: PathBuf,
    events_path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl MemoryStorage {
    /// Initialize or discover memory directory in the project root or ancestors
    pub fn discover_or_create(start_dir: &Path) -> Result<Self> {
        let mut curr = start_dir.to_path_buf();
        let mut found_dir = None;

        loop {
            let pi_mem = curr.join(MEMORY_DIR);
            let legacy_mem = curr.join(LEGACY_DIR);

            if pi_mem.exists() {
                found_dir = Some((curr.clone(), pi_mem));
                break;
            } else if legacy_mem.exists() {
                found_dir = Some((curr.clone(), legacy_mem));
                break;
            }

            if !curr.pop() {
                break;
            }
        }

        let (root_dir, memory_dir) = match found_dir {
            Some(pair) => pair,
            None => {
                let root = start_dir.to_path_buf();
                let mem = root.join(MEMORY_DIR);
                fs::create_dir_all(&mem)
                    .with_context(|| format!("Failed to create memory directory at {:?}", mem))?;
                (root, mem)
            }
        };

        let events_path = memory_dir.join(EVENTS_FILE);
        if !events_path.exists() {
            File::create(&events_path)?;
        }

        Ok(Self {
            root_dir,
            memory_dir,
            events_path,
            lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn memory_dir(&self) -> &Path {
        &self.memory_dir
    }

    pub fn events_path(&self) -> &Path {
        &self.events_path
    }

    /// Append an event to events.jsonl thread-safely
    pub fn append_event(&self, event: &MemoryEvent) -> Result<()> {
        let _guard = self.lock.lock().unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)
            .with_context(|| format!("Failed to open events file {:?}", self.events_path))?;

        let json_line = serde_json::to_string(event)?;
        writeln!(file, "{}", json_line)?;
        file.flush()?;
        Ok(())
    }

    /// Load all events sequentially
    pub fn load_events(&self) -> Result<Vec<MemoryEvent>> {
        let _guard = self.lock.lock().unwrap();
        if !self.events_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.events_path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for (idx, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<MemoryEvent>(trimmed) {
                Ok(evt) => events.push(evt),
                Err(e) => {
                    tracing::warn!("Corrupted memory event on line {}: {}", idx + 1, e);
                }
            }
        }

        Ok(events)
    }

    /// Get next sequential issue ID (e.g. "0001", "0002")
    pub fn next_issue_id(&self) -> Result<String> {
        let events = self.load_events()?;
        let mut max_id = 0u32;

        for evt in events {
            if evt.event_type == EventType::Issue {
                if let Some(id_str) = evt.issue_id.as_deref() {
                    if let Ok(n) = id_str.trim_start_matches('0').parse::<u32>() {
                        if n > max_id {
                            max_id = n;
                        }
                    }
                }
            }
        }

        Ok(format!("{:04}", max_id + 1))
    }

    /// Find the latest active (unresolved) issue ID
    pub fn latest_open_issue_id(&self) -> Result<Option<String>> {
        let events = self.load_events()?;
        let mut open_issues = std::collections::HashSet::new();

        for evt in &events {
            match evt.event_type {
                EventType::Issue => {
                    if let Some(id) = &evt.issue_id {
                        open_issues.insert(id.clone());
                    }
                }
                EventType::Fix => {
                    if let Some(id) = &evt.issue_id {
                        open_issues.remove(id);
                    }
                }
                _ => {}
            }
        }

        let mut sorted: Vec<String> = open_issues.into_iter().collect();
        sorted.sort();
        Ok(sorted.last().cloned())
    }
}
