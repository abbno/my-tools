use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{Child, Command};
use std::sync::{Mutex, LazyLock};
use std::time::Duration;

use tauri::Emitter;

use crate::db;
use crate::models::{Config, ConnectionLog, LogAction, TunnelInfo, TunnelStatus, TunnelType};
use crate::utils::logger;

use super::{APP_HANDLE, TUNNELS};

/// SSH 进程信息
pub static SSH_PROCESSES: LazyLock<Mutex<HashMap<String, Child>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 监控任务句柄
pub static MONITOR_HANDLES: LazyLock<Mutex<HashMap<String, tauri::async_runtime::JoinHandle<()>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 端口占用进程信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortProcessInfo {
    pub pid: u32,
    pub name: String,
}

/// 检查远程主机连通性（带超时）- 支持域名和IP
pub fn check_remote_connectivity(host: &str, port: i32, timeout_secs: u64) -> Result<(), String> {
    use std::net::SocketAddr;
    use std::time::Instant;

    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_secs(timeout_secs);
    let start = Instant::now();

    // 先尝试直接解析为 SocketAddr（适用于 IP 地址）
    if let Ok(socket_addr) = addr_str.parse::<SocketAddr>() {
        println!("[预检查] 尝试连接 IP 地址: {}", socket_addr);
        match TcpStream::connect_timeout(&socket_addr, timeout) {
            Ok(_) => {
                println!("[预检查] 连接成功: {}", socket_addr);
                return Ok(())
            }
            Err(e) => {
                println!("[预检查] 连接失败: {}, 错误: {}", socket_addr, e);
                return Err(format!("远程主机 {}:{} 连接失败: {}", host, port, e));
            }
        }
    }

    // 如果是域名，使用 to_socket_addrs 解析
    println!("[预检查] 尝试解析域名: {}", addr_str);
    let socket_addrs = addr_str.to_socket_addrs()
        .map_err(|e| {
            println!("[预检查] 域名解析失败: {}, 错误: {}", addr_str, e);
            format!("域名解析失败 {}: {}", addr_str, e)
        })?;

    // 收集所有解析出的地址
    let addrs: Vec<SocketAddr> = socket_addrs.collect();
    println!("[预检查] 解析出 {} 个地址: {:?}", addrs.len(), addrs);

    // 计算剩余超时时间
    let elapsed = start.elapsed();
    let remaining_timeout = timeout - elapsed;
    println!("[预检查] DNS解析耗时 {}ms, 剩余超时 {}ms", elapsed.as_millis(), remaining_timeout.as_millis());

    if remaining_timeout <= Duration::ZERO {
        println!("[预检查] 超时：DNS解析耗时过长");
        return Err(format!("远程主机 {}:{} 连接超时（DNS解析耗时过长）", host, port));
    }

    // 尝试连接解析出的每个地址
    for socket_addr in &addrs {
        println!("[预检查] 尝试连接: {}", socket_addr);
        match TcpStream::connect_timeout(socket_addr, remaining_timeout) {
            Ok(_) => {
                println!("[预检查] 连接成功: {}", socket_addr);
                return Ok(())
            }
            Err(e) => {
                println!("[预检查] 连接失败: {}, 错误: {}", socket_addr, e);
            }
        }
    }

    println!("[预检查] 所有地址连接均失败");
    Err(format!("远程主机 {}:{} 连接失败（尝试了 {} 个地址）", host, port, addrs.len()))
}

