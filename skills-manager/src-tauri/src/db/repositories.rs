// src-tauri/src/db/repositories.rs
use rusqlite::{Connection, Row};
use crate::models::{Repository, AuthConfig};
use chrono::{DateTime, Utc};

fn row_to_repository(row: &Row) -> Result<Repository, rusqlite::Error> {
    let auth_type: String = row.get(4)?;
    let auth_token: Option<String> = row.get(5)?;
    let auth_username: Option<String> = row.get(6)?;
    let auth_password: Option<String> = row.get(7)?;
    let last_sync_str: Option<String> = row.get(9)?;
    let enabled_int: i32 = row.get(10)?;

    let last_sync = last_sync_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

    Ok(Repository {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        branch: row.get(3)?,
        auth: AuthConfig {
            auth_type,
            token: auth_token,
            username: auth_username,
            password: auth_password,
        },
        sync_interval: row.get::<_, i64>(8)? as u64,
        selected_skills: Vec::new(), // Will be populated from skills table
        last_sync,
        enabled: enabled_int != 0,
    })
}

pub fn get_all(conn: &Connection) -> Result<Vec<Repository>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, branch, auth_type, auth_token, auth_username, auth_password, sync_interval, last_sync, enabled FROM repositories"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let repos = stmt.query_map([], row_to_repository)
        .map_err(|e| format!("Failed to query repositories: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map repositories: {}", e))?;

    Ok(repos)
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Repository>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, branch, auth_type, auth_token, auth_username, auth_password, sync_interval, last_sync, enabled FROM repositories WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt.query_row([id], row_to_repository).ok();
    Ok(result)
}

pub fn insert(conn: &Connection, repo: &Repository) -> Result<(), String> {
    let last_sync_str = repo.last_sync.map(|dt| dt.to_rfc3339());
    conn.execute(
        "INSERT INTO repositories (id, name, url, branch, auth_type, auth_token, auth_username, auth_password, sync_interval, last_sync, enabled) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            &repo.id,
            &repo.name,
            &repo.url,
            &repo.branch,
            &repo.auth.auth_type,
            repo.auth.token.as_ref(),
            repo.auth.username.as_ref(),
            repo.auth.password.as_ref(),
            repo.sync_interval as i64,
            last_sync_str.as_ref(),
            repo.enabled as i32,
        ],
    ).map_err(|e| format!("Failed to insert repository: {}", e))?;
    Ok(())
}

pub fn update(conn: &Connection, repo: &Repository) -> Result<(), String> {
    let last_sync_str = repo.last_sync.map(|dt| dt.to_rfc3339());
    conn.execute(
        "UPDATE repositories SET name = ?2, url = ?3, branch = ?4, auth_type = ?5, auth_token = ?6, auth_username = ?7, auth_password = ?8, sync_interval = ?9, last_sync = ?10, enabled = ?11 WHERE id = ?1",
        rusqlite::params![
            &repo.id,
            &repo.name,
            &repo.url,
            &repo.branch,
            &repo.auth.auth_type,
            repo.auth.token.as_ref(),
            repo.auth.username.as_ref(),
            repo.auth.password.as_ref(),
            repo.sync_interval as i64,
            last_sync_str.as_ref(),
            repo.enabled as i32,
        ],
    ).map_err(|e| format!("Failed to update repository: {}", e))?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM repositories WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Failed to delete repository: {}", e))?;
    Ok(())
}