// src-tauri/src/models/skill.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub local_path: String,
    pub is_selected: bool,
}