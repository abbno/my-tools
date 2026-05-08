use crate::utils::autostart;
use crate::db;

/// 获取软件开机启动状态
#[tauri::command]
pub fn get_autostart_status() -> bool {
    autostart::is_autostart_enabled()
}

/// 设置软件开机启动状态
#[tauri::command]
pub fn set_autostart(enable: bool) -> Result<(), String> {
    if enable {
        autostart::enable_autostart().map_err(|e| e.to_string())?;
    } else {
        autostart::disable_autostart().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 设置隧道开机启动状态
#[tauri::command]
pub fn set_tunnel_autostart(config_id: String, auto_start: bool) -> Result<crate::models::Config, String> {
    db::set_auto_start(&config_id, auto_start).map_err(|e| e.to_string())
}

/// 获取所有开机启动的隧道配置
#[tauri::command]
pub fn get_autostart_tunnels() -> Result<Vec<crate::models::Config>, String> {
    db::get_auto_start_configs().map_err(|e| e.to_string())
}