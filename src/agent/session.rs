use crate::providers::types::Message;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model: String,
    pub message_count: usize,
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions_dir: PathBuf,
    current_session_id: String,
    messages: Vec<Message>,
}

impl SessionManager {
    pub fn new(project_root: &Path) -> Result<Self> {
        let sessions_dir = project_root.join(".pi/sessions");
        fs::create_dir_all(&sessions_dir)?;

        let session_id = format!("ses_{}", Uuid::new_v4().to_string().replace('-', "")[..12].to_string());
        Ok(Self {
            sessions_dir,
            current_session_id: session_id,
            messages: Vec::new(),
        })
    }

    pub fn session_id(&self) -> &str {
        &self.current_session_id
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn save_current_session(&self, model: &str) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.jsonl", self.current_session_id));
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&session_file)?;

        for msg in &self.messages {
            let line = serde_json::to_string(msg)?;
            writeln!(file, "{}", line)?;
        }

        // Write metadata
        let first_user_query = self
            .messages
            .iter()
            .find(|m| m.role == crate::providers::types::Role::User)
            .map(|m| m.content.chars().take(50).collect::<String>())
            .unwrap_or_else(|| "New Session".to_string());

        let meta = SessionMetadata {
            id: self.current_session_id.clone(),
            title: first_user_query,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            model: model.to_string(),
            message_count: self.messages.len(),
        };

        let meta_file = self.sessions_dir.join(format!("{}.meta.json", self.current_session_id));
        fs::write(&meta_file, serde_json::to_string_pretty(&meta)?)?;

        Ok(())
    }

    pub fn list_saved_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut list = Vec::new();
        if !self.sessions_dir.exists() {
            return Ok(list);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.to_string_lossy().ends_with(".meta.json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(meta) = serde_json::from_str::<SessionMetadata>(&content) {
                        list.push(meta);
                    }
                }
            }
        }

        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }

    pub fn rollback_last_turn(&mut self) -> bool {
        if self.messages.len() >= 2 {
            self.messages.pop(); // Pop assistant response
            self.messages.pop(); // Pop user turn or tool call
            true
        } else {
            false
        }
    }
}
