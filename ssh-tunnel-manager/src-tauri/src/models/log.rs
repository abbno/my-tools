use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLog {
    pub id: String,
    pub config_id: String,
    pub action: LogAction,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogAction {
    Connect,
    Disconnect,
    Reconnect,
    Error,
}

impl ConnectionLog {
    pub fn new(config_id: String, action: LogAction, message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            config_id,
            action,
            message,
            created_at: Utc::now(),
        }
    }
}
