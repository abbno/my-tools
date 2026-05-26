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