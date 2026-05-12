// src-tauri/src/db/schema.rs
use rusqlite::Connection;
use crate::models::Agent;

pub fn init_schema(conn: &Connection) -> Result<(), String> {
    create_repositories_table(conn)?;
    create_agents_table(conn)?;
    create_settings_table(conn)?;
    create_skills_table(conn)?;
    Ok(())
}

fn create_repositories_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repositories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            branch TEXT NOT NULL,
            auth_type TEXT NOT NULL DEFAULT 'none',
            auth_token TEXT,
            auth_username TEXT,
            auth_password TEXT,
            sync_interval INTEGER NOT NULL DEFAULT 3600,
            last_sync TEXT,
            enabled INTEGER NOT NULL DEFAULT 1
        )",
        [],
    ).map_err(|e| format!("Failed to create repositories table: {}", e))?;
    Ok(())
}

fn create_agents_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1
        )",
        [],
    ).map_err(|e| format!("Failed to create agents table: {}", e))?;
    Ok(())
}

fn create_settings_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("Failed to create settings table: {}", e))?;
    Ok(())
}

fn create_skills_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            repo_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            path TEXT NOT NULL,
            local_path TEXT NOT NULL,
            is_selected INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| format!("Failed to create skills table: {}", e))?;
    Ok(())
}

pub fn init_default_settings(conn: &Connection) -> Result<(), String> {
    let defaults = [
        ("default_sync_interval", "3600"),
        ("auto_sync", "true"),
        ("check_interval", "300"),
    ];

    for (key, value) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        ).map_err(|e| format!("Failed to insert default setting: {}", e))?;
    }
    Ok(())
}

pub fn init_default_agents(conn: &Connection) -> Result<(), String> {
    let default_agents = Agent::default_agents();
    for agent in default_agents {
        conn.execute(
            "INSERT OR IGNORE INTO agents (id, name, path, enabled) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![agent.id, agent.name, agent.path, agent.enabled],
        ).map_err(|e| format!("Failed to insert default agent: {}", e))?;
    }
    Ok(())
}