use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(rename = "type")]
    pub auth_type: String, // "none" | "token" | "username-password"
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            auth_type: "none".to_string(),
            token: None,
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub url: String,
    pub auth: AuthConfig,
    pub sync_interval: u64, // seconds
    pub selected_skills: Vec<String>,
    pub last_sync: Option<DateTime<Utc>>,
    pub enabled: bool,
}

impl Repository {
    pub fn new(name: String, url: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            url,
            auth: AuthConfig::default(),
            sync_interval: 3600, // 1 hour default
            selected_skills: Vec::new(),
            last_sync: None,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

impl Agent {
    pub fn default_agents() -> Vec<Self> {
        vec![
            Self { id: "claude-code".into(), name: "Claude Code".into(), path: "~/.claude/skills/".into(), enabled: true },
            Self { id: "codex-cli".into(), name: "Codex CLI".into(), path: "~/.agents/skills/".into(), enabled: false },
            Self { id: "gemini-cli".into(), name: "Gemini CLI".into(), path: "~/.gemini/skills/".into(), enabled: true },
            Self { id: "trae".into(), name: "Trae".into(), path: "~/.trae/skills/".into(), enabled: true },
            Self { id: "opencode".into(), name: "OpenCode".into(), path: "~/.opencode/skills/".into(), enabled: false },
            Self { id: "trae-cn".into(), name: "Trae CN".into(), path: "~/.trae-cn/skills/".into(), enabled: true },
            Self { id: "central".into(), name: "Central".into(), path: "~/.agents/skills/".into(), enabled: true },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_sync_interval: u64,
    pub auto_sync: bool,
    pub check_interval: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_sync_interval: 3600,
            auto_sync: true,
            check_interval: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repositories: Vec<Repository>,
    pub agents: Vec<Agent>,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repositories: Vec::new(),
            agents: Agent::default_agents(),
            settings: Settings::default(),
        }
    }
}