/// 获取占用本地端口的进程信息（Windows）
#[cfg(target_os = "windows")]
pub fn get_port_process_info(host: &str, port: i32) -> Option<PortProcessInfo> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // 使用 netstat 查找占用端口的进程
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let target_addr = format!("{}:{}", host, port);
    println!("[端口检查] 目标地址: {}", target_addr);
    println!("[端口检查] netstat 输出:\n{}", stdout);

    // 解析 netstat 输出，查找占用端口的 PID
    for line in stdout.lines() {
        println!("[端口检查] 检查行: {}", line);
        if line.contains(&target_addr) {
            println!("[端口检查] 找到匹配行: {}", line);
            let parts: Vec<&str> = line.split_whitespace().collect();
            println!("[端口检查] 分割结果: {:?}", parts);
            if parts.len() >= 5 {
                let pid = parts[4].parse::<u32>().ok()?;
                println!("[端口检查] 解析到 PID: {}", pid);
                if pid > 0 {
                    // 获取进程名称
                    let tasklist_output = Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
                        .creation_flags(CREATE_NO_WINDOW)
                        .output()
                        .ok()?;

                    let tasklist_stdout = String::from_utf8_lossy(&tasklist_output.stdout);
                    println!("[端口检查] tasklist 输出:\n{}", tasklist_stdout);

                    // CSV 格式: "Image Name","PID","Session Name","Session#","Mem Usage"
                    for task_line in tasklist_stdout.lines() {
                        if task_line.contains(&pid.to_string()) {
                            // 解析 CSV 格式，提取进程名称
                            let proc_name = task_line
                                .split(',')
                                .next()
                                .map(|s| s.trim().replace('"', ""))
                                .unwrap_or_else(|| "Unknown".to_string());
                            println!("[端口检查] 进程名称: {}", proc_name);

                            return Some(PortProcessInfo { pid, name: proc_name });
                        }
                    }

                    // 如果 CSV 格式解析失败，返回默认值
                    return Some(PortProcessInfo { pid, name: "Unknown".to_string() });
                }
            }
        }
    }

    println!("[端口检查] 未找到占用端口的进程");
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_port_process_info(host: &str, port: i32) -> Option<PortProcessInfo> {
    // Linux/macOS 使用 lsof 或 ss
    let output = Command::new("ss")
        .args(["-tlnp"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let target_port = format!(":{}", port);

    for line in stdout.lines() {
        if line.contains(&target_port) && line.contains(host) {
            // 解析 pid/进程名
            if let Some(pid_str) = line.split("pid=").nth(1) {
                let pid = pid_str.split(',').next()?.parse::<u32>().ok()?;
                let name = pid_str.split(',').nth(1)?.split(')').next()?.to_string();
                return Some(PortProcessInfo { pid, name });
            }
        }
    }

    None
}

/// 构建 SSH 命令参数
pub fn build_ssh_args(config: &Config) -> Vec<String> {
    let mut args = Vec::new();

    // 基本连接参数
    args.push("-p".to_string());
    args.push(config.port.to_string());

    // 用户名和主机
    args.push(format!("{}@{}", config.username, config.host));

    // 不执行远程命令
    args.push("-N".to_string());

    // 保持连接参数
    args.push("-o".to_string());
    args.push("ServerAliveInterval=15".to_string());
    args.push("-o".to_string());
    args.push("ServerAliveCountMax=3".to_string());

    // 禁用严格主机密钥检查（首次连接）
    args.push("-o".to_string());
    args.push("StrictHostKeyChecking=accept-new".to_string());

    // 根据隧道类型构建参数
    match config.tunnel_type {
        TunnelType::Local => {
            // 本地转发: -L local_host:local_port:remote_host:remote_port
            if let (Some(remote_host), Some(remote_port)) = (&config.remote_host, config.remote_port) {
                args.push("-L".to_string());
                args.push(format!("{}:{}:{}:{}",
                    config.local_host,
                    config.local_port,
                    remote_host,
                    remote_port
                ));
            }
        }
        TunnelType::Remote => {
            // 远程转发: -R remote_port:local_host:local_port
            if let (Some(remote_port), Some(_remote_host)) = (config.remote_port, &config.remote_host) {
                args.push("-R".to_string());
                args.push(format!("{}:{}:{}",
                    remote_port,
                    config.local_host,
                    config.local_port
                ));
            }
        }
        TunnelType::Dynamic => {
            // 动态转发: -D local_host:local_port (SOCKS 代理)
            args.push("-D".to_string());
            args.push(format!("{}:{}", config.local_host, config.local_port));
        }
    }

    // 密钥认证
    if let Some(key_path) = &config.key_path {
        args.push("-i".to_string());
        args.push(key_path.clone());
    }

    args
}

/// 检查端口是否被占用
pub fn is_port_in_use(host: &str, port: i32) -> bool {
    let addr = format!("{}:{}", host, port);
    TcpStream::connect(&addr).is_ok()
}

/// 启动 SSH 隧道（带预检查）
pub fn start_ssh_tunnel(config: &Config) -> Result<TunnelInfo, String> {
    // 先停止可能存在的旧监控任务和清除旧状态
    stop_monitor(&config.id);
    {
        let mut tunnels = TUNNELS.lock().unwrap();
        tunnels.remove(&config.id);
    }

    logger::info(&format!(
        "开始建立 SSH 隧道: {} ({}@{}:{})",
        config.name, config.username, config.host, config.port
    ));

    // 检查远程主机连通性（2秒超时）
    if let Err(e) = check_remote_connectivity(&config.host, config.port, 2) {
        let error_msg = format!("远程主机连接失败: {}", e);
        logger::error(&format!("SSH 隧道启动失败 [{}]: {}", config.name, error_msg));
        log_connection(&config.id, LogAction::Error, error_msg.clone());
        super::send_notification("SSH 隧道连接失败", &format!("{}: {}", config.name, error_msg));
        emit_error_status(&config.id, &error_msg);
        return Err(error_msg);
    }

    // 检查本地端口是否被占用
    if is_port_in_use(&config.local_host, config.local_port) {
        // 获取占用端口的进程信息
        let process_info = get_port_process_info(&config.local_host, config.local_port);
        let error_msg = if let Some(info) = process_info {
            format!("本地端口 {}:{} 已被占用 (PID: {}, 进程: {})",
                config.local_host, config.local_port, info.pid, info.name)
        } else {
            format!("本地端口 {}:{} 已被占用", config.local_host, config.local_port)
        };

        logger::error(&format!("SSH 隧道启动失败 [{}]: {}", config.name, error_msg));
        log_connection(&config.id, LogAction::Error, error_msg.clone());
        super::send_notification("SSH 隧道连接失败", &format!("{}: {}", config.name, error_msg));
        emit_error_status(&config.id, &error_msg);

        // 返回包含进程信息的错误
        return Err(error_msg);
    }

    // 构建 SSH 参数
    let args = build_ssh_args(config);

    // 启动 SSH 进程
    #[cfg(target_os = "windows")]
    let child = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("ssh")
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| {
                let error_msg = format!("启动 SSH 进程失败: {}", e);
                logger::error(&format!("SSH 隧道启动失败 [{}]: {}", config.name, error_msg));
                log_connection(&config.id, LogAction::Error, error_msg.clone());
                super::send_notification("SSH 隧道连接失败", &format!("{}: {}", config.name, error_msg));
                error_msg
            })?
    };

    #[cfg(not(target_os = "windows"))]
    let child = {
        Command::new("ssh")
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                let error_msg = format!("启动 SSH 进程失败: {}", e);
                logger::error(&format!("SSH 隧道启动失败 [{}]: {}", config.name, error_msg));
                log_connection(&config.id, LogAction::Error, error_msg.clone());
                super::send_notification("SSH 隧道连接失败", &format!("{}: {}", config.name, error_msg));
                error_msg
            })?
    };

    let pid = child.id();

    // 保存进程
    {
        let mut processes = SSH_PROCESSES.lock().unwrap();
        processes.insert(config.id.clone(), child);
    }

    // 创建隧道信息
    let tunnel_info = TunnelInfo {
        config_id: config.id.clone(),
        status: TunnelStatus::Running,
        pid: Some(pid),
        error_message: None,
    };

    // 保存隧道状态
    {
        let mut tunnels = TUNNELS.lock().unwrap();
        tunnels.insert(config.id.clone(), tunnel_info.clone());
    }

    // 记录连接日志
    logger::info(&format!("SSH 隧道启动成功 [{}] (PID: {})", config.name, pid));
    log_connection(&config.id, LogAction::Connect, format!("SSH 隧道已启动 (PID: {})", pid));

    // 发送事件通知
    emit_tunnel_event(&config.id, "started", Some(pid), None);

    // 发送状态变化事件给前端
    super::emit_tunnel_status_event(&config.id);

    // 发送成功通知
    super::send_notification("SSH 隧道已连接", &format!("{} 已成功启动 (PID: {})", config.name, pid));

    // 更新托盘菜单
    super::update_tray_menu();

    Ok(tunnel_info)
}

