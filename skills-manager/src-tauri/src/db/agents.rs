// src-tauri/src/db/agents.rs
use rusqlite::{Connection, Row};
use crate::models::Agent;

fn row_to_agent(row: &Row) -> Result<Agent, rusqlite::Error> {
    let enabled_int: i32 = row.get(3)?;
    Ok(Agent {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        enabled: enabled_int != 0,
    })
}

pub fn get_all(conn: &Connection) -> Result<Vec<Agent>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, enabled FROM agents"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let agents = stmt.query_map([], row_to_agent)
        .map_err(|e| format!("Failed to query agents: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map agents: {}", e))?;

    Ok(agents)
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Agent>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, enabled FROM agents WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt.query_row([id], row_to_agent).ok();
    Ok(result)
}

pub fn insert(conn: &Connection, agent: &Agent) -> Result<(), String> {
    conn.execute(
        "INSERT INTO agents (id, name, path, enabled) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![&agent.id, &agent.name, &agent.path, agent.enabled as i32],
    ).map_err(|e| format!("Failed to insert agent: {}", e))?;
    Ok(())
}

pub fn update(conn: &Connection, agent: &Agent) -> Result<(), String> {
    conn.execute(
        "UPDATE agents SET name = ?2, path = ?3, enabled = ?4 WHERE id = ?1",
        rusqlite::params![&agent.id, &agent.name, &agent.path, agent.enabled as i32],
    ).map_err(|e| format!("Failed to update agent: {}", e))?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM agents WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Failed to delete agent: {}", e))?;
    Ok(())
}