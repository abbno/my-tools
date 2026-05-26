// 配置管理命令
use crate::db;
use crate::models::{Config, CreateConfigRequest, UpdateConfigRequest, AuthType, TunnelType};
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// 配置 DTO - 用于前后端数据传输
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDto {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
    pub tunnel_type: String,
    pub local_host: String,
    pub local_port: i32,
    pub remote_host: Option<String>,
    pub remote_port: Option<i32>,
    pub auto_reconnect: bool,
    pub reconnect_interval: i32,
    pub is_favorite: bool,
    pub favorite_order: i32,
    pub auto_start: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// SSH 连接测试请求
#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
    pub local_host: String,
    pub local_port: i32,
}

/// 单步测试结果
#[derive(Debug, Serialize)]
pub struct TestStepResult {
    pub success: bool,
    pub message: String,
}

/// 测试详情
#[derive(Debug, Serialize)]
pub struct TestDetails {
    pub local_port: TestStepResult,
    pub tcp_connectivity: TestStepResult,
    pub ssh_login: TestStepResult,
}

/// 测试结果
#[derive(Debug, Serialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub details: TestDetails,
}

impl From<Config> for ConfigDto {
    fn from(config: Config) -> Self {
        Self {
            id: config.id,
            name: config.name,
            group_id: config.group_id,
            host: config.host,
            port: config.port,
            username: config.username,
            auth_type: auth_type_to_string(&config.auth_type),
            password: config.password,
            key_path: config.key_path,
            key_passphrase: config.key_passphrase,
            tunnel_type: tunnel_type_to_string(&config.tunnel_type),
            local_host: config.local_host,
            local_port: config.local_port,
            remote_host: config.remote_host,
            remote_port: config.remote_port,
            auto_reconnect: config.auto_reconnect,
            reconnect_interval: config.reconnect_interval,
            is_favorite: config.is_favorite,
            favorite_order: config.favorite_order,
            auto_start: config.auto_start,
            created_at: config.created_at.to_rfc3339(),
            updated_at: config.updated_at.to_rfc3339(),
        }
    }
}

fn auth_type_to_string(auth_type: &AuthType) -> String {
    match auth_type {
        AuthType::Password => "password".to_string(),
        AuthType::Key => "key".to_string(),
    }
}

fn string_to_auth_type(s: &str) -> AuthType {
    match s.to_lowercase().as_str() {
        "key" => AuthType::Key,
        _ => AuthType::Password,
    }
}

fn tunnel_type_to_string(tunnel_type: &TunnelType) -> String {
    match tunnel_type {
        TunnelType::Local => "local".to_string(),
        TunnelType::Remote => "remote".to_string(),
        TunnelType::Dynamic => "dynamic".to_string(),
    }
}

fn string_to_tunnel_type(s: &str) -> TunnelType {
    match s.to_lowercase().as_str() {
        "remote" => TunnelType::Remote,
        "dynamic" => TunnelType::Dynamic,
        _ => TunnelType::Local,
    }
}

/// 获取配置列表
#[tauri::command]
pub fn get_configs(group_id: Option<String>) -> Result<Vec<ConfigDto>, String> {
    let configs = match group_id {
        Some(gid) => db::get_configs_by_group(&gid).map_err(|e| e.to_string())?,
        None => db::get_configs().map_err(|e| e.to_string())?,
    };
    Ok(configs.into_iter().map(ConfigDto::from).collect())
}

/// 获取单个配置
#[tauri::command]
pub fn get_config(id: String) -> Result<Option<ConfigDto>, String> {
    let config = db::get_config_by_id(&id).map_err(|e| e.to_string())?;
    Ok(config.map(ConfigDto::from))
}

