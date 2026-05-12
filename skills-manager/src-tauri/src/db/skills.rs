// src-tauri/src/db/skills.rs
use rusqlite::{Connection, Row};
use crate::models::SkillMeta;
use uuid::Uuid;

fn row_to_skill(row: &Row) -> Result<SkillMeta, rusqlite::Error> {
    let is_selected_int: i32 = row.get(6)?;
    Ok(SkillMeta {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        path: row.get(4)?,
        local_path: row.get(5)?,
        is_selected: is_selected_int != 0,
    })
}

pub fn get_by_repo(conn: &Connection, repo_id: &str) -> Result<Vec<SkillMeta>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, repo_id, name, description, path, local_path, is_selected FROM skills WHERE repo_id = ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let skills = stmt.query_map([repo_id], row_to_skill)
        .map_err(|e| format!("Failed to query skills: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map skills: {}", e))?;

    Ok(skills)
}

pub fn get_selected_paths(conn: &Connection, repo_id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare(
        "SELECT path FROM skills WHERE repo_id = ?1 AND is_selected = 1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let paths = stmt.query_map([repo_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query selected skills: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect paths: {}", e))?;

    Ok(paths)
}

pub fn insert(conn: &Connection, skill: &SkillMeta) -> Result<(), String> {
    conn.execute(
        "INSERT INTO skills (id, repo_id, name, description, path, local_path, is_selected) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            &skill.id,
            &skill.repo_id,
            &skill.name,
            &skill.description,
            &skill.path,
            &skill.local_path,
            skill.is_selected as i32,
        ],
    ).map_err(|e| format!("Failed to insert skill: {}", e))?;
    Ok(())
}

pub fn update_selection(conn: &Connection, skill_id: &str, is_selected: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE skills SET is_selected = ?2 WHERE id = ?1",
        rusqlite::params![skill_id, is_selected as i32],
    ).map_err(|e| format!("Failed to update skill selection: {}", e))?;
    Ok(())
}

pub fn clear_by_repo(conn: &Connection, repo_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM skills WHERE repo_id = ?1",
        [repo_id],
    ).map_err(|e| format!("Failed to clear repo skills: {}", e))?;
    Ok(())
}

pub fn sync_skills(conn: &Connection, repo_id: &str, skills: &[SkillMeta], selected_paths: &[String]) -> Result<(), String> {
    // Clear existing skills for this repo
    clear_by_repo(conn, repo_id)?;

    // Insert new skills with selection status
    for skill in skills {
        let is_selected = selected_paths.contains(&skill.path);
        let skill_with_id = SkillMeta {
            id: if skill.id.is_empty() { Uuid::new_v4().to_string() } else { skill.id.clone() },
            is_selected,
            ..skill.clone()
        };
        insert(conn, &skill_with_id)?;
    }

    Ok(())
}