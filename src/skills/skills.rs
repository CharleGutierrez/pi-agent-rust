use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SkillManager {
    skills: HashMap<String, Skill>,
}

impl SkillManager {
    pub fn load_all(work_dir: &Path) -> Result<Self> {
        let mut skills = HashMap::new();

        // 1. Check project local .pi/skills
        let local_dir = work_dir.join(".pi/skills");
        if local_dir.exists() {
            Self::scan_dir(&local_dir, &mut skills)?;
        }

        // 2. Check global ~/.pi/skills
        if let Some(home) = dirs::home_dir() {
            let global_dir = home.join(".pi/skills");
            if global_dir.exists() {
                Self::scan_dir(&global_dir, &mut skills)?;
            }
        }

        Ok(Self { skills })
    }

    fn scan_dir(dir: &Path, skills: &mut HashMap<String, Skill>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let content = fs::read_to_string(&path)?;
                let first_line = content.lines().next().unwrap_or("").trim_start_matches('#').trim().to_string();
                let description = if first_line.is_empty() {
                    format!("Skill from {}", path.file_name().unwrap_or_default().to_string_lossy())
                } else {
                    first_line
                };

                skills.insert(
                    name.clone(),
                    Skill {
                        name,
                        description,
                        content,
                    },
                );
            }
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }
}
