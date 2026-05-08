use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: AuthType,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
    pub tunnel_type: TunnelType,
    pub local_host: String,
    pub local_port: i32,
    pub remote_host: Option<String>,
    pub remote_port: Option<i32>,
    pub auto_reconnect: bool,
    pub reconnect_interval: i32,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub favorite_order: i32,
    #[serde(default)]
    pub auto_start: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Password,
    Key,
}

impl Default for AuthType {
    fn default() -> Self {
        Self::Password
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Local,
    Remote,
    Dynamic,
}

impl Default for TunnelType {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConfigRequest {
    pub name: String,
    pub group_id: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: AuthType,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
    pub tunnel_type: TunnelType,
    pub local_host: String,
    pub local_port: i32,
    pub remote_host: Option<String>,
    pub remote_port: Option<i32>,
    pub auto_reconnect: bool,
    pub reconnect_interval: i32,
    pub is_favorite: Option<bool>,
    pub auto_start: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: AuthType,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase: Option<String>,
    pub tunnel_type: TunnelType,
    pub local_host: String,
    pub local_port: i32,
    pub remote_host: Option<String>,
    pub remote_port: Option<i32>,
    pub auto_reconnect: bool,
    pub reconnect_interval: i32,
    pub is_favorite: Option<bool>,
    pub auto_start: Option<bool>,
}
