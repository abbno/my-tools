use crate::models::{AuthType, Config, ConnectionLog, Group, LogAction, TunnelType};
use crate::utils::crypto;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row, OptionalExtension};
use std::sync::MutexGuard;

use super::DB;

/// 获取数据库连接锁
fn get_db() -> MutexGuard<'static, Option<Connection>> {
    DB.lock().unwrap()
}

// ==================== 分组操作 ====================

/// 获取所有分组
pub fn get_groups() -> Result<Vec<Group>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, created_at FROM groups ORDER BY sort_order ASC, created_at ASC"
    )?;

    let groups = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            created_at: parse_datetime(&row.get::<_, String>(3)?),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(groups)
}

// ==================== 配置操作 ====================
pub fn save_group(group: &Group) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    // 检查是否存在
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM groups WHERE id = ?1",
        params![group.id],
        |row| row.get(0),
    )?;

    if exists {
        // 更新
        conn.execute(
            "UPDATE groups SET name = ?1, sort_order = ?2 WHERE id = ?3",
            params![group.name, group.sort_order, group.id],
        )?;
    } else {
        // 新建
        conn.execute(
            "INSERT INTO groups (id, name, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                group.id,
                group.name,
                group.sort_order,
                group.created_at.to_rfc3339()
            ],
        )?;
    }

    Ok(())
}

/// 删除分组
pub fn delete_group(id: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    // 先将该分组下的配置的 group_id 设为 NULL
    conn.execute(
        "UPDATE configs SET group_id = NULL WHERE group_id = ?1",
        params![id],
    )?;

    // 删除分组
    conn.execute("DELETE FROM groups WHERE id = ?1", params![id])?;

    Ok(())
}

// ==================== 配置操作 ====================

/// 映射查询行到 Config 结构
fn map_config_row(row: &Row) -> Result<Config, rusqlite::Error> {
    let auth_type_str: String = row.get(6)?;
    let tunnel_type_str: String = row.get(10)?;

    // 解密密码
    let password: Option<String> = row.get::<_, Option<String>>(7)?;
    let decrypted_password = match password {
        Some(ref encrypted) => crypto::decrypt(encrypted).ok(),
        None => None,
    };

    // 解密密钥密码
    let key_passphrase: Option<String> = row.get::<_, Option<String>>(9)?;
    let decrypted_key_passphrase = match key_passphrase {
        Some(ref encrypted) => crypto::decrypt(encrypted).ok(),
        None => None,
    };

    Ok(Config {
        id: row.get(0)?,
        name: row.get(1)?,
        group_id: row.get(2)?,
        host: row.get(3)?,
        port: row.get(4)?,
        username: row.get(5)?,
        auth_type: parse_auth_type(&auth_type_str),
        password: decrypted_password,
        key_path: row.get(8)?,
        key_passphrase: decrypted_key_passphrase,
        tunnel_type: parse_tunnel_type(&tunnel_type_str),
        local_host: row.get(11)?,
        local_port: row.get(12)?,
        remote_host: row.get(13)?,
        remote_port: row.get(14)?,
        auto_reconnect: row.get::<_, i32>(15)? != 0,
        reconnect_interval: row.get(16)?,
        created_at: parse_datetime(&row.get::<_, String>(17)?),
        updated_at: parse_datetime(&row.get::<_, String>(18)?),
        is_favorite: row.get::<_, i32>(19)? != 0,
        favorite_order: row.get(20)?,
        auto_start: row.get::<_, i32>(21)? != 0,
    })
}

/// 获取所有配置
pub fn get_configs() -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         ORDER BY created_at DESC"
    )?;

    let configs = stmt.query_map([], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}

/// 根据 ID 获取配置
pub fn get_config_by_id(id: &str) -> Result<Option<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE id = ?1"
    )?;

    let config = stmt.query_row(params![id], map_config_row).optional()?;

    Ok(config)
}

/// 根据分组 ID 获取配置
pub fn get_configs_by_group(group_id: &str) -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE group_id = ?1
         ORDER BY created_at DESC"
    )?;

    let configs = stmt.query_map(params![group_id], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}

