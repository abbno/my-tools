use std::fs;
use crate::git::get_repo_path;

#[tauri::command]
pub fn read_skill_content(repo_id: String, skill_path: String) -> Result<String, String> {
    let repo_path = get_repo_path(&repo_id)?;
    let skill_md_path = repo_path.join(&skill_path).join("SKILL.md");

    if !skill_md_path.exists() {
        return Err(format!("SKILL.md not found at {}", skill_md_path.to_string_lossy()));
    }

    fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))
}