/// 创建配置
#[tauri::command]
pub fn save_config(request: CreateConfigRequest) -> Result<ConfigDto, String> {
    let now = Utc::now();
    let is_favorite = request.is_favorite.unwrap_or(false);
    let favorite_order = if is_favorite {
        db::get_max_favorite_order().map_err(|e| e.to_string())? + 1
    } else {
        0
    };

    let config = Config {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        group_id: request.group_id,
        host: request.host,
        port: request.port,
        username: request.username,
        auth_type: request.auth_type,
        password: request.password,
        key_path: request.key_path,
        key_passphrase: request.key_passphrase,
        tunnel_type: request.tunnel_type,
        local_host: request.local_host,
        local_port: request.local_port,
        remote_host: request.remote_host,
        remote_port: request.remote_port,
        auto_reconnect: request.auto_reconnect,
        reconnect_interval: request.reconnect_interval,
        is_favorite,
        favorite_order,
        auto_start: false,
        created_at: now,
        updated_at: now,
    };

    db::save_config(&config).map_err(|e| e.to_string())?;

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(ConfigDto::from(config))
}

/// 更新配置
#[tauri::command]
pub fn update_config(request: UpdateConfigRequest) -> Result<ConfigDto, String> {
    // 先获取现有配置以保留 created_at
    let existing = db::get_config_by_id(&request.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "配置不存在".to_string())?;

    let now = Utc::now();

    // 处理常用状态变更
    let (is_favorite, favorite_order) = match request.is_favorite {
        Some(true) if !existing.is_favorite => {
            // 新增常用：获取新的排序号
            let new_order = db::get_max_favorite_order().map_err(|e| e.to_string())? + 1;
            (true, new_order)
        }
        Some(false) if existing.is_favorite => {
            // 取消常用：排序号设为0
            (false, 0)
        }
        _ => {
            // 保持原状态
            (existing.is_favorite, existing.favorite_order)
        }
    };

    let config = Config {
        id: request.id,
        name: request.name,
        group_id: request.group_id,
        host: request.host,
        port: request.port,
        username: request.username,
        auth_type: request.auth_type,
        password: request.password,
        key_path: request.key_path,
        key_passphrase: request.key_passphrase,
        tunnel_type: request.tunnel_type,
        local_host: request.local_host,
        local_port: request.local_port,
        remote_host: request.remote_host,
        remote_port: request.remote_port,
        auto_reconnect: request.auto_reconnect,
        reconnect_interval: request.reconnect_interval,
        is_favorite,
        favorite_order,
        auto_start: existing.auto_start,
        created_at: existing.created_at,
        updated_at: now,
    };

    db::save_config(&config).map_err(|e| e.to_string())?;

    // 如果取消常用，需要重排其他常用项
    if request.is_favorite == Some(false) && existing.is_favorite {
        db::reorder_after_remove_favorite(existing.favorite_order).map_err(|e| e.to_string())?;
    }

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(ConfigDto::from(config))
}

/// 删除配置
#[tauri::command]
pub fn delete_config(id: String) -> Result<(), String> {
    db::delete_config(&id).map_err(|e| e.to_string())?;

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(())
}

/// 搜索配置
#[tauri::command]
pub fn search_configs(keyword: String) -> Result<Vec<ConfigDto>, String> {
    let configs = db::search_configs(&keyword).map_err(|e| e.to_string())?;
    Ok(configs.into_iter().map(ConfigDto::from).collect())
}

/// 导出配置
#[tauri::command]
pub fn export_configs(ids: Option<Vec<String>>) -> Result<String, String> {
    let configs = match ids {
        Some(ref id_list) => db::export_configs(Some(id_list)).map_err(|e| e.to_string())?,
        None => db::export_configs(None).map_err(|e| e.to_string())?,
    };

    let dtos: Vec<ConfigDto> = configs.into_iter().map(ConfigDto::from).collect();
    serde_json::to_string(&dtos).map_err(|e| format!("序列化失败: {}", e))
}

