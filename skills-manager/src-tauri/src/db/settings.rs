// src-tauri/src/db/settings.rs
use rusqlite::Connection;
use crate::models::Settings;

pub fn get_all(conn: &Connection) -> Result<Settings, String> {
    let default_sync_interval = get_value(conn, "default_sync_interval")?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(3600);

    let auto_sync = get_value(conn, "auto_sync")?
        .map(|v| v == "true")
        .unwrap_or(true);

    let check_interval = get_value(conn, "check_interval")?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300);

    Ok(Settings {
        default_sync_interval,
        auto_sync,
        check_interval,
    })
}

pub fn get_value(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get setting '{}': {}", key, e)),
    }
}

pub fn set_value(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, value],
    ).map_err(|e| format!("Failed to set setting '{}': {}", key, e))?;
    Ok(())
}

pub fn update_settings(conn: &Connection, settings: &Settings) -> Result<(), String> {
    set_value(conn, "default_sync_interval", &settings.default_sync_interval.to_string())?;
    set_value(conn, "auto_sync", if settings.auto_sync { "true" } else { "false" })?;
    set_value(conn, "check_interval", &settings.check_interval.to_string())?;
    Ok(())
}