/// 停止 SSH 隧道
pub fn stop_ssh_tunnel(config_id: &str) -> Result<TunnelInfo, String> {
    // 检查是否在运行，同时获取配置名称用于通知
    let (pid, config_name) = {
        let tunnels = TUNNELS.lock().unwrap();
        match tunnels.get(config_id) {
            Some(info) if info.status == TunnelStatus::Running => {
                // 从数据库获取配置名称
                let name = db::get_config_by_id(config_id)
                    .ok()
                    .flatten()
                    .map(|c| c.name)
                    .unwrap_or_else(|| config_id.to_string());
                (info.pid, name)
            }
            _ => return Err("隧道未在运行".to_string()),
        }
    };

    // 停止监控任务
    stop_monitor(config_id);

    // 终止进程
    {
        let mut processes = SSH_PROCESSES.lock().unwrap();
        if let Some(mut child) = processes.remove(config_id) {
            // 尝试正常终止
            if let Err(e) = child.kill() {
                log_connection(config_id, LogAction::Error, format!("终止进程失败: {}", e));
            } else {
                log_connection(config_id, LogAction::Disconnect, "SSH 隧道已停止".to_string());
            }
        }
    }

    // 更新状态
    let tunnel_info = TunnelInfo {
        config_id: config_id.to_string(),
        status: TunnelStatus::Stopped,
        pid: None,
        error_message: None,
    };

    {
        let mut tunnels = TUNNELS.lock().unwrap();
        tunnels.insert(config_id.to_string(), tunnel_info.clone());
    }

    // 发送事件通知
    emit_tunnel_event(config_id, "stopped", pid, None);

    // 发送状态变化事件给前端
    super::emit_tunnel_status_event(config_id);

    // 发送停止通知
    super::send_notification("SSH 隧道已停止", &format!("{} 已手动停止", config_name));

    // 更新托盘菜单
    super::update_tray_menu();

    Ok(tunnel_info)
}

