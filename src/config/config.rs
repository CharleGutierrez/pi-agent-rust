use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomModelConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String, // "openai", "anthropic", "gemini", "ollama", etc.
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub context_window: u32,
    pub supports_tools: bool,
    pub supports_thinking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_model_str")]
    pub default_model: String,
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
    #[serde(default = "default_auto_precheck")]
    pub auto_memory_precheck: bool,
    #[serde(default)]
    pub custom_instructions: Option<String>,
    #[serde(default)]
    pub custom_models: Vec<CustomModelConfig>,
}

fn default_model_str() -> String {
    "claude-3-7-sonnet-latest".to_string()
}

fn default_max_turns() -> usize {
    25
}

fn default_auto_precheck() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_model: default_model_str(),
            max_turns: default_max_turns(),
            auto_memory_precheck: default_auto_precheck(),
            custom_instructions: None,
            custom_models: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn global_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".pi").join("config.json"))
    }

    pub fn load_or_default(work_dir: &Path) -> Self {
        // 1. Try local project config
        let local_config = work_dir.join(".pi/config.json");
        if local_config.exists() {
            if let Ok(content) = fs::read_to_string(&local_config) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }

        // 2. Try global config
        if let Some(global_config) = Self::global_config_path() {
            if global_config.exists() {
                if let Ok(content) = fs::read_to_string(&global_config) {
                    if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                        return cfg;
                    }
                }
            }
        }

        Self::default()
    }

    pub fn save_global(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::global_config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            fs::write(&path, json)?;
        }
        Ok(())
    }
}