/// 保存配置（新建或更新）
pub fn save_config(config: &Config) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    // 加密密码
    let encrypted_password = match &config.password {
        Some(password) => Some(crypto::encrypt(password).map_err(|e| {
            rusqlite::Error::InvalidParameterName(format!("密码加密失败: {}", e))
        })?),
        None => None,
    };

    // 加密密钥密码
    let encrypted_key_passphrase = match &config.key_passphrase {
        Some(passphrase) => Some(crypto::encrypt(passphrase).map_err(|e| {
            rusqlite::Error::InvalidParameterName(format!("密钥密码加密失败: {}", e))
        })?),
        None => None,
    };

    // 检查是否存在
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM configs WHERE id = ?1",
        params![config.id],
        |row| row.get(0),
    )?;

    if exists {
        // 更新
        conn.execute(
            "UPDATE configs SET
                name = ?1, group_id = ?2, host = ?3, port = ?4, username = ?5,
                auth_type = ?6, password = ?7, key_path = ?8, key_passphrase = ?9,
                tunnel_type = ?10, local_host = ?11, local_port = ?12,
                remote_host = ?13, remote_port = ?14, auto_reconnect = ?15,
                reconnect_interval = ?16, updated_at = ?17, is_favorite = ?18, favorite_order = ?19,
                auto_start = ?20
            WHERE id = ?21",
            params![
                config.name,
                config.group_id,
                config.host,
                config.port,
                config.username,
                auth_type_to_string(&config.auth_type),
                encrypted_password,
                config.key_path,
                encrypted_key_passphrase,
                tunnel_type_to_string(&config.tunnel_type),
                config.local_host,
                config.local_port,
                config.remote_host,
                config.remote_port,
                if config.auto_reconnect { 1 } else { 0 },
                config.reconnect_interval,
                config.updated_at.to_rfc3339(),
                if config.is_favorite { 1 } else { 0 },
                config.favorite_order,
                if config.auto_start { 1 } else { 0 },
                config.id,
            ],
        )?;
    } else {
        // 新建
        conn.execute(
            "INSERT INTO configs (
                id, name, group_id, host, port, username, auth_type, password,
                key_path, key_passphrase, tunnel_type, local_host, local_port,
                remote_host, remote_port, auto_reconnect, reconnect_interval,
                created_at, updated_at, is_favorite, favorite_order, auto_start
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                config.id,
                config.name,
                config.group_id,
                config.host,
                config.port,
                config.username,
                auth_type_to_string(&config.auth_type),
                encrypted_password,
                config.key_path,
                encrypted_key_passphrase,
                tunnel_type_to_string(&config.tunnel_type),
                config.local_host,
                config.local_port,
                config.remote_host,
                config.remote_port,
                if config.auto_reconnect { 1 } else { 0 },
                config.reconnect_interval,
                config.created_at.to_rfc3339(),
                config.updated_at.to_rfc3339(),
                if config.is_favorite { 1 } else { 0 },
                config.favorite_order,
                if config.auto_start { 1 } else { 0 },
            ],
        )?;
    }

    Ok(())
}

/// 删除配置
pub fn delete_config(id: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    // 先删除相关的日志
    conn.execute(
        "DELETE FROM connection_logs WHERE config_id = ?1",
        params![id],
    )?;

    // 删除配置
    conn.execute("DELETE FROM configs WHERE id = ?1", params![id])?;

    Ok(())
}

/// 搜索配置
pub fn search_configs(keyword: &str) -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let pattern = format!("%{}%", keyword);

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE name LIKE ?1 OR host LIKE ?1 OR username LIKE ?1
         ORDER BY created_at DESC"
    )?;

    let configs = stmt.query_map(params![pattern], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}

// ==================== 常用配置操作 ====================

/// 获取常用配置列表（按 favorite_order 排序）
pub fn get_favorites() -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE is_favorite = 1
         ORDER BY favorite_order ASC"
    )?;

    let configs = stmt.query_map([], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}