/// 导入配置
#[tauri::command]
pub fn import_configs(json: String) -> Result<i32, String> {
    let dtos: Vec<ConfigDto> = serde_json::from_str(&json)
        .map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let now = Utc::now();
    let mut count = 0;

    for dto in dtos {
        let config = Config {
            id: dto.id,
            name: dto.name,
            group_id: dto.group_id,
            host: dto.host,
            port: dto.port,
            username: dto.username,
            auth_type: string_to_auth_type(&dto.auth_type),
            password: dto.password,
            key_path: dto.key_path,
            key_passphrase: dto.key_passphrase,
            tunnel_type: string_to_tunnel_type(&dto.tunnel_type),
            local_host: dto.local_host,
            local_port: dto.local_port,
            remote_host: dto.remote_host,
            remote_port: dto.remote_port,
            auto_reconnect: dto.auto_reconnect,
            reconnect_interval: dto.reconnect_interval,
            is_favorite: dto.is_favorite,
            favorite_order: dto.favorite_order,
            auto_start: dto.auto_start,
            created_at: now,
            updated_at: now,
        };

        db::save_config(&config).map_err(|e| e.to_string())?;
        count += 1;
    }

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(count)
}

/// 获取常用配置列表
#[tauri::command]
pub fn get_favorites() -> Result<Vec<ConfigDto>, String> {
    let configs = db::get_favorites().map_err(|e| e.to_string())?;
    Ok(configs.into_iter().map(ConfigDto::from).collect())
}

