// 应用设置管理命令
use crate::db;

/// 获取单个应用设置
#[tauri::command]
pub fn get_app_setting(key: String) -> Option<String> {
    db::get_app_setting(&key).ok().flatten()
}

/// 保存应用设置
#[tauri::command]
pub fn save_app_setting(key: String, value: String) -> Result<(), String> {
    db::save_app_setting(&key, &value).map_err(|e| e.to_string())
}

/// 删除应用设置
#[tauri::command]
pub fn delete_app_setting(key: String) -> Result<(), String> {
    db::delete_app_setting(&key).map_err(|e| e.to_string())
}