/// 设置配置的常用状态
pub fn set_favorite(config_id: &str, is_favorite: bool) -> Result<Config, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    if is_favorite {
        // 标记为常用：获取当前最大 order，设置为 max + 1
        let max_order: i32 = conn.query_row(
            "SELECT COALESCE(MAX(favorite_order), 0) FROM configs WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )?;

        conn.execute(
            "UPDATE configs SET is_favorite = 1, favorite_order = ?1 WHERE id = ?2",
            params![max_order + 1, config_id],
        )?;
    } else {
        // 取消常用：先获取该配置的 order，用于后续重排
        let old_order: i32 = conn.query_row(
            "SELECT favorite_order FROM configs WHERE id = ?1",
            params![config_id],
            |row| row.get(0),
        )?;

        // 设置为非常用
        conn.execute(
            "UPDATE configs SET is_favorite = 0, favorite_order = 0 WHERE id = ?1",
            params![config_id],
        )?;

        // 重排其他常用项（填补空缺）
        conn.execute(
            "UPDATE configs SET favorite_order = favorite_order - 1
             WHERE is_favorite = 1 AND favorite_order > ?1",
            params![old_order],
        )?;
    }

    // 在同一个锁内直接查询更新后的配置（避免重复获取锁导致死锁）
    let config = conn.query_row(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path,
                key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port,
                auto_reconnect, reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs WHERE id = ?1",
        params![config_id],
        map_config_row,
    )?;

    Ok(config)
}

/// 批量更新常用配置的排序
pub fn reorder_favorites(orders: &[(String, i32)]) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    for (config_id, order) in orders {
        conn.execute(
            "UPDATE configs SET favorite_order = ?1 WHERE id = ?2 AND is_favorite = 1",
            params![order, config_id],
        )?;
    }

    Ok(())
}

/// 获取当前最大常用排序号
pub fn get_max_favorite_order() -> Result<i32, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.query_row(
        "SELECT COALESCE(MAX(favorite_order), 0) FROM configs WHERE is_favorite = 1",
        [],
        |row| row.get(0),
    )
}

/// 取消常用后重排其他常用项
pub fn reorder_after_remove_favorite(removed_order: i32) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "UPDATE configs SET favorite_order = favorite_order - 1
         WHERE is_favorite = 1 AND favorite_order > ?1",
        params![removed_order],
    )?;

    Ok(())
}

// ==================== 开机启动操作 ====================

/// 设置配置的开机启动状态
pub fn set_auto_start(config_id: &str, auto_start: bool) -> Result<Config, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "UPDATE configs SET auto_start = ?1 WHERE id = ?2",
        params![if auto_start { 1 } else { 0 }, config_id],
    )?;

    // 查询更新后的配置
    let config = conn.query_row(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path,
                key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port,
                auto_reconnect, reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs WHERE id = ?1",
        params![config_id],
        map_config_row,
    )?;

    Ok(config)
}

/// 获取所有开机启动的配置
pub fn get_auto_start_configs() -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE auto_start = 1"
    )?;

    let configs = stmt.query_map([], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}

// ==================== 日志操作 ====================

/// 获取指定配置的日志
pub fn get_logs(config_id: &str, limit: Option<i32>) -> Result<Vec<ConnectionLog>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let limit_val = limit.unwrap_or(100);

    let mut stmt = conn.prepare(
        "SELECT id, config_id, action, message, created_at
         FROM connection_logs
         WHERE config_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2"
    )?;

    let logs = stmt.query_map(params![config_id, limit_val], |row| {
        let action_str: String = row.get(2)?;
        Ok(ConnectionLog {
            id: row.get(0)?,
            config_id: row.get(1)?,
            action: parse_log_action(&action_str),
            message: row.get(3)?,
            created_at: parse_datetime(&row.get::<_, String>(4)?),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(logs)
}

/// 保存日志
pub fn save_log(log: &ConnectionLog) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "INSERT INTO connection_logs (id, config_id, action, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            log.id,
            log.config_id,
            log_action_to_string(&log.action),
            log.message,
            log.created_at.to_rfc3339()
        ],
    )?;

    Ok(())
}

/// 清除指定配置的日志
pub fn clear_logs(config_id: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "DELETE FROM connection_logs WHERE config_id = ?1",
        params![config_id],
    )?;

    Ok(())
}

/// 清除所有日志
pub fn clear_all_logs() -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute("DELETE FROM connection_logs", [])?;

    Ok(())
}

/// 清理超过指定天数的旧日志
/// 参数 days: 保留最近多少天的日志
pub fn cleanup_old_logs(days: i32) -> Result<usize, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    // 计算截止时间（当前时间减去指定天数）
    let cutoff_time = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let cutoff_str = cutoff_time.to_rfc3339();

    // 删除超过指定天数的日志
    let deleted = conn.execute(
        "DELETE FROM connection_logs WHERE created_at < ?1",
        params![cutoff_str],
    )?;

    Ok(deleted)
}

// ==================== 导入导出 ====================

/// 导出配置
pub fn export_configs(config_ids: Option<&[String]>) -> Result<Vec<Config>, rusqlite::Error> {
    match config_ids {
        Some(ids) => {
            let mut configs = Vec::new();
            for id in ids {
                if let Some(config) = get_config_by_id(id)? {
                    configs.push(config);
                }
            }
            Ok(configs)
        }
        None => get_configs(),
    }
}

// ==================== 辅助函数 ====================

/// 解析日期时间
fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// 解析认证类型
fn parse_auth_type(s: &str) -> AuthType {
    match s.to_lowercase().as_str() {
        "password" => AuthType::Password,
        "key" => AuthType::Key,
        _ => AuthType::Password,
    }
}

/// 认证类型转字符串
fn auth_type_to_string(auth_type: &AuthType) -> String {
    match auth_type {
        AuthType::Password => "password".to_string(),
        AuthType::Key => "key".to_string(),
    }
}

/// 解析隧道类型
fn parse_tunnel_type(s: &str) -> TunnelType {
    match s.to_lowercase().as_str() {
        "local" => TunnelType::Local,
        "remote" => TunnelType::Remote,
        "dynamic" => TunnelType::Dynamic,
        _ => TunnelType::Local,
    }
}

/// 隧道类型转字符串
fn tunnel_type_to_string(tunnel_type: &TunnelType) -> String {
    match tunnel_type {
        TunnelType::Local => "local".to_string(),
        TunnelType::Remote => "remote".to_string(),
        TunnelType::Dynamic => "dynamic".to_string(),
    }
}

/// 解析日志动作
fn parse_log_action(s: &str) -> LogAction {
    match s.to_lowercase().as_str() {
        "connect" => LogAction::Connect,
        "disconnect" => LogAction::Disconnect,
        "reconnect" => LogAction::Reconnect,
        "error" => LogAction::Error,
        _ => LogAction::Error,
    }
}

/// 日志动作转字符串
fn log_action_to_string(action: &LogAction) -> String {
    match action {
        LogAction::Connect => "connect".to_string(),
        LogAction::Disconnect => "disconnect".to_string(),
        LogAction::Reconnect => "reconnect".to_string(),
        LogAction::Error => "error".to_string(),
    }
}

// ==================== 应用设置操作 ====================

/// 获取单个设置项
pub fn get_app_setting(key: &str) -> Result<Option<String>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let value: Option<String> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    ).optional()?.flatten();

    Ok(value)
}

/// 保存设置项
pub fn save_app_setting(key: &str, value: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let updated_at = chrono::Utc::now().to_rfc3339();

    // 使用 INSERT OR REPLACE 实现 upsert
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params![key, value, updated_at],
    )?;

    Ok(())
}

/// 删除设置项
pub fn delete_app_setting(key: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "DELETE FROM app_settings WHERE key = ?1",
        params![key],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auth_type() {
        assert!(matches!(parse_auth_type("password"), AuthType::Password));
        assert!(matches!(parse_auth_type("key"), AuthType::Key));
        assert!(matches!(parse_auth_type("PASSWORD"), AuthType::Password));
        assert!(matches!(parse_auth_type("KEY"), AuthType::Key));
    }

    #[test]
    fn test_parse_tunnel_type() {
        assert!(matches!(parse_tunnel_type("local"), TunnelType::Local));
        assert!(matches!(parse_tunnel_type("remote"), TunnelType::Remote));
        assert!(matches!(parse_tunnel_type("dynamic"), TunnelType::Dynamic));
    }

    #[test]
    fn test_parse_log_action() {
        assert!(matches!(parse_log_action("connect"), LogAction::Connect));
        assert!(matches!(parse_log_action("disconnect"), LogAction::Disconnect));
        assert!(matches!(parse_log_action("reconnect"), LogAction::Reconnect));
        assert!(matches!(parse_log_action("error"), LogAction::Error));
    }
}
