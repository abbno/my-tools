mod sqlite;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, LazyLock};

pub use sqlite::*;

/// 全局数据库连接
pub static DB: LazyLock<Mutex<Option<Connection>>> = LazyLock::new(|| Mutex::new(None));

/// 初始化数据库
pub fn init(db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;

    // 创建表
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            group_id TEXT,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            username TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            password TEXT,
            key_path TEXT,
            key_passphrase TEXT,
            tunnel_type TEXT NOT NULL,
            local_host TEXT NOT NULL,
            local_port INTEGER NOT NULL,
            remote_host TEXT,
            remote_port INTEGER,
            auto_reconnect INTEGER DEFAULT 0,
            reconnect_interval INTEGER DEFAULT 5,
            is_favorite INTEGER DEFAULT 0,
            favorite_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (group_id) REFERENCES groups(id)
        );

        CREATE TABLE IF NOT EXISTS connection_logs (
            id TEXT PRIMARY KEY,
            config_id TEXT NOT NULL,
            action TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (config_id) REFERENCES configs(id)
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_configs_group_id ON configs(group_id);
        CREATE INDEX IF NOT EXISTS idx_logs_config_id ON connection_logs(config_id);
        CREATE INDEX IF NOT EXISTS idx_logs_created_at ON connection_logs(created_at);
        "#,
    )?;

    // 存储连接
    *DB.lock().unwrap() = Some(conn);

    // 数据库迁移：添加新列（如果不存在）
    migrate_database()?;

    Ok(())
}

/// 数据库迁移
fn migrate_database() -> Result<(), Box<dyn std::error::Error>> {
    let db = DB.lock().unwrap();
    let conn = db.as_ref().ok_or_else(|| "数据库未初始化")?;

    // 检查 is_favorite 列是否存在
    let is_favorite_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'is_favorite'",
        [],
        |row| row.get(0),
    )?;

    if !is_favorite_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN is_favorite INTEGER DEFAULT 0", [])?;
    }

    // 检查 favorite_order 列是否存在
    let favorite_order_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'favorite_order'",
        [],
        |row| row.get(0),
    )?;

    if !favorite_order_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN favorite_order INTEGER DEFAULT 0", [])?;
    }

    // 检查 auto_start 列是否存在
    let auto_start_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'auto_start'",
        [],
        |row| row.get(0),
    )?;

    if !auto_start_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN auto_start INTEGER DEFAULT 0", [])?;
    }

    Ok(())
}
