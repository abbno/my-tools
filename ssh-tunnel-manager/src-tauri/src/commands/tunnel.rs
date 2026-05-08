// 阧道控制命令
use crate::db;
use crate::models::{TunnelStatus, TunnelInfo};
use crate::ssh::{
    get_running_tunnels, get_tunnel_status as ssh_get_tunnel_status, restart_ssh_tunnel,
    start_monitor_with_defaults, start_ssh_tunnel, stop_monitor_task, stop_ssh_tunnel,
    check_remote_connectivity, is_port_in_use, get_port_process_info, PortProcessInfo,
};
use serde::{Deserialize, Serialize};

/// 预检查结果
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreCheckResult {
    pub remote_ok: bool,
    pub remote_error: Option<String>,
    pub local_port_ok: bool,
    pub local_port_error: Option<String>,
    pub port_process_info: Option<PortProcessInfo>,
}

/// 预检查隧道启动条件
#[tauri::command]
pub fn precheck_tunnel(config_id: String) -> Result<PreCheckResult, String> {
    let config = db::get_config_by_id(&config_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "配置不存在".to_string())?;

    // 检查远程主机连通性（2秒超时）
    let remote_result = check_remote_connectivity(&config.host, config.port, 2);
    let (remote_ok, remote_error) = match remote_result {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e)),
    };

    // 检查本地端口是否被占用
    let local_port_occupied = is_port_in_use(&config.local_host, config.local_port);
    let (local_port_ok, local_port_error, port_process_info) = if local_port_occupied {
        let info = get_port_process_info(&config.local_host, config.local_port);
        let error_msg = if let Some(ref proc_info) = info {
            format!("端口被占用 (PID: {}, 进程: {})", proc_info.pid, proc_info.name)
        } else {
            "端口被占用".to_string()
        };
        (false, Some(error_msg), info)
    } else {
        (true, None, None)
    };

    Ok(PreCheckResult {
        remote_ok,
        remote_error,
        local_port_ok,
        local_port_error,
        port_process_info,
    })
}

/// 阧道状态 DTO - 用于前后端数据传输
#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelStatusDto {
    pub config_id: String,
    pub status: String,
    pub pid: Option<u32>,
    pub message: Option<String>,
}

impl From<TunnelInfo> for TunnelStatusDto {
    fn from(info: TunnelInfo) -> Self {
        Self {
            config_id: info.config_id,
            status: match info.status {
                TunnelStatus::Stopped => "stopped",
                TunnelStatus::Starting => "starting",
                TunnelStatus::Running => "running",
                TunnelStatus::Stopping => "stopping",
                TunnelStatus::Error => "error",
                TunnelStatus::Reconnecting => "reconnecting",
            }
            .to_string(),
            pid: info.pid,
            message: info.error_message,
        }
    }
}

/// 启动隧道
#[tauri::command]
pub async fn start_tunnel(config_id: String) -> Result<TunnelStatusDto, String> {
    // 获取配置（通过 spawn_blocking 避免 blocking async runtime）
    let config = tokio::task::spawn_blocking(move || {
        db::get_config_by_id(&config_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "配置不存在".to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    // 启动隧道
    let info = start_ssh_tunnel(&config)?;

    // 启动监控
    start_monitor_with_defaults(config.id.clone(), config.auto_reconnect, config.reconnect_interval);

    Ok(info.into())
}

/// 停止隧道
#[tauri::command]
pub fn stop_tunnel(config_id: String) -> Result<TunnelStatusDto, String> {
    // 停止监控
    stop_monitor_task(&config_id);

    // 停止隧道
    let info = stop_ssh_tunnel(&config_id)?;

    Ok(info.into())
}

/// 重启隧道
#[tauri::command]
pub async fn restart_tunnel(config_id: String) -> Result<TunnelStatusDto, String> {
    // 获取配置
    let config = db::get_config_by_id(&config_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "配置不存在".to_string())?;

    // 停止监控
    stop_monitor_task(&config_id);

    // 重启隧道
    let info = restart_ssh_tunnel(&config)?;

    // 重新启动监控
    start_monitor_with_defaults(config_id.clone(), config.auto_reconnect, config.reconnect_interval);

    Ok(info.into())
}

/// 获取隧道状态
#[tauri::command]
pub fn get_tunnel_status(config_id: String) -> Result<TunnelStatusDto, String> {
    let info = ssh_get_tunnel_status(&config_id);
    Ok(info.into())
}

/// 获取所有运行中的隧道
#[tauri::command]
pub fn get_running_tunnels_cmd() -> Result<Vec<TunnelStatusDto>, String> {
    let tunnels = get_running_tunnels();
    Ok(tunnels.into_iter().map(|info| info.into()).collect())
}