/// 获取隧道状态
pub fn get_tunnel_status(config_id: &str) -> TunnelInfo {
    let tunnels = TUNNELS.lock().unwrap();
    tunnels.get(config_id).cloned().unwrap_or_else(|| TunnelInfo::new(config_id.to_string()))
}

/// 停止所有运行中的 SSH 隧道
pub fn stop_all_tunnels() {
    // 获取所有运行中的隧道 ID
    let running_ids: Vec<String> = {
        let tunnels = TUNNELS.lock().unwrap();
        tunnels
            .values()
            .filter(|info| info.status == TunnelStatus::Running)
            .map(|info| info.config_id.clone())
            .collect()
    };

    // 停止每个隧道的监控任务（abort 并移除）
    for config_id in &running_ids {
        stop_monitor(config_id);
    }

    // 终止所有 SSH 进程
    {
        let mut processes = SSH_PROCESSES.lock().unwrap();
        for config_id in &running_ids {
            if let Some(mut child) = processes.remove(config_id) {
                let _ = child.kill();
                log_connection(config_id, LogAction::Disconnect, "软件退出，SSH 隧道已停止".to_string());
            }
        }
    }

    // 清空隧道状态和进程记录
    TUNNELS.lock().unwrap().clear();
    SSH_PROCESSES.lock().unwrap().clear();

    println!("已停止所有 SSH 隧道连接");
}

/// 获取所有运行中的隧道
pub fn get_running_tunnels() -> Vec<TunnelInfo> {
    let tunnels = TUNNELS.lock().unwrap();
    tunnels
        .values()
        .filter(|info| info.status == TunnelStatus::Running)
        .cloned()
        .collect()
}

/// 重启 SSH 隧道
pub fn restart_ssh_tunnel(config: &Config) -> Result<TunnelInfo, String> {
    // 先停止（忽略错误）
    let _ = stop_ssh_tunnel(&config.id);

    // 等待端口释放
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 重新启动
    start_ssh_tunnel(config)
}

/// 检查进程是否仍在运行
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::fs;
        fs::metadata(format!("/proc/{}", pid)).is_ok()
    }
}

/// 记录连接日志（同步版本，用于同步上下文）
pub fn log_connection(config_id: &str, action: LogAction, message: String) {
    let log = ConnectionLog::new(config_id.to_string(), action, message);
    if let Err(e) = db::save_log(&log) {
        eprintln!("保存日志失败: {}", e);
    }
}

/// 记录连接日志（异步版本，用于 async 上下文）
pub async fn log_connection_async(config_id: String, action: LogAction, message: String) {
    tokio::task::spawn_blocking(move || {
        let log = ConnectionLog::new(config_id, action, message);
        if let Err(e) = db::save_log(&log) {
            eprintln!("保存日志失败: {}", e);
        }
    })
    .await
    .ok();
}

/// 发送隧道事件
fn emit_tunnel_event(config_id: &str, event_type: &str, pid: Option<u32>, error: Option<&str>) {
    if let Some(app_handle) = APP_HANDLE.lock().unwrap().as_ref() {
        let mut payload = serde_json::Map::new();
        payload.insert("config_id".to_string(), serde_json::Value::String(config_id.to_string()));
        payload.insert("event_type".to_string(), serde_json::Value::String(event_type.to_string()));

        if let Some(pid) = pid {
            payload.insert("pid".to_string(), serde_json::Value::Number(pid.into()));
        }

        if let Some(error) = error {
            payload.insert("error".to_string(), serde_json::Value::String(error.to_string()));
        }

        let _ = app_handle.emit("tunnel-event", payload);
    }
}

/// 停止监控任务
pub fn stop_monitor(config_id: &str) {
    let mut handles = MONITOR_HANDLES.lock().unwrap();
    if let Some(handle) = handles.remove(config_id) {
        handle.abort();
    }
}

/// 发送错误状态事件给前端
fn emit_error_status(config_id: &str, error_msg: &str) {
    // 更新隧道状态为 Error
    {
        let mut tunnels = TUNNELS.lock().unwrap();
        tunnels.insert(config_id.to_string(), TunnelInfo {
            config_id: config_id.to_string(),
            status: TunnelStatus::Error,
            pid: None,
            error_message: Some(error_msg.to_string()),
        });
    }
    // 发送状态变化事件
    super::emit_tunnel_status_event(config_id);
}