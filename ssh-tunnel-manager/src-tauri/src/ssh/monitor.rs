use std::time::Duration;

use crate::db;
use crate::models::{LogAction, TunnelStatus};

use super::sidecar::{is_process_running, is_port_in_use, build_ssh_args, log_connection_async, SSH_PROCESSES};
use super::{TUNNELS, MONITOR_HANDLES};

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub check_interval: u64,        // 检查间隔（秒）
    pub auto_reconnect: bool,       // 是否自动重连
    pub reconnect_interval: i32,    // 重连间隔（秒）
    pub max_reconnect_attempts: i32, // 最大重连次数（0 表示无限制）
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: 5,
            auto_reconnect: false,
            reconnect_interval: 5,
            max_reconnect_attempts: 0,
        }
    }
}

/// 启动监控任务
pub fn start_monitor(config_id: String, monitor_config: MonitorConfig) {
    // 检查是否已有监控任务
    {
        let handles = MONITOR_HANDLES.lock().unwrap();
        if handles.contains_key(&config_id) {
            return;
        }
    }

    let config_id_clone = config_id.clone();
    // 使用 tauri::async_runtime::spawn 代替 tokio::spawn，确保在任何上下文都能运行
    let handle = tauri::async_runtime::spawn(async move {
        let mut reconnect_attempts = 0i32;
        let mut last_status = TunnelStatus::Running;

        loop {
            // 检查隧道状态
            let current_status = {
                let tunnels = TUNNELS.lock().unwrap();
                tunnels.get(&config_id_clone).map(|info| info.status.clone()).unwrap_or(TunnelStatus::Stopped)
            };

            // 如果隧道已停止，退出监控
            if current_status == TunnelStatus::Stopped {
                break;
            }

            // 获取 PID
            let pid = {
                let tunnels = TUNNELS.lock().unwrap();
                tunnels.get(&config_id_clone).and_then(|info| info.pid)
            };

            // 检查进程是否还在运行
            if let Some(pid) = pid {
                if !is_process_running(pid) {
                    // 进程已退出
                    handle_process_exit(&config_id_clone, monitor_config.auto_reconnect).await;

                    if monitor_config.auto_reconnect {
                        // 检查重连次数
                        if monitor_config.max_reconnect_attempts > 0
                            && reconnect_attempts >= monitor_config.max_reconnect_attempts
                        {
                            log_connection_async(config_id_clone.clone(), LogAction::Error,
                                format!("已达到最大重连次数 ({})", monitor_config.max_reconnect_attempts)).await;
                            break;
                        }

                        reconnect_attempts += 1;

                        // 等待重连间隔
                        tokio::time::sleep(Duration::from_secs(monitor_config.reconnect_interval as u64)).await;

                        // 尝试重连
                        if let Err(e) = attempt_reconnect(&config_id_clone).await {
                            log_connection_async(config_id_clone.clone(), LogAction::Error,
                                format!("自动重连失败 (尝试 {}): {}", reconnect_attempts, e)).await;
                        } else {
                            reconnect_attempts = 0; // 重置重连计数
                        }
                    } else {
                        // 不自动重连，退出监控
                        break;
                    }
                } else {
                    // 进程正常运行，重置重连计数
                    if last_status == TunnelStatus::Reconnecting {
                        reconnect_attempts = 0;
                    }
                }
            }

            last_status = current_status;

            // 等待下一次检查
            tokio::time::sleep(Duration::from_secs(monitor_config.check_interval)).await;
        }
    });

    // 保存监控句柄
    {
        let mut handles = MONITOR_HANDLES.lock().unwrap();
        handles.insert(config_id, handle);
    }
}

/// 处理进程退出
async fn handle_process_exit(config_id: &str, auto_reconnect: bool) {
    // 更新状态
    {
        let mut tunnels = TUNNELS.lock().unwrap();
        if let Some(info) = tunnels.get_mut(config_id) {
            info.status = if auto_reconnect {
                TunnelStatus::Reconnecting
            } else {
                TunnelStatus::Error
            };
            info.pid = None;
        }
    }

    // 清理进程记录
    {
        let mut processes = SSH_PROCESSES.lock().unwrap();
        processes.remove(config_id);
    }

    // 记录日志
    log_connection_async(config_id.to_string(), LogAction::Error, "SSH 进程已意外退出".to_string()).await;

    // 发送状态变化事件给前端
    super::emit_tunnel_status_event(config_id);

    // 获取隧道名称用于通知
    let config_id_for_name = config_id.to_string();
    let tunnel_name = tokio::task::spawn_blocking(move || {
        db::get_config_by_id(&config_id_for_name)
            .ok()
            .flatten()
            .map(|c| c.name)
            .unwrap_or_else(|| config_id_for_name.clone())
    })
    .await
    .unwrap_or_else(|_| config_id.to_string());

    // 发送断开通知
    if auto_reconnect {
        super::send_notification("SSH 隧道已断开", &format!("隧道 {} 连接已断开，正在尝试重连...", tunnel_name));
    } else {
        super::send_notification("SSH 隧道已断开", &format!("隧道 {} 连接已断开", tunnel_name));
    }

    // 更新托盘菜单
    super::update_tray_menu();
}

/// 尝试重连
async fn attempt_reconnect(config_id: &str) -> Result<(), String> {
    let config_id_owned = config_id.to_string();
    // 从数据库获取配置（使用 spawn_blocking）
    let config = tokio::task::spawn_blocking(move || {
        db::get_config_by_id(&config_id_owned)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "配置不存在".to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    // 检查端口是否可用
    if is_port_in_use(&config.local_host, config.local_port) {
        return Err(format!("端口 {}:{} 仍被占用", config.local_host, config.local_port));
    }

    // 构建并启动 SSH
    let args = build_ssh_args(&config);

    #[cfg(target_os = "windows")]
    let child = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("ssh")
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("启动 SSH 进程失败: {}", e))?
    };

    #[cfg(not(target_os = "windows"))]
    let child = {
        std::process::Command::new("ssh")
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动 SSH 进程失败: {}", e))?
    };

    let pid = child.id();

    // 保存进程
    {
        let mut processes = SSH_PROCESSES.lock().unwrap();
        processes.insert(config_id.to_string(), child);
    }

    // 更新隧道信息
    {
        let mut tunnels = TUNNELS.lock().unwrap();
        if let Some(info) = tunnels.get_mut(config_id) {
            info.status = TunnelStatus::Running;
            info.pid = Some(pid);
            info.error_message = None;
        }
    }

    // 记录重连日志
    log_connection_async(config_id.to_string(), LogAction::Reconnect, format!("SSH 隧道已重连 (PID: {})", pid)).await;

    // 发送状态变化事件
    super::emit_tunnel_status_event(config_id);

    // 发送重连成功通知
    super::send_notification("SSH 隧道已重连", &format!("{} 已成功重新连接 (PID: {})", config.name, pid));

    // 更新托盘菜单
    super::update_tray_menu();

    Ok(())
}

/// 停止监控任务
pub fn stop_monitor_task(config_id: &str) {
    let mut handles = MONITOR_HANDLES.lock().unwrap();
    if let Some(handle) = handles.remove(config_id) {
        handle.abort();
    }
}

/// 便捷方法：启动带默认配置的监控
pub fn start_monitor_with_defaults(config_id: String, auto_reconnect: bool, reconnect_interval: i32) {
    let config = MonitorConfig {
        check_interval: 5,
        auto_reconnect,
        reconnect_interval,
        max_reconnect_attempts: 0, // 无限制
    };
    start_monitor(config_id, config);
}
