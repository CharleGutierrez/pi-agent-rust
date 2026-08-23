use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Represents project structural and architectural map
#[derive(Debug, Clone)]
pub struct ProjectMap {
    pub purpose: String,
    pub stack: Vec<String>,
    pub structure: Vec<StructureEntry>,
    pub relationships: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StructureEntry {
    pub path: String,
    pub description: String,
    pub is_dir: bool,
    pub depth: usize,
}

impl ProjectMap {
    pub fn new() -> Self {
        Self {
            purpose: String::new(),
            stack: Vec::new(),
            structure: Vec::new(),
            relationships: Vec::new(),
        }
    }

    /// Read PROJECT_MAP.md from directory if present
    pub fn load_from_file(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        Ok(Some(Self::parse(&content)))
    }

    /// Parse markdown representation of project map
    pub fn parse(content: &str) -> Self {
        let mut map = Self::new();
        let mut current_section = "";

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("## Project purpose") || trimmed.starts_with("## Purpose") {
                current_section = "purpose";
                continue;
            } else if trimmed.starts_with("## Stack") || trimmed.starts_with("## Tech Stack") {
                current_section = "stack";
                continue;
            } else if trimmed.starts_with("## Structure") {
                current_section = "structure";
                continue;
            } else if trimmed.starts_with("## Relationships") {
                current_section = "relationships";
                continue;
            } else if trimmed.starts_with("## ") {
                current_section = "";
                continue;
            }

            match current_section {
                "purpose" => {
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        if !map.purpose.is_empty() {
                            map.purpose.push(' ');
                        }
                        map.purpose.push_str(trimmed);
                    }
                }
                "stack" => {
                    if trimmed.starts_with('-') || trimmed.starts_with('*') {
                        let item = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
                        if !item.is_empty() {
                            map.stack.push(item.to_string());
                        }
                    }
                }
                "structure" => {
                    if trimmed.starts_with('-') || trimmed.starts_with('*') {
                        let depth = (line.len() - line.trim_start().len()) / 2;
                        let item = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
                        let parts: Vec<&str> = item.splitn(2, "—").collect();
                        let p = parts[0].trim().trim_matches('`').to_string();
                        let desc = if parts.len() > 1 {
                            parts[1].trim().to_string()
                        } else {
                            String::new()
                        };
                        let is_dir = p.ends_with('/');
                        map.structure.push(StructureEntry {
                            path: p,
                            description: desc,
                            is_dir,
                            depth,
                        });
                    }
                }
                "relationships" => {
                    if trimmed.starts_with('-') || trimmed.starts_with('*') {
                        let item = trimmed.trim_start_matches(|c| c == '-' || c == '*' || c == ' ');
                        if !item.is_empty() {
                            map.relationships.push(item.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        map
    }

    /// Auto-generate structure from actual filesystem directory tree
    pub fn auto_scan(root_dir: &Path) -> Self {
        let mut map = Self::new();
        map.purpose = format!("Autonomous project at {}", root_dir.display());

        // Detect stack manifests
        if root_dir.join("Cargo.toml").exists() {
            map.stack.push("Rust (Cargo)".to_string());
        }
        if root_dir.join("package.json").exists() {
            map.stack.push("Node.js / TypeScript / JavaScript (npm)".to_string());
        }
        if root_dir.join("pyproject.toml").exists() || root_dir.join("requirements.txt").exists() {
            map.stack.push("Python".to_string());
        }
        if root_dir.join("go.mod").exists() {
            map.stack.push("Go".to_string());
        }

        // Walk directories respecting common ignores
        let mut entries = Vec::new();
        for entry in WalkDir::new(root_dir)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "target" && name != "node_modules" && name != "dist" && name != "build" && name != "__pycache__"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path == root_dir {
                continue;
            }
            if let Ok(rel) = path.strip_prefix(root_dir) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                let is_dir = path.is_dir();
                let depth = rel.components().count() - 1;
                let path_display = if is_dir { format!("{}/", rel_str) } else { rel_str };
                
                let desc = Self::infer_file_role(&path_display);
                entries.push(StructureEntry {
                    path: path_display,
                    description: desc,
                    is_dir,
                    depth,
                });
            }
        }

        map.structure = entries;
        map
    }

    fn infer_file_role(path: &str) -> String {
        if path.ends_with("Cargo.toml") {
            "Rust package manifest & dependencies".to_string()
        } else if path.ends_with("main.rs") {
            "CLI entry point & runner".to_string()
        } else if path.ends_with("lib.rs") {
            "Core library module declarations".to_string()
        } else if path.ends_with("package.json") {
            "Node.js package manifest & scripts".to_string()
        } else if path.ends_with("README.md") {
            "Project documentation & instructions".to_string()
        } else if path.starts_with("src/") && path.ends_with('/') {
            "Source code module directory".to_string()
        } else if path.starts_with("tests/") {
            "Automated integration and unit tests".to_string()
        } else if path.ends_with('/') {
            "Module folder".to_string()
        } else {
            "Source component".to_string()
        }
    }

    /// Convert to markdown format
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Project Map\n\n");

        out.push_str("## Project purpose\n");
        if self.purpose.is_empty() {
            out.push_str("Describe the project purpose here.\n\n");
        } else {
            out.push_str(&format!("{}\n\n", self.purpose));
        }

        if !self.stack.is_empty() {
            out.push_str("## Stack\n");
            for s in &self.stack {
                out.push_str(&format!("- {}\n", s));
            }
            out.push('\n');
        }

        out.push_str("## Structure\n");
        for entry in &self.structure {
            let indent = "  ".repeat(entry.depth);
            out.push_str(&format!("{}- `{}` — {}\n", indent, entry.path, entry.description));
        }
        out.push('\n');

        out.push_str("## Relationships\n");
        if self.relationships.is_empty() {
            out.push_str("- Core modules communicate via message passing and tool harnesses.\n");
        } else {
            for r in &self.relationships {
                out.push_str(&format!("- {}\n", r));
            }
        }
        out.push('\n');

        out
    }

    /// Save to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_markdown())
            .with_context(|| format!("Failed to write project map to {:?}", path))?;
        Ok(())
    }
}
