mod autostart;
mod sidecar;
mod monitor;

use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use tauri::AppHandle;
use tauri::Emitter;

pub use autostart::*;
pub use monitor::*;
pub use sidecar::*;

use crate::models::TunnelInfo;
use crate::models::TunnelStatus;

/// 运行中的隧道信息
pub static TUNNELS: LazyLock<Mutex<HashMap<String, TunnelInfo>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 应用句柄
pub static APP_HANDLE: LazyLock<Mutex<Option<AppHandle>>> = LazyLock::new(|| Mutex::new(None));

/// 初始化 SSH 管理器
pub fn init(app_handle: AppHandle) {
    *APP_HANDLE.lock().unwrap() = Some(app_handle);
}

/// 更新托盘菜单（代理到 lib.rs）
pub fn update_tray_menu() {
    if let Some(app) = APP_HANDLE.lock().unwrap().as_ref() {
        crate::update_tray_menu(app);
    }
}

/// 发送通知
pub fn send_notification(title: &str, body: &str) {
    if let Some(app) = APP_HANDLE.lock().unwrap().as_ref() {
        use tauri_plugin_notification::NotificationExt;
        let _ = app.notification().builder()
            .title(title)
            .body(body)
            .show();
    }
}

/// 发送隧道状态变化事件给前端
pub fn emit_tunnel_status_event(config_id: &str) {
    if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
        let tunnels = TUNNELS.lock().unwrap();
        let info = tunnels.get(config_id);
        if let Some(info) = info {
            let status_str = match info.status {
                TunnelStatus::Running => "running",
                TunnelStatus::Stopped => "stopped",
                TunnelStatus::Starting => "starting",
                TunnelStatus::Stopping => "stopping",
                TunnelStatus::Error => "error",
                TunnelStatus::Reconnecting => "reconnecting",
            };

            let mut payload = serde_json::Map::new();
            payload.insert("configId".to_string(), serde_json::Value::String(config_id.to_string()));
            payload.insert("status".to_string(), serde_json::Value::String(status_str.to_string()));
            if let Some(pid) = info.pid {
                payload.insert("pid".to_string(), serde_json::Value::Number(pid.into()));
            }
            if let Some(error) = &info.error_message {
                payload.insert("errorMessage".to_string(), serde_json::Value::String(error.clone()));
            }

            let _ = app_handle.emit("tunnel-status-changed", payload);
        }
    }
}
