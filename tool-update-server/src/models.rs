use serde::{Deserialize, Serialize};

/// Software/App information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub app_id: String,
    pub version: String,
    pub release_date: String,
    pub changelog: Vec<ChangelogItem>,
    pub download_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub created_at: String,
}

/// Changelog item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub description: String,
}

/// Create app request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppRequest {
    pub id: String,
    pub name: String,
}

/// Version info for API response (client-compatible format)
#[derive(Debug, Clone, Serialize)]
pub struct VersionInfoResponse {
    pub version: String,
    pub release_date: String,
    pub download_url: String,
    pub changelog: Vec<ChangelogItem>,
}

impl From<Version> for VersionInfoResponse {
    fn from(v: Version) -> Self {
        Self {
            version: v.version,
            release_date: v.release_date,
            download_url: v.download_url,
            changelog: v.changelog,
        }
    }
}