/// 设置配置的常用状态
#[tauri::command]
pub async fn set_favorite(config_id: String, is_favorite: bool) -> Result<ConfigDto, String> {
    let config = tokio::task::spawn_blocking(move || {
        db::set_favorite(&config_id, is_favorite).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(ConfigDto::from(config))
}

/// 批量更新常用配置排序
#[tauri::command]
pub async fn reorder_favorites(orders: Vec<(String, i32)>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        db::reorder_favorites(&orders).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(())
}

/// 测试 SSH 连接
#[tauri::command]
pub async fn test_ssh_connection(request: TestConnectionRequest) -> Result<TestConnectionResult, String> {
    use crate::ssh::{is_port_in_use, get_port_process_info, check_remote_connectivity};
    use crate::utils::logger;

    let host = &request.host;
    let port = request.port;
    let username = &request.username;
    let local_host = &request.local_host;
    let local_port = request.local_port;

    logger::info(&format!("开始测试 SSH 连接: {}@{}:{}", username, host, port));

    // 1. 检查本地端口
    let local_port_result = if is_port_in_use(local_host, local_port) {
        let process_info = get_port_process_info(local_host, local_port);
        let msg = if let Some(info) = process_info {
            format!("本地端口 {}:{} 已被占用 (PID: {}, 进程: {})", local_host, local_port, info.pid, info.name)
        } else {
            format!("本地端口 {}:{} 已被占用", local_host, local_port)
        };
        TestStepResult { success: false, message: msg }
    } else {
        TestStepResult { success: true, message: format!("本地端口 {}:{} 可用", local_host, local_port) }
    };

    // 如果本地端口被占用，直接返回失败
    if !local_port_result.success {
        logger::error(&format!("SSH 连接测试失败 [本地端口]: {}", local_port_result.message));
        return Ok(TestConnectionResult {
            success: false,
            message: local_port_result.message.clone(),
            details: TestDetails {
                local_port: local_port_result,
                tcp_connectivity: TestStepResult { success: false, message: "未测试".to_string() },
                ssh_login: TestStepResult { success: false, message: "未测试".to_string() },
            },
        });
    }

    // 2. 检查 TCP 连通性
    let tcp_result = match check_remote_connectivity(host, port, 2) {
        Ok(_) => TestStepResult { success: true, message: format!("远程主机 {}:{} 可达", host, port) },
        Err(e) => TestStepResult { success: false, message: format!("远程主机 {}:{} 连接失败: {}", host, port, e) },
    };

    // 如果 TCP 连通性失败，直接返回失败
    if !tcp_result.success {
        logger::error(&format!("SSH 连接测试失败 [TCP连通性]: {}", tcp_result.message));
        return Ok(TestConnectionResult {
            success: false,
            message: tcp_result.message.clone(),
            details: TestDetails {
                local_port: local_port_result,
                tcp_connectivity: tcp_result,
                ssh_login: TestStepResult { success: false, message: "未测试".to_string() },
            },
        });
    }

    // 3. 测试 SSH 登录认证
    let ssh_result = test_ssh_login(&request);

    // 构建最终结果
    let success = ssh_result.success;
    let message = if success {
        "连接测试成功".to_string()
    } else {
        ssh_result.message.clone()
    };

    if success {
        logger::info(&format!("SSH 连接测试成功: {}@{}:{}", username, host, port));
    } else {
        logger::error(&format!("SSH 连接测试失败 [SSH登录]: {}", ssh_result.message));
    }

    Ok(TestConnectionResult {
        success,
        message,
        details: TestDetails {
            local_port: local_port_result,
            tcp_connectivity: tcp_result,
            ssh_login: ssh_result,
        },
    })
}

/// 测试 SSH 登录认证
fn test_ssh_login(request: &TestConnectionRequest) -> TestStepResult {
    use std::process::Command;

    let host = &request.host;
    let port = request.port;
    let username = &request.username;
    let auth_type = &request.auth_type;

    // 输入验证：防止参数注入
    if host.contains("-o") || host.contains(' ') || host.contains('\t') {
        return TestStepResult {
            success: false,
            message: "主机地址格式无效，不能包含空格或 SSH 选项".to_string()
        };
    }
    if username.contains("-o") || username.contains(' ') || username.contains('\t') {
        return TestStepResult {
            success: false,
            message: "用户名格式无效，不能包含空格或 SSH 选项".to_string()
        };
    }

    // 端口范围验证
    if port < 1 || port > 65535 {
        return TestStepResult {
            success: false,
            message: format!("端口范围无效: {} (应在 1-65535)", port)
        };
    }

    // 顺序构建 SSH 命令参数
    let mut args = Vec::new();

    // 密钥认证：添加密钥路径
    if auth_type == "key" {
        if let Some(key_path) = &request.key_path {
            args.push("-i".to_string());
            args.push(key_path.clone());
        }
    }

    // SSH 选项
    args.push("-o".to_string());
    args.push("BatchMode=yes".to_string());
    args.push("-o".to_string());
    args.push("ConnectTimeout=10".to_string());
    args.push("-o".to_string());
    args.push("StrictHostKeyChecking=accept-new".to_string());

    // 端口
    args.push("-p".to_string());
    args.push(port.to_string());

    // 用户名和主机
    args.push(format!("{}@{}", username, host));

    // 执行 exit 命令
    args.push("exit".to_string());

    // Windows: 隐藏窗口
    #[cfg(target_os = "windows")]
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("ssh")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    };

    #[cfg(not(target_os = "windows"))]
    let output = {
        Command::new("ssh")
            .args(&args)
            .output()
    };

    match output {
        Ok(output) => {
            if output.status.success() {
                TestStepResult {
                    success: true,
                    message: format!("SSH 登录认证成功 ({}认证)", auth_type)
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let error_msg = parse_ssh_error(&stderr);
                TestStepResult { success: false, message: error_msg }
            }
        }
        Err(e) => {
            TestStepResult {
                success: false,
                message: format!("SSH 命令执行失败: {}", e)
            }
        }
    }
}

/// 解析 SSH 错误信息
fn parse_ssh_error(stderr: &str) -> String {
    // 常见错误模式
    if stderr.contains("Permission denied") {
        if stderr.contains("publickey") {
            "SSH 登录失败: 密钥认证失败，请检查密钥路径和密码".to_string()
        } else if stderr.contains("password") {
            "SSH 登录失败: 密码认证失败，请检查用户名和密码".to_string()
        } else {
            "SSH 登录失败: 认证被拒绝".to_string()
        }
    } else if stderr.contains("Connection timed out") {
        "SSH 登录超时（10秒）".to_string()
    } else if stderr.contains("Host key verification failed") {
        "SSH 登录失败: 主机密钥验证失败".to_string()
    } else if stderr.contains("No such file or directory") && stderr.contains("identity file") {
        "SSH 登录失败: 密钥文件不存在".to_string()
    } else {
        // 提取第一行错误信息
        let first_line = stderr.lines().next().unwrap_or("未知错误");
        format!("SSH 登录失败: {}", first_line)
    }
}