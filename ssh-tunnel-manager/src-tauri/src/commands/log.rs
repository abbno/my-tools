// 日志管理命令
use crate::db;
use crate::models::{ConnectionLog, LogAction};
use serde::{Deserialize, Serialize};

/// 日志 DTO - 用于前后端数据传输
#[derive(Debug, Serialize, Deserialize)]
pub struct LogDto {
    pub id: String,
    pub config_id: String,
    pub action: String,
    pub message: String,
    pub created_at: String,
}

impl From<ConnectionLog> for LogDto {
    fn from(log: ConnectionLog) -> Self {
        Self {
            id: log.id,
            config_id: log.config_id,
            action: match log.action {
                LogAction::Connect => "connect",
                LogAction::Disconnect => "disconnect",
                LogAction::Reconnect => "reconnect",
                LogAction::Error => "error",
            }
            .to_string(),
            message: log.message,
            created_at: log.created_at.to_rfc3339(),
        }
    }
}

/// 获取指定配置的日志
#[tauri::command]
pub fn get_logs(config_id: String, limit: Option<i32>) -> Result<Vec<LogDto>, String> {
    let logs = db::get_logs(&config_id, limit).map_err(|e| e.to_string())?;
    Ok(logs.into_iter().map(LogDto::from).collect())
}

/// 清除指定配置的日志
#[tauri::command]
pub fn clear_logs(config_id: String) -> Result<(), String> {
    db::clear_logs(&config_id).map_err(|e| e.to_string())?;
    Ok(())
}

/// 清理超过指定天数的旧日志（默认30天）
#[tauri::command]
pub fn cleanup_logs(days: Option<i32>) -> Result<usize, String> {
    let days_val = days.unwrap_or(30);
    db::cleanup_old_logs(days_val).map_err(|e| e.to_string())
}

/// 清除所有日志
#[tauri::command]
pub fn clear_all_logs() -> Result<(), String> {
    db::clear_all_logs().map_err(|e| e.to_string())?;
    Ok(())
}