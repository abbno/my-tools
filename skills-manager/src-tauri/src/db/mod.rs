// src-tauri/src/db/mod.rs
pub mod connection;
pub mod schema;
pub mod repositories;
pub mod agents;
pub mod settings;
pub mod skills;

use connection::get_connection;
use schema::init_schema;

pub fn init_database() -> Result<(), String> {
    let conn = get_connection()?;
    init_schema(&conn)?;
    init_default_data(&conn)?;
    Ok(())
}

fn init_default_data(conn: &rusqlite::Connection) -> Result<(), String> {
    schema::init_default_settings(conn)?;
    schema::init_default_agents(conn)?;
    Ok(())
}