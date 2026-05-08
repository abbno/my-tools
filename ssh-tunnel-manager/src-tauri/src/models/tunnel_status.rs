use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub config_id: String,
    pub status: TunnelStatus,
    pub pid: Option<u32>,
    pub error_message: Option<String>,
}

impl TunnelInfo {
    pub fn new(config_id: String) -> Self {
        Self {
            config_id,
            status: TunnelStatus::Stopped,
            pid: None,
            error_message: None,
        }
    }
}
