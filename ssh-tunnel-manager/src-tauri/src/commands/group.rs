// 分组管理命令
use crate::db;
use crate::models::{Group, CreateGroupRequest};
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// 分组 DTO - 用于前后端数据传输
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDto {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
    pub created_at: String,
}

impl From<Group> for GroupDto {
    fn from(group: Group) -> Self {
        Self {
            id: group.id,
            name: group.name,
            sort_order: group.sort_order,
            created_at: group.created_at.to_rfc3339(),
        }
    }
}

/// 获取所有分组
#[tauri::command]
pub fn get_groups() -> Result<Vec<GroupDto>, String> {
    let groups = db::get_groups().map_err(|e| e.to_string())?;
    Ok(groups.into_iter().map(GroupDto::from).collect())
}

/// 保存分组（创建或更新）
#[tauri::command]
pub fn save_group(request: CreateGroupRequest) -> Result<GroupDto, String> {
    let now = Utc::now();
    let group = Group {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        sort_order: request.sort_order.unwrap_or(0),
        created_at: now,
    };

    db::save_group(&group).map_err(|e| e.to_string())?;

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(GroupDto::from(group))
}

/// 删除分组
#[tauri::command]
pub fn delete_group(id: String) -> Result<(), String> {
    db::delete_group(&id).map_err(|e| e.to_string())?;

    // 更新托盘菜单
    crate::ssh::update_tray_menu();

    Ok(())
}