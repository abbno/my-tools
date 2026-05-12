// src-tauri/src/db/connection.rs
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::MutexGuard;
use rusqlite::Connection;
use once_cell::sync::Lazy;

static DB_CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let db_path = get_db_path().expect("Failed to get db path");
    let conn = Connection::open(&db_path).expect("Failed to open database");
    Mutex::new(conn)
});

pub fn get_db_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get exe directory")?;
    Ok(exe_dir.join("skills-manager.db"))
}

pub fn get_connection() -> Result<MutexGuard<'static, Connection>, String> {
    DB_CONNECTION.lock().map_err(|e| format!("Failed to get db lock: {}", e))
}