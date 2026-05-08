use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 应用设置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    /// 设置项键名
    pub key: String,
    /// 设置项值
    pub value: String,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 创建设置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAppSettingRequest {
    pub key: String,
    pub value: String,
}