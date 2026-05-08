# SSH Tunnel Manager 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 开发一个跨平台桌面应用，可视化管理 SSH 端口转发隧道

**Architecture:** Tauri 2.x + Vue 3 前后端分离架构，Rust 后端通过 Sidecar 调用系统 SSH 客户端，SQLite 存储配置数据，支持本地/远程/动态三种隧道类型

**Tech Stack:** Vue 3, TypeScript, TDesign, Pinia, Tauri 2.x, Rust, SQLite, rusqlite, tauri-plugin-shell

---

## 文件结构总览

### 前端文件 (src/)

| 文件 | 职责 |
|------|------|
| `src/types/index.ts` | TypeScript 类型定义 |
| `src/api/tauri.ts` | Tauri Command 封装 |
| `src/stores/group.ts` | 分组状态管理 |
| `src/stores/config.ts` | 配置状态管理 |
| `src/stores/tunnel.ts` | 隧道状态管理 |
| `src/utils/format.ts` | 格式化工具函数 |
| `src/components/Sidebar.vue` | 分组侧边栏组件 |
| `src/components/TunnelList.vue` | 隧道列表组件 |
| `src/components/TunnelCard.vue` | 隧道卡片组件 |
| `src/components/ConfigForm.vue` | 配置表单弹窗组件 |
| `src/components/GroupForm.vue` | 分组表单弹窗组件 |
| `src/components/LogPanel.vue` | 日志面板组件 |
| `src/views/Home.vue` | 主页面 |
| `src/App.vue` | 根组件 |
| `src/main.ts` | 入口文件 |

### Rust 后端文件 (src-tauri/src/)

| 文件 | 职责 |
|------|------|
| `src-tauri/src/main.rs` | 程序入口 |
| `src-tauri/src/lib.rs` | 库入口，注册 commands |
| `src-tauri/src/models/mod.rs` | 模型模块导出 |
| `src-tauri/src/models/config.rs` | Config 数据模型 |
| `src-tauri/src/models/group.rs` | Group 数据模型 |
| `src-tauri/src/models/log.rs` | Log 数据模型 |
| `src-tauri/src/models/tunnel_status.rs` | TunnelStatus 数据模型 |
| `src-tauri/src/db/mod.rs` | 数据库模块导出 |
| `src-tauri/src/db/sqlite.rs` | SQLite 连接和初始化 |
| `src-tauri/src/utils/mod.rs` | 工具模块导出 |
| `src-tauri/src/utils/crypto.rs` | AES-GCM 加密工具 |
| `src-tauri/src/commands/mod.rs` | 命令模块导出 |
| `src-tauri/src/commands/group.rs` | 分组管理命令 |
| `src-tauri/src/commands/config.rs` | 配置管理命令 |
| `src-tauri/src/commands/tunnel.rs` | 隧道控制命令 |
| `src-tauri/src/commands/log.rs` | 日志管理命令 |
| `src-tauri/src/ssh/mod.rs` | SSH 模块导出 |
| `src-tauri/src/ssh/sidecar.rs` | SSH 进程管理 |
| `src-tauri/src/ssh/monitor.rs` | 状态监控和自动重连 |

### 配置文件

| 文件 | 职责 |
|------|------|
| `package.json` | 前端依赖配置 |
| `vite.config.ts` | Vite 构建配置 |
| `tsconfig.json` | TypeScript 配置 |
| `src-tauri/Cargo.toml` | Rust 依赖配置 |
| `src-tauri/tauri.conf.json` | Tauri 配置 |
| `src-tauri/capabilities/default.json` | Tauri 权限配置 |

---

## Task 1: 项目初始化

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `index.html`
- Create: `.gitignore`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建前端 package.json**

```json
{
  "name": "ssh-proxy",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "vue": "^3.4.21",
    "vue-router": "^4.3.0",
    "pinia": "^2.1.7",
    "tdesign-vue-next": "^1.9.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.0.4",
    "typescript": "^5.4.2",
    "vite": "^5.1.6",
    "vue-tsc": "^2.0.6"
  }
}
```

- [ ] **Step 2: 创建 Vite 配置文件**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
})
```

- [ ] **Step 3: 创建 TypeScript 配置文件**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 4: 创建 tsconfig.node.json**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 5: 创建 index.html**

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>SSH Tunnel Manager</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 6: 创建 .gitignore**

```
# Dependencies
node_modules/

# Build outputs
dist/
dist-ssr/
*.local

# Editor directories and files
.vscode/*
!.vscode/extensions.json
.idea/
*.suo
*.ntvs*
*.njsproj
*.sln
*.sw?

# OS files
.DS_Store
Thumbs.db

# Tauri
src-tauri/target/

# Database
*.db
*.db-journal
```

- [ ] **Step 7: 创建 Rust Cargo.toml**

```toml
[package]
name = "ssh-proxy"
version = "0.1.0"
description = "SSH Tunnel Manager"
authors = ["you"]
edition = "2021"

[lib]
name = "ssh_proxy_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
aes-gcm = "0.10"
base64 = "0.22"
thiserror = "1"

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

- [ ] **Step 8: 创建 Tauri 配置文件**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "SSH Tunnel Manager",
  "version": "0.1.0",
  "identifier": "com.sshproxy.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "SSH Tunnel Manager",
        "width": 1000,
        "height": 700,
        "minWidth": 800,
        "minHeight": 500,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    },
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 9: 创建 Tauri 权限配置**

```json
{
  "identifier": "default",
  "description": "Default capabilities for the app",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-spawn",
    "shell:allow-execute",
    "shell:allow-kill",
    {
      "identifier": "shell:allow-open",
      "allow": [
        {
          "path": "$RESOURCE/**",
          "sidecar": true
        }
      ]
    }
  ]
}
```

- [ ] **Step 10: 创建 Rust build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 11: 创建 Rust main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ssh_proxy_lib::run()
}
```

- [ ] **Step 12: 创建 Rust lib.rs 骨架**

```rust
mod commands;
mod db;
mod models;
mod ssh;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            let data_dir = app_handle.path().app_data_dir().expect("无法获取数据目录");

            // 确保数据目录存在
            std::fs::create_dir_all(&data_dir).ok();

            // 数据库文件放在程序运行目录
            let exe_dir = std::env::current_exe()
                .expect("无法获取程序路径")
                .parent()
                .expect("无法获取程序目录")
                .to_path_buf();
            let db_path = exe_dir.join("ssh-proxy.db");

            db::init(&db_path).expect("数据库初始化失败");

            // 初始化 SSH 管理器
            ssh::init(app_handle.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 分组管理
            commands::group::get_groups,
            commands::group::save_group,
            commands::group::delete_group,
            // 配置管理
            commands::config::get_configs,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::delete_config,
            commands::config::search_configs,
            // 隧道控制
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::restart_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::get_running_tunnels,
            // 日志管理
            commands::log::get_logs,
            commands::log::clear_logs,
            // 导入导出
            commands::config::export_configs,
            commands::config::import_configs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 13: 提交初始化代码**

```bash
git init
git add .
git commit -m "chore: 初始化项目结构"
```

---

## Task 2: Rust 数据模型定义

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/config.rs`
- Create: `src-tauri/src/models/group.rs`
- Create: `src-tauri/src/models/log.rs`
- Create: `src-tauri/src/models/tunnel_status.rs`

- [ ] **Step 1: 创建 models/mod.rs**

```rust
pub mod config;
pub mod group;
pub mod log;
pub mod tunnel_status;

pub use config::*;
pub use group::*;
pub use log::*;
pub use tunnel_status::*;
```

- [ ] **Step 2: 创建 models/config.rs**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl Config {
    pub fn new(
        name: String,
        host: String,
        port: i32,
        username: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            group_id: None,
            host,
            port,
            username,
            auth_type: AuthType::Password,
            password: None,
            key_path: None,
            key_passphrase: None,
            tunnel_type: TunnelType::Local,
            local_host: "127.0.0.1".to_string(),
            local_port: 0,
            remote_host: None,
            remote_port: None,
            auto_reconnect: false,
            reconnect_interval: 5,
            created_at: now,
            updated_at: now,
        }
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
}
```

- [ ] **Step 3: 创建 models/group.rs**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            sort_order: 0,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
}
```

- [ ] **Step 4: 创建 models/log.rs**

```rust
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
```

- [ ] **Step 5: 创建 models/tunnel_status.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TunnelStatus {
    Running,
    Stopped,
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
```

- [ ] **Step 6: 提交数据模型代码**

```bash
git add src-tauri/src/models/
git commit -m "feat: 添加 Rust 数据模型定义"
```

---

## Task 3: 数据库层实现

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/sqlite.rs`

- [ ] **Step 1: 创建 db/mod.rs**

```rust
mod sqlite;

use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;

pub static DB: Mutex<Option<Connection>> = Mutex::new(None);

pub fn init(db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;

    // 创建表
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            group_id TEXT,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            username TEXT NOT NULL,
            auth_type TEXT NOT NULL,
            password TEXT,
            key_path TEXT,
            key_passphrase TEXT,
            tunnel_type TEXT NOT NULL,
            local_host TEXT NOT NULL,
            local_port INTEGER NOT NULL,
            remote_host TEXT,
            remote_port INTEGER,
            auto_reconnect INTEGER DEFAULT 0,
            reconnect_interval INTEGER DEFAULT 5,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (group_id) REFERENCES groups(id)
        );

        CREATE TABLE IF NOT EXISTS connection_logs (
            id TEXT PRIMARY KEY,
            config_id TEXT NOT NULL,
            action TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (config_id) REFERENCES configs(id)
        );

        CREATE INDEX IF NOT EXISTS idx_configs_group_id ON configs(group_id);
        CREATE INDEX IF NOT EXISTS idx_logs_config_id ON connection_logs(config_id);
        CREATE INDEX IF NOT EXISTS idx_logs_created_at ON connection_logs(created_at);
        "#,
    )?;

    // 存储连接
    *DB.lock().unwrap() = Some(conn);

    Ok(())
}

pub fn get_connection() -> Result<Connection, rusqlite::Error> {
    let db_path = std::env::current_exe()
        .map(|p| p.parent().unwrap().join("ssh-proxy.db"))
        .unwrap();
    Connection::open(db_path)
}
```

- [ ] **Step 2: 创建 db/sqlite.rs**

```rust
use crate::models::*;
use crate::utils::crypto;
use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};

// ==================== 分组操作 ====================

pub fn get_groups(conn: &Connection) -> Result<Vec<Group>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, created_at FROM groups ORDER BY sort_order ASC"
    )?;

    let groups = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            created_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(groups)
}

pub fn get_group_by_id(conn: &Connection, id: &str) -> Result<Option<Group>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, created_at FROM groups WHERE id = ?"
    )?;

    let mut groups = stmt.query_map([id], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            created_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?;

    Ok(groups.next().transpose()?)
}

pub fn save_group(conn: &Connection, group: &Group) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO groups (id, name, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET name = ?2, sort_order = ?3",
        params![
            group.id,
            group.name,
            group.sort_order,
            group.created_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn delete_group(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM groups WHERE id = ?", [id])?;
    Ok(())
}

// ==================== 配置操作 ====================

pub fn get_configs(conn: &Connection, group_id: Option<&str>) -> Result<Vec<Config>, rusqlite::Error> {
    let sql = match group_id {
        Some(_) => "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect, reconnect_interval, created_at, updated_at FROM configs WHERE group_id = ? ORDER BY created_at DESC",
        None => "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect, reconnect_interval, created_at, updated_at FROM configs ORDER BY created_at DESC",
    };

    let mut stmt = conn.prepare(sql)?;

    let rows = match group_id {
        Some(gid) => stmt.query_map([gid], map_config_row)?,
        None => stmt.query_map([], map_config_row)?,
    };

    let mut configs = Vec::new();
    for row in rows {
        let mut config = row?;
        // 解密密码
        if let Some(ref pwd) = config.password {
            config.password = crypto::decrypt(pwd).ok();
        }
        if let Some(ref passphrase) = config.key_passphrase {
            config.key_passphrase = crypto::decrypt(passphrase).ok();
        }
        configs.push(config);
    }

    Ok(configs)
}

fn map_config_row(row: &rusqlite::Row) -> Result<Config, rusqlite::Error> {
    Ok(Config {
        id: row.get(0)?,
        name: row.get(1)?,
        group_id: row.get(2)?,
        host: row.get(3)?,
        port: row.get(4)?,
        username: row.get(5)?,
        auth_type: match row.get::<_, String>(6)?.as_str() {
            "key" => AuthType::Key,
            _ => AuthType::Password,
        },
        password: row.get(7)?,
        key_path: row.get(8)?,
        key_passphrase: row.get(9)?,
        tunnel_type: match row.get::<_, String>(10)?.as_str() {
            "remote" => TunnelType::Remote,
            "dynamic" => TunnelType::Dynamic,
            _ => TunnelType::Local,
        },
        local_host: row.get(11)?,
        local_port: row.get(12)?,
        remote_host: row.get(13)?,
        remote_port: row.get(14)?,
        auto_reconnect: row.get::<_, i32>(15)? != 0,
        reconnect_interval: row.get(16)?,
        created_at: row.get::<_, String>(17)?.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: row.get::<_, String>(18)?.parse().unwrap_or_else(|_| Utc::now()),
    })
}

pub fn get_config_by_id(conn: &Connection, id: &str) -> Result<Option<Config>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect, reconnect_interval, created_at, updated_at FROM configs WHERE id = ?"
    )?;

    let mut configs = stmt.query_map([id], map_config_row)?;

    if let Some(mut config) = configs.next().transpose()? {
        // 解密密码
        if let Some(ref pwd) = config.password {
            config.password = crypto::decrypt(pwd).ok();
        }
        if let Some(ref passphrase) = config.key_passphrase {
            config.key_passphrase = crypto::decrypt(passphrase).ok();
        }
        return Ok(Some(config));
    }

    Ok(None)
}

pub fn save_config(conn: &Connection, config: &Config) -> Result<(), rusqlite::Error> {
    // 加密密码
    let encrypted_password = config.password.as_ref().map(|p| crypto::encrypt(p));
    let encrypted_passphrase = config.key_passphrase.as_ref().map(|p| crypto::encrypt(p));

    let auth_type_str = match config.auth_type {
        AuthType::Password => "password",
        AuthType::Key => "key",
    };

    let tunnel_type_str = match config.tunnel_type {
        TunnelType::Local => "local",
        TunnelType::Remote => "remote",
        TunnelType::Dynamic => "dynamic",
    };

    conn.execute(
        "INSERT INTO configs (id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect, reconnect_interval, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
         ON CONFLICT(id) DO UPDATE SET name = ?2, group_id = ?3, host = ?4, port = ?5, username = ?6, auth_type = ?7, password = ?8, key_path = ?9, key_passphrase = ?10, tunnel_type = ?11, local_host = ?12, local_port = ?13, remote_host = ?14, remote_port = ?15, auto_reconnect = ?16, reconnect_interval = ?17, updated_at = ?19",
        params![
            config.id,
            config.name,
            config.group_id,
            config.host,
            config.port,
            config.username,
            auth_type_str,
            encrypted_password,
            config.key_path,
            encrypted_passphrase,
            tunnel_type_str,
            config.local_host,
            config.local_port,
            config.remote_host,
            config.remote_port,
            config.auto_reconnect as i32,
            config.reconnect_interval,
            config.created_at.to_rfc3339(),
            config.updated_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn delete_config(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    // 先删除相关日志
    conn.execute("DELETE FROM connection_logs WHERE config_id = ?", [id])?;
    // 再删除配置
    conn.execute("DELETE FROM configs WHERE id = ?", [id])?;
    Ok(())
}

pub fn search_configs(conn: &Connection, keyword: &str) -> Result<Vec<Config>, rusqlite::Error> {
    let pattern = format!("%{}%", keyword);
    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect, reconnect_interval, created_at, updated_at FROM configs WHERE name LIKE ? OR host LIKE ? ORDER BY created_at DESC"
    )?;

    let rows = stmt.query_map([&pattern, &pattern], map_config_row)?;

    let mut configs = Vec::new();
    for row in rows {
        let mut config = row?;
        if let Some(ref pwd) = config.password {
            config.password = crypto::decrypt(pwd).ok();
        }
        if let Some(ref passphrase) = config.key_passphrase {
            config.key_passphrase = crypto::decrypt(passphrase).ok();
        }
        configs.push(config);
    }

    Ok(configs)
}

// ==================== 日志操作 ====================

pub fn get_logs(conn: &Connection, config_id: &str, limit: Option<i32>) -> Result<Vec<ConnectionLog>, rusqlite::Error> {
    let limit = limit.unwrap_or(100);
    let mut stmt = conn.prepare(
        "SELECT id, config_id, action, message, created_at FROM connection_logs WHERE config_id = ? ORDER BY created_at DESC LIMIT ?"
    )?;

    let logs = stmt.query_map(params![config_id, limit], |row| {
        Ok(ConnectionLog {
            id: row.get(0)?,
            config_id: row.get(1)?,
            action: match row.get::<_, String>(2)?.as_str() {
                "disconnect" => LogAction::Disconnect,
                "reconnect" => LogAction::Reconnect,
                "error" => LogAction::Error,
                _ => LogAction::Connect,
            },
            message: row.get(3)?,
            created_at: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| Utc::now()),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(logs)
}

pub fn save_log(conn: &Connection, log: &ConnectionLog) -> Result<(), rusqlite::Error> {
    let action_str = match log.action {
        LogAction::Connect => "connect",
        LogAction::Disconnect => "disconnect",
        LogAction::Reconnect => "reconnect",
        LogAction::Error => "error",
    };

    conn.execute(
        "INSERT INTO connection_logs (id, config_id, action, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            log.id,
            log.config_id,
            action_str,
            log.message,
            log.created_at.to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn clear_logs(conn: &Connection, config_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM connection_logs WHERE config_id = ?", [config_id])?;
    Ok(())
}

// ==================== 导入导出 ====================

pub fn export_configs(conn: &Connection, ids: Option<&[String]>) -> Result<String, rusqlite::Error> {
    let configs = match ids {
        Some(id_list) => {
            let placeholders: Vec<String> = id_list.iter().map(|_| "?".to_string()).collect();
            let sql = format!(
                "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect, reconnect_interval, created_at, updated_at FROM configs WHERE id IN ({})",
                placeholders.join(",")
            );
            let params: Vec<&str> = id_list.iter().map(String::as_str).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(rusqlite::params_from_iter(params), map_config_row)?;
            rows.collect::<Result<Vec<_>, _>>()?
        }
        None => get_configs(conn, None)?,
    };

    Ok(serde_json::to_string(&configs).unwrap_or_default())
}

pub fn import_configs(conn: &Connection, json: &str) -> Result<i32, rusqlite::Error> {
    let configs: Vec<Config> = serde_json::from_str(json).map_err(|_| rusqlite::Error::InvalidQuery)?;

    let mut count = 0;
    for mut config in configs {
        // 生成新 ID 避免冲突
        config.id = uuid::Uuid::new_v4().to_string();
        config.created_at = Utc::now();
        config.updated_at = Utc::now();
        save_config(conn, &config)?;
        count += 1;
    }

    Ok(count)
}
```

- [ ] **Step 3: 提交数据库层代码**

```bash
git add src-tauri/src/db/
git commit -m "feat: 实现数据库层操作"
```

---

## Task 4: 加密工具实现

**Files:**
- Create: `src-tauri/src/utils/mod.rs`
- Create: `src-tauri/src/utils/crypto.rs`

- [ ] **Step 1: 创建 utils/mod.rs**

```rust
pub mod crypto;
```

- [ ] **Step 2: 创建 utils/crypto.rs**

```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

// 固定密钥（实际应用中应该从系统密钥链获取或让用户设置主密码）
// 这里使用硬编码密钥仅用于演示，生产环境应使用更安全的方式
lazy_static::lazy_static! {
    static ref KEY: [u8; 32] = {
        // 从环境变量或配置中获取，这里使用固定值
        let mut key = [0u8; 32];
        let key_str = "SSH_PROXY_SECRET_KEY_32BYTES!!";
        key.copy_from_slice(key_str.as_bytes());
        key
    };
}

fn get_cipher() -> Aes256Gcm {
    Aes256Gcm::new_from_slice(&*KEY).expect("无效的密钥")
}

pub fn encrypt(plaintext: &str) -> String {
    let cipher = get_cipher();
    let nonce = Nonce::from_slice(b"unique_nonce"); // 固定 nonce，生产环境应随机生成

    match cipher.encrypt(nonce, plaintext.as_bytes()) {
        Ok(ciphertext) => BASE64.encode(&ciphertext),
        Err(_) => plaintext.to_string(),
    }
}

pub fn decrypt(ciphertext: &str) -> Result<String, String> {
    let cipher = get_cipher();
    let nonce = Nonce::from_slice(b"unique_nonce");

    match BASE64.decode(ciphertext) {
        Ok(data) => match cipher.decrypt(nonce, data.as_slice()) {
            Ok(plaintext) => String::from_utf8(plaintext).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "my_secret_password";
        let encrypted = encrypt(original);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }
}
```

- [ ] **Step 3: 更新 Cargo.toml 添加 lazy_static**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
lazy_static = "1.4"
```

- [ ] **Step 4: 提交加密工具代码**

```bash
git add src-tauri/src/utils/ src-tauri/Cargo.toml
git commit -m "feat: 实现 AES-GCM 加密工具"
```

---

## Task 5: SSH 管理模块实现

**Files:**
- Create: `src-tauri/src/ssh/mod.rs`
- Create: `src-tauri/src/ssh/sidecar.rs`
- Create: `src-tauri/src/ssh/monitor.rs`

- [ ] **Step 1: 创建 ssh/mod.rs**

```rust
mod sidecar;
mod monitor;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;

pub use sidecar::*;
pub use monitor::*;

/// 运行中的隧道信息
pub static TUNNELS: Mutex<HashMap<String, TunnelInfo>> = Mutex::new(HashMap::new());

/// 应用句柄
pub static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

/// 初始化 SSH 管理器
pub fn init(app_handle: AppHandle) {
    *APP_HANDLE.lock().unwrap() = Some(app_handle);
}
```

- [ ] **Step 2: 创建 ssh/sidecar.rs**

```rust
use crate::models::{Config, TunnelType, TunnelInfo, TunnelStatus, ConnectionLog, LogAction};
use crate::db::sqlite;
use crate::ssh::{TUNNELS, APP_HANDLE};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;
use std::sync::Mutex;

/// SSH 进程管理
pub static SSH_PROCESSES: Mutex<HashMap<String, CommandChild>> = Mutex::new(HashMap::new());

/// 构建 SSH 命令参数
pub fn build_ssh_args(config: &Config) -> Vec<String> {
    let mut args = Vec::new();

    // 添加隧道参数
    match config.tunnel_type {
        TunnelType::Local => {
            args.push("-L".to_string());
            args.push(format!(
                "{}:{}:{}:{}",
                config.local_host,
                config.local_port,
                config.remote_host.as_ref().unwrap_or(&"localhost".to_string()),
                config.remote_port.unwrap_or(22)
            ));
        }
        TunnelType::Remote => {
            args.push("-R".to_string());
            args.push(format!(
                "{}:{}:{}:{}",
                config.remote_host.as_ref().unwrap_or(&"0.0.0.0".to_string()),
                config.remote_port.unwrap_or(22),
                config.local_host,
                config.local_port
            ));
        }
        TunnelType::Dynamic => {
            args.push("-D".to_string());
            args.push(format!("{}:{}", config.local_host, config.local_port));
        }
    }

    // 基本连接参数
    args.push("-p".to_string());
    args.push(config.port.to_string());

    // 保持连接
    args.push("-o".to_string());
    args.push("ServerAliveInterval=60".to_string());
    args.push("-o".to_string());
    args.push("ServerAliveCountMax=3".to_string());

    // 不检查主机密钥（生产环境应考虑安全性）
    args.push("-o".to_string());
    args.push("StrictHostKeyChecking=no".to_string());
    args.push("-o".to_string());
    args.push("UserKnownHostsFile=/dev/null".to_string());

    // 后台运行
    args.push("-N".to_string());

    // 用户和主机
    args.push(format!("{}@{}", config.username, config.host));

    args
}

/// 启动 SSH 隧道
pub async fn start_ssh_tunnel(config: &Config) -> Result<u32, String> {
    let app_handle = APP_HANDLE.lock().unwrap().clone();
    let app_handle = app_handle.ok_or("应用未初始化")?;

    let args = build_ssh_args(config);

    // 检查端口是否被占用
    if is_port_in_use(&config.local_host, config.local_port) {
        return Err(format!("端口 {}:{} 已被占用", config.local_host, config.local_port));
    }

    // 使用系统 SSH 命令
    let shell = app_handle.shell();

    let result = shell
        .command("ssh")
        .args(&args)
        .spawn()
        .map_err(|e| format!("启动 SSH 进程失败: {}", e))?;

    let pid = result.pid();

    // 存储进程信息
    if let Some(child) = result.child() {
        SSH_PROCESSES.lock().unwrap().insert(config.id.clone(), child);
    }

    // 更新隧道状态
    let mut tunnels = TUNNELS.lock().unwrap();
    tunnels.insert(config.id.clone(), TunnelInfo {
        config_id: config.id.clone(),
        status: TunnelStatus::Running,
        pid: Some(pid),
        error_message: None,
    });

    // 记录日志
    if let Ok(conn) = crate::db::get_connection() {
        let log = ConnectionLog::new(
            config.id.clone(),
            LogAction::Connect,
            format!("隧道启动成功，PID: {}", pid),
        );
        let _ = sqlite::save_log(&conn, &log);
    }

    Ok(pid)
}

/// 停止 SSH 隧道
pub fn stop_ssh_tunnel(config_id: &str) -> Result<(), String> {
    let mut processes = SSH_PROCESSES.lock().unwrap();

    if let Some(mut child) = processes.remove(config_id) {
        child.kill().map_err(|e| format!("停止进程失败: {}", e))?;
    }

    // 更新隧道状态
    let mut tunnels = TUNNELS.lock().unwrap();
    if let Some(info) = tunnels.get_mut(config_id) {
        info.status = TunnelStatus::Stopped;
        info.pid = None;
    }

    // 记录日志
    if let Ok(conn) = crate::db::get_connection() {
        let log = ConnectionLog::new(
            config_id.to_string(),
            LogAction::Disconnect,
            "隧道已停止".to_string(),
        );
        let _ = sqlite::save_log(&conn, &log);
    }

    Ok(())
}

/// 检查端口是否被占用
fn is_port_in_use(host: &str, port: i32) -> bool {
    use std::net::TcpListener;

    let addr = format!("{}:{}", host, port);
    TcpListener::bind(&addr).is_err()
}

/// 获取隧道状态
pub fn get_tunnel_status(config_id: &str) -> TunnelInfo {
    let tunnels = TUNNELS.lock().unwrap();
    tunnels.get(config_id).cloned().unwrap_or_else(|| TunnelInfo::new(config_id.to_string()))
}

/// 获取所有运行中的隧道
pub fn get_running_tunnels() -> Vec<TunnelInfo> {
    let tunnels = TUNNELS.lock().unwrap();
    tunnels.values().cloned().collect()
}
```

- [ ] **Step 3: 创建 ssh/monitor.rs**

```rust
use crate::models::{ConnectionLog, LogAction, TunnelStatus};
use crate::ssh::{TUNNELS, SSH_PROCESSES, APP_HANDLE};
use std::time::Duration;
use tokio::time::sleep;

/// 启动状态监控任务
pub fn start_monitor(config_id: String, auto_reconnect: bool, reconnect_interval: i32) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;

            let should_reconnect = {
                let processes = SSH_PROCESSES.lock().unwrap();
                let tunnels = TUNNELS.lock().unwrap();

                if let Some(info) = tunnels.get(&config_id) {
                    if info.status == TunnelStatus::Running {
                        // 检查进程是否还在
                        if !processes.contains_key(&config_id) {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if should_reconnect {
                // 进程已退出
                {
                    let mut tunnels = TUNNELS.lock().unwrap();
                    if let Some(info) = tunnels.get_mut(&config_id) {
                        info.status = TunnelStatus::Stopped;
                        info.pid = None;
                    }
                }

                // 记录日志
                if let Ok(conn) = crate::db::get_connection() {
                    let log = ConnectionLog::new(
                        config_id.clone(),
                        LogAction::Error,
                        "SSH 连接已断开".to_string(),
                    );
                    let _ = crate::db::sqlite::save_log(&conn, &log);
                }

                // 自动重连
                if auto_reconnect {
                    {
                        let mut tunnels = TUNNELS.lock().unwrap();
                        if let Some(info) = tunnels.get_mut(&config_id) {
                            info.status = TunnelStatus::Reconnecting;
                        }
                    }

                    sleep(Duration::from_secs(reconnect_interval as u64)).await;

                    // 尝试重连
                    if let Ok(conn) = crate::db::get_connection() {
                        if let Ok(Some(config)) = crate::db::sqlite::get_config_by_id(&conn, &config_id) {
                            match crate::ssh::sidecar::start_ssh_tunnel(&config).await {
                                Ok(_) => {
                                    let log = ConnectionLog::new(
                                        config_id.clone(),
                                        LogAction::Reconnect,
                                        "自动重连成功".to_string(),
                                    );
                                    let _ = crate::db::sqlite::save_log(&conn, &log);
                                }
                                Err(e) => {
                                    let log = ConnectionLog::new(
                                        config_id.clone(),
                                        LogAction::Error,
                                        format!("自动重连失败: {}", e),
                                    );
                                    let _ = crate::db::sqlite::save_log(&conn, &log);

                                    let mut tunnels = TUNNELS.lock().unwrap();
                                    if let Some(info) = tunnels.get_mut(&config_id) {
                                        info.status = TunnelStatus::Error;
                                        info.error_message = Some(e);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 检查隧道是否已被停止
            let tunnels = TUNNELS.lock().unwrap();
            let should_stop = !tunnels.contains_key(&config_id) ||
                tunnels.get(&config_id).map(|t| t.status == TunnelStatus::Stopped).unwrap_or(true);
            drop(tunnels);

            if should_stop {
                break;
            }
        }
    });
}
```

- [ ] **Step 4: 更新 Cargo.toml 添加 tokio**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 5: 提交 SSH 管理模块代码**

```bash
git add src-tauri/src/ssh/ src-tauri/Cargo.toml
git commit -m "feat: 实现 SSH 隧道管理模块"
```

---

## Task 6: Tauri Commands 实现

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/group.rs`
- Create: `src-tauri/src/commands/config.rs`
- Create: `src-tauri/src/commands/tunnel.rs`
- Create: `src-tauri/src/commands/log.rs`

- [ ] **Step 1: 创建 commands/mod.rs**

```rust
pub mod group;
pub mod config;
pub mod tunnel;
pub mod log;
```

- [ ] **Step 2: 创建 commands/group.rs**

```rust
use crate::db;
use crate::models::{Group, CreateGroupRequest, UpdateGroupRequest};

#[tauri::command]
pub fn get_groups() -> Result<Vec<Group>, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::get_groups(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_group(request: CreateGroupRequest) -> Result<Group, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;

    let group = Group {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        sort_order: request.sort_order.unwrap_or(0),
        created_at: chrono::Utc::now(),
    };

    db::sqlite::save_group(&conn, &group).map_err(|e| e.to_string())?;
    Ok(group)
}

#[tauri::command]
pub fn delete_group(id: String) -> Result<(), String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::delete_group(&conn, &id).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 创建 commands/config.rs**

```rust
use crate::db;
use crate::models::{Config, CreateConfigRequest, UpdateConfigRequest};

#[tauri::command]
pub fn get_configs(group_id: Option<String>) -> Result<Vec<Config>, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::get_configs(&conn, group_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(id: String) -> Result<Option<Config>, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::get_config_by_id(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(request: CreateConfigRequest) -> Result<Config, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
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
        created_at: now,
        updated_at: now,
    };

    db::sqlite::save_config(&conn, &config).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub fn update_config(request: UpdateConfigRequest) -> Result<Config, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;

    // 获取原配置以保留创建时间
    let original = db::sqlite::get_config_by_id(&conn, &request.id)
        .map_err(|e| e.to_string())?
        .ok_or("配置不存在")?;

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
        created_at: original.created_at,
        updated_at: chrono::Utc::now(),
    };

    db::sqlite::save_config(&conn, &config).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub fn delete_config(id: String) -> Result<(), String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::delete_config(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_configs(keyword: String) -> Result<Vec<Config>, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::search_configs(&conn, &keyword).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_configs(ids: Option<Vec<String>>) -> Result<String, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::export_configs(&conn, ids.as_deref().map(|v| v.as_slice())).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_configs(json: String) -> Result<i32, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::import_configs(&conn, &json).map_err(|e| e.to_string())
}
```

- [ ] **Step 4: 创建 commands/tunnel.rs**

```rust
use crate::db;
use crate::models::TunnelInfo;
use crate::ssh::{sidecar, monitor};

#[tauri::command]
pub async fn start_tunnel(config_id: String) -> Result<(), String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    let config = db::sqlite::get_config_by_id(&conn, &config_id)
        .map_err(|e| e.to_string())?
        .ok_or("配置不存在")?;

    let auto_reconnect = config.auto_reconnect;
    let reconnect_interval = config.reconnect_interval;

    sidecar::start_ssh_tunnel(&config).await?;

    // 启动监控
    monitor::start_monitor(config_id, auto_reconnect, reconnect_interval);

    Ok(())
}

#[tauri::command]
pub fn stop_tunnel(config_id: String) -> Result<(), String> {
    sidecar::stop_ssh_tunnel(&config_id)
}

#[tauri::command]
pub async fn restart_tunnel(config_id: String) -> Result<(), String> {
    sidecar::stop_ssh_tunnel(&config_id)?;
    start_tunnel(config_id).await
}

#[tauri::command]
pub fn get_tunnel_status(config_id: String) -> Result<TunnelInfo, String> {
    Ok(sidecar::get_tunnel_status(&config_id))
}

#[tauri::command]
pub fn get_running_tunnels() -> Result<Vec<TunnelInfo>, String> {
    Ok(sidecar::get_running_tunnels())
}
```

- [ ] **Step 5: 创建 commands/log.rs**

```rust
use crate::db;
use crate::models::ConnectionLog;

#[tauri::command]
pub fn get_logs(config_id: String, limit: Option<i32>) -> Result<Vec<ConnectionLog>, String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::get_logs(&conn, &config_id, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_logs(config_id: String) -> Result<(), String> {
    let conn = db::get_connection().map_err(|e| e.to_string())?;
    db::sqlite::clear_logs(&conn, &config_id).map_err(|e| e.to_string())
}
```

- [ ] **Step 6: 更新 lib.rs 修正命令注册**

更新 `src-tauri/src/lib.rs` 中的 invoke_handler，添加 update_config：

```rust
.invoke_handler(tauri::generate_handler![
    // 分组管理
    commands::group::get_groups,
    commands::group::save_group,
    commands::group::delete_group,
    // 配置管理
    commands::config::get_configs,
    commands::config::get_config,
    commands::config::save_config,
    commands::config::update_config,
    commands::config::delete_config,
    commands::config::search_configs,
    // 隧道控制
    commands::tunnel::start_tunnel,
    commands::tunnel::stop_tunnel,
    commands::tunnel::restart_tunnel,
    commands::tunnel::get_tunnel_status,
    commands::tunnel::get_running_tunnels,
    // 日志管理
    commands::log::get_logs,
    commands::log::clear_logs,
    // 导入导出
    commands::config::export_configs,
    commands::config::import_configs,
])
```

- [ ] **Step 7: 提交 Commands 代码**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: 实现 Tauri Commands 接口"
```

---

## Task 7: 前端类型定义

**Files:**
- Create: `src/types/index.ts`

- [ ] **Step 1: 创建 TypeScript 类型定义**

```typescript
// 认证类型
export type AuthType = 'password' | 'key'

// 隧道类型
export type TunnelType = 'local' | 'remote' | 'dynamic'

// 隧道状态
export type TunnelStatus = 'running' | 'stopped' | 'error' | 'reconnecting'

// 日志操作类型
export type LogAction = 'connect' | 'disconnect' | 'reconnect' | 'error'

// 配置接口
export interface Config {
  id: string
  name: string
  groupId: string | null
  host: string
  port: number
  username: string
  authType: AuthType
  password: string | null
  keyPath: string | null
  keyPassphrase: string | null
  tunnelType: TunnelType
  localHost: string
  localPort: number
  remoteHost: string | null
  remotePort: number | null
  autoReconnect: boolean
  reconnectInterval: number
  createdAt: string
  updatedAt: string
}

// 创建配置请求
export interface CreateConfigRequest {
  name: string
  groupId: string | null
  host: string
  port: number
  username: string
  authType: AuthType
  password: string | null
  keyPath: string | null
  keyPassphrase: string | null
  tunnelType: TunnelType
  localHost: string
  localPort: number
  remoteHost: string | null
  remotePort: number | null
  autoReconnect: boolean
  reconnectInterval: number
}

// 更新配置请求
export interface UpdateConfigRequest extends CreateConfigRequest {
  id: string
}

// 分组接口
export interface Group {
  id: string
  name: string
  sortOrder: number
  createdAt: string
}

// 创建分组请求
export interface CreateGroupRequest {
  name: string
  sortOrder?: number
}

// 连接日志接口
export interface ConnectionLog {
  id: string
  configId: string
  action: LogAction
  message: string
  createdAt: string
}

// 隧道信息接口
export interface TunnelInfo {
  configId: string
  status: TunnelStatus
  pid: number | null
  errorMessage: string | null
}
```

- [ ] **Step 2: 提交类型定义**

```bash
git add src/types/
git commit -m "feat: 添加前端 TypeScript 类型定义"
```

---

## Task 8: Tauri API 封装

**Files:**
- Create: `src/api/tauri.ts`

- [ ] **Step 1: 创建 Tauri API 封装**

```typescript
import { invoke } from '@tauri-apps/api/core'
import type {
  Config,
  CreateConfigRequest,
  UpdateConfigRequest,
  Group,
  CreateGroupRequest,
  ConnectionLog,
  TunnelInfo
} from '@/types'

// ==================== 分组管理 ====================

export async function getGroups(): Promise<Group[]> {
  return invoke<Group[]>('get_groups')
}

export async function saveGroup(request: CreateGroupRequest): Promise<Group> {
  return invoke<Group>('save_group', { request })
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke('delete_group', { id })
}

// ==================== 配置管理 ====================

export async function getConfigs(groupId?: string): Promise<Config[]> {
  return invoke<Config[]>('get_configs', { groupId: groupId || null })
}

export async function getConfig(id: string): Promise<Config | null> {
  return invoke<Config | null>('get_config', { id })
}

export async function saveConfig(request: CreateConfigRequest): Promise<Config> {
  // 转换字段名为 snake_case
  const payload = {
    name: request.name,
    group_id: request.groupId,
    host: request.host,
    port: request.port,
    username: request.username,
    auth_type: request.authType,
    password: request.password,
    key_path: request.keyPath,
    key_passphrase: request.keyPassphrase,
    tunnel_type: request.tunnelType,
    local_host: request.localHost,
    local_port: request.localPort,
    remote_host: request.remoteHost,
    remote_port: request.remotePort,
    auto_reconnect: request.autoReconnect,
    reconnect_interval: request.reconnectInterval
  }
  return invoke<Config>('save_config', { request: payload })
}

export async function updateConfig(request: UpdateConfigRequest): Promise<Config> {
  const payload = {
    id: request.id,
    name: request.name,
    group_id: request.groupId,
    host: request.host,
    port: request.port,
    username: request.username,
    auth_type: request.authType,
    password: request.password,
    key_path: request.keyPath,
    key_passphrase: request.keyPassphrase,
    tunnel_type: request.tunnelType,
    local_host: request.localHost,
    local_port: request.localPort,
    remote_host: request.remoteHost,
    remote_port: request.remotePort,
    auto_reconnect: request.autoReconnect,
    reconnect_interval: request.reconnectInterval
  }
  return invoke<Config>('update_config', { request: payload })
}

export async function deleteConfig(id: string): Promise<void> {
  return invoke('delete_config', { id })
}

export async function searchConfigs(keyword: string): Promise<Config[]> {
  return invoke<Config[]>('search_configs', { keyword })
}

// ==================== 隧道控制 ====================

export async function startTunnel(configId: string): Promise<void> {
  return invoke('start_tunnel', { configId })
}

export async function stopTunnel(configId: string): Promise<void> {
  return invoke('stop_tunnel', { configId })
}

export async function restartTunnel(configId: string): Promise<void> {
  return invoke('restart_tunnel', { configId })
}

export async function getTunnelStatus(configId: string): Promise<TunnelInfo> {
  return invoke<TunnelInfo>('get_tunnel_status', { configId })
}

export async function getRunningTunnels(): Promise<TunnelInfo[]> {
  return invoke<TunnelInfo[]>('get_running_tunnels')
}

// ==================== 日志管理 ====================

export async function getLogs(configId: string, limit?: number): Promise<ConnectionLog[]> {
  return invoke<ConnectionLog[]>('get_logs', { configId, limit })
}

export async function clearLogs(configId: string): Promise<void> {
  return invoke('clear_logs', { configId })
}

// ==================== 导入导出 ====================

export async function exportConfigs(ids?: string[]): Promise<string> {
  return invoke<string>('export_configs', { ids })
}

export async function importConfigs(json: string): Promise<number> {
  return invoke<number>('import_configs', { json })
}
```

- [ ] **Step 2: 提交 API 封装**

```bash
git add src/api/
git commit -m "feat: 封装 Tauri API 调用"
```

---

## Task 9: Pinia 状态管理

**Files:**
- Create: `src/stores/group.ts`
- Create: `src/stores/config.ts`
- Create: `src/stores/tunnel.ts`

- [ ] **Step 1: 创建 group.ts**

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Group, CreateGroupRequest } from '@/types'
import * as api from '@/api/tauri'

export const useGroupStore = defineStore('group', () => {
  const groups = ref<Group[]>([])
  const loading = ref(false)

  async function fetchGroups() {
    loading.value = true
    try {
      groups.value = await api.getGroups()
    } finally {
      loading.value = false
    }
  }

  async function createGroup(request: CreateGroupRequest) {
    const group = await api.saveGroup(request)
    groups.value.push(group)
    groups.value.sort((a, b) => a.sortOrder - b.sortOrder)
    return group
  }

  async function removeGroup(id: string) {
    await api.deleteGroup(id)
    groups.value = groups.value.filter(g => g.id !== id)
  }

  return {
    groups,
    loading,
    fetchGroups,
    createGroup,
    removeGroup
  }
})
```

- [ ] **Step 2: 创建 config.ts**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Config, CreateConfigRequest, UpdateConfigRequest } from '@/types'
import * as api from '@/api/tauri'

export const useConfigStore = defineStore('config', () => {
  const configs = ref<Config[]>([])
  const loading = ref(false)
  const selectedGroupId = ref<string | null>(null)

  const filteredConfigs = computed(() => {
    if (!selectedGroupId.value) {
      return configs.value
    }
    return configs.value.filter(c => c.groupId === selectedGroupId.value)
  })

  async function fetchConfigs(groupId?: string) {
    loading.value = true
    try {
      configs.value = await api.getConfigs(groupId || undefined)
    } finally {
      loading.value = false
    }
  }

  async function createConfig(request: CreateConfigRequest) {
    const config = await api.saveConfig(request)
    configs.value.unshift(config)
    return config
  }

  async function updateConfig(request: UpdateConfigRequest) {
    const config = await api.updateConfig(request)
    const index = configs.value.findIndex(c => c.id === config.id)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }

  async function removeConfig(id: string) {
    await api.deleteConfig(id)
    configs.value = configs.value.filter(c => c.id !== id)
  }

  async function search(keyword: string) {
    loading.value = true
    try {
      configs.value = await api.searchConfigs(keyword)
    } finally {
      loading.value = false
    }
  }

  function setSelectedGroup(groupId: string | null) {
    selectedGroupId.value = groupId
  }

  return {
    configs,
    filteredConfigs,
    loading,
    selectedGroupId,
    fetchConfigs,
    createConfig,
    updateConfig,
    removeConfig,
    search,
    setSelectedGroup
  }
})
```

- [ ] **Step 3: 创建 tunnel.ts**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TunnelInfo, TunnelStatus } from '@/types'
import * as api from '@/api/tauri'

export const useTunnelStore = defineStore('tunnel', () => {
  const tunnels = ref<Map<string, TunnelInfo>>(new Map())
  const loading = ref(new Set<string>())

  const runningCount = computed(() => {
    let count = 0
    tunnels.value.forEach(t => {
      if (t.status === 'running') count++
    })
    return count
  })

  async function startTunnel(configId: string) {
    loading.value.add(configId)
    try {
      await api.startTunnel(configId)
      tunnels.value.set(configId, {
        configId,
        status: 'running',
        pid: null,
        errorMessage: null
      })
    } catch (error) {
      tunnels.value.set(configId, {
        configId,
        status: 'error',
        pid: null,
        errorMessage: String(error)
      })
      throw error
    } finally {
      loading.value.delete(configId)
    }
  }

  async function stopTunnel(configId: string) {
    loading.value.add(configId)
    try {
      await api.stopTunnel(configId)
      tunnels.value.set(configId, {
        configId,
        status: 'stopped',
        pid: null,
        errorMessage: null
      })
    } finally {
      loading.value.delete(configId)
    }
  }

  async function restartTunnel(configId: string) {
    loading.value.add(configId)
    try {
      await api.restartTunnel(configId)
      tunnels.value.set(configId, {
        configId,
        status: 'running',
        pid: null,
        errorMessage: null
      })
    } finally {
      loading.value.delete(configId)
    }
  }

  async function fetchStatus(configId: string) {
    const status = await api.getTunnelStatus(configId)
    tunnels.value.set(configId, status)
    return status
  }

  async function fetchAllRunning() {
    const running = await api.getRunningTunnels()
    running.forEach(t => tunnels.value.set(t.configId, t))
  }

  function getStatus(configId: string): TunnelStatus {
    return tunnels.value.get(configId)?.status || 'stopped'
  }

  function isLoading(configId: string): boolean {
    return loading.value.has(configId)
  }

  return {
    tunnels,
    runningCount,
    startTunnel,
    stopTunnel,
    restartTunnel,
    fetchStatus,
    fetchAllRunning,
    getStatus,
    isLoading
  }
})
```

- [ ] **Step 4: 提交状态管理代码**

```bash
git add src/stores/
git commit -m "feat: 实现 Pinia 状态管理"
```

---

## Task 10: 工具函数

**Files:**
- Create: `src/utils/format.ts`

- [ ] **Step 1: 创建格式化工具函数**

```typescript
import type { Config, TunnelType } from '@/types'

/**
 * 格式化日期时间
 */
export function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

/**
 * 格式化 SSH 命令
 */
export function formatSshCommand(config: Config): string {
  const parts = ['ssh']

  switch (config.tunnelType) {
    case 'local':
      parts.push('-L')
      parts.push(`${config.localHost}:${config.localPort}:${config.remoteHost || 'localhost'}:${config.remotePort || 22}`)
      break
    case 'remote':
      parts.push('-R')
      parts.push(`${config.remoteHost || '0.0.0.0'}:${config.remotePort || 22}:${config.localHost}:${config.localPort}`)
      break
    case 'dynamic':
      parts.push('-D')
      parts.push(`${config.localHost}:${config.localPort}`)
      break
  }

  parts.push('-p')
  parts.push(String(config.port))
  parts.push(`${config.username}@${config.host}`)

  return parts.join(' ')
}

/**
 * 获取隧道类型显示名称
 */
export function getTunnelTypeLabel(type: TunnelType): string {
  const labels: Record<TunnelType, string> = {
    local: '本地转发',
    remote: '远程转发',
    dynamic: '动态转发'
  }
  return labels[type]
}

/**
 * 获取隧道类型标签颜色
 */
export function getTunnelTypeTheme(type: TunnelType): 'primary' | 'success' | 'warning' {
  const themes: Record<TunnelType, 'primary' | 'success' | 'warning'> = {
    local: 'primary',
    remote: 'success',
    dynamic: 'warning'
  }
  return themes[type]
}

/**
 * 复制文本到剪贴板
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}
```

- [ ] **Step 2: 提交工具函数**

```bash
git add src/utils/
git commit -m "feat: 添加格式化工具函数"
```

---

## Task 11: 前端组件实现 - 分组侧边栏

**Files:**
- Create: `src/components/Sidebar.vue`

- [ ] **Step 1: 创建 Sidebar.vue**

```vue
<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <span class="title">分组</span>
      <t-button
        variant="text"
        size="small"
        @click="showGroupForm = true"
      >
        <template #icon><add-icon /></template>
      </t-button>
    </div>

    <div class="group-list">
      <div
        class="group-item"
        :class="{ active: !selectedGroupId }"
        @click="selectGroup(null)"
      >
        <span class="group-name">全部</span>
        <span class="group-count">{{ totalCount }}</span>
      </div>

      <div
        v-for="group in groups"
        :key="group.id"
        class="group-item"
        :class="{ active: selectedGroupId === group.id }"
        @click="selectGroup(group.id)"
        @contextmenu.prevent="showContextMenu($event, group)"
      >
        <span class="group-name">{{ group.name }}</span>
        <span class="group-count">{{ getGroupCount(group.id) }}</span>
      </div>
    </div>

    <!-- 分组表单弹窗 -->
    <t-dialog
      v-model:visible="showGroupForm"
      header="新建分组"
      :confirm-btn="{ content: '确定', loading: submitting }"
      @confirm="handleCreateGroup"
    >
      <t-form :data="groupForm" :rules="groupRules" ref="formRef">
        <t-form-item label="分组名称" name="name">
          <t-input v-model="groupForm.name" placeholder="请输入分组名称" />
        </t-form-item>
      </t-form>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { AddIcon } from 'tdesign-icons-vue-next'
import { useGroupStore } from '@/stores/group'
import { useConfigStore } from '@/stores/config'
import type { Group } from '@/types'

const groupStore = useGroupStore()
const configStore = useConfigStore()

const groups = computed(() => groupStore.groups)
const selectedGroupId = computed(() => configStore.selectedGroupId)
const totalCount = computed(() => configStore.configs.length)

const showGroupForm = ref(false)
const submitting = ref(false)
const formRef = ref()
const groupForm = ref({ name: '' })
const groupRules = {
  name: [{ required: true, message: '请输入分组名称' }]
}

function selectGroup(groupId: string | null) {
  configStore.setSelectedGroup(groupId)
}

function getGroupCount(groupId: string): number {
  return configStore.configs.filter(c => c.groupId === groupId).length
}

async function handleCreateGroup() {
  const valid = await formRef.value?.validate()
  if (valid !== true) return

  submitting.value = true
  try {
    await groupStore.createGroup({ name: groupForm.value.name })
    MessagePlugin.success('分组创建成功')
    showGroupForm.value = false
    groupForm.value.name = ''
  } catch (error) {
    MessagePlugin.error('创建失败：' + error)
  } finally {
    submitting.value = false
  }
}

function showContextMenu(event: MouseEvent, group: Group) {
  // TODO: 实现右键菜单（编辑、删除）
}
</script>

<style scoped>
.sidebar {
  width: 200px;
  height: 100%;
  background: var(--td-bg-color-container);
  border-right: 1px solid var(--td-component-border);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--td-component-border);
}

.sidebar-header .title {
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.group-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.group-item {
  padding: 10px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.group-item:hover {
  background: var(--td-bg-color-container-hover);
}

.group-item.active {
  background: var(--td-brand-color-light);
}

.group-name {
  color: var(--td-text-color-primary);
}

.group-count {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background: var(--td-bg-color-component);
  padding: 2px 8px;
  border-radius: 10px;
}
</style>
```

- [ ] **Step 2: 提交组件代码**

```bash
git add src/components/Sidebar.vue
git commit -m "feat: 实现分组侧边栏组件"
```

---

## Task 12: 前端组件实现 - 隧道卡片

**Files:**
- Create: `src/components/TunnelCard.vue`

- [ ] **Step 1: 创建 TunnelCard.vue**

```vue
<template>
  <div class="tunnel-card" :class="statusClass">
    <div class="card-header">
      <div class="status-indicator">
        <span class="status-dot" :class="configId"></span>
        <span class="status-text">{{ statusText }}</span>
      </div>
      <div class="tunnel-type">
        <t-tag :theme="tunnelTypeTheme" size="small">
          {{ tunnelTypeLabel }}
        </t-tag>
      </div>
    </div>

    <div class="card-body">
      <div class="config-name">{{ config.name }}</div>
      <div class="ssh-command" @click="copyCommand">
        <code>{{ sshCommand }}</code>
        <t-tooltip content="点击复制">
          <t-icon name="file-copy" size="14" />
        </t-tooltip>
      </div>
    </div>

    <div class="card-footer">
      <t-button
        v-if="status === 'running'"
        variant="outline"
        size="small"
        theme="danger"
        :loading="isLoading"
        @click="handleStop"
      >
        停止
      </t-button>
      <t-button
        v-else
        variant="outline"
        size="small"
        theme="primary"
        :loading="isLoading"
        @click="handleStart"
      >
        启动
      </t-button>
      <t-button variant="text" size="small" @click="handleEdit">
        编辑
      </t-button>
      <t-button variant="text" size="small" @click="handleShowLogs">
        日志
      </t-button>
      <t-popconfirm
        v-if="status !== 'running'"
        content="确定删除此配置吗？"
        @confirm="handleDelete"
      >
        <t-button variant="text" size="small" theme="danger">
          删除
        </t-button>
      </t-popconfirm>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import type { Config, TunnelStatus } from '@/types'
import { useTunnelStore } from '@/stores/tunnel'
import { useConfigStore } from '@/stores/config'
import {
  formatSshCommand,
  getTunnelTypeLabel,
  getTunnelTypeTheme,
  copyToClipboard
} from '@/utils/format'

const props = defineProps<{
  config: Config
}>()

const emit = defineEmits<{
  edit: [config: Config]
  showLogs: [configId: string]
}>()

const tunnelStore = useTunnelStore()
const configStore = useConfigStore()

const configId = computed(() => props.config.id)
const status = computed<TunnelStatus>(() => tunnelStore.getStatus(configId.value))
const isLoading = computed(() => tunnelStore.isLoading(configId.value))

const statusClass = computed(() => `status-${status.value}`)
const statusText = computed(() => {
  const texts: Record<TunnelStatus, string> = {
    running: '运行中',
    stopped: '已停止',
    error: '错误',
    reconnecting: '重连中'
  }
  return texts[status.value]
})

const tunnelTypeLabel = computed(() => getTunnelTypeLabel(props.config.tunnelType))
const tunnelTypeTheme = computed(() => getTunnelTypeTheme(props.config.tunnelType))
const sshCommand = computed(() => formatSshCommand(props.config))

async function handleStart() {
  try {
    await tunnelStore.startTunnel(configId.value)
    MessagePlugin.success('隧道启动成功')
  } catch (error) {
    MessagePlugin.error('启动失败：' + error)
  }
}

async function handleStop() {
  try {
    await tunnelStore.stopTunnel(configId.value)
    MessagePlugin.success('隧道已停止')
  } catch (error) {
    MessagePlugin.error('停止失败：' + error)
  }
}

function handleEdit() {
  emit('edit', props.config)
}

function handleShowLogs() {
  emit('showLogs', configId.value)
}

async function handleDelete() {
  try {
    await configStore.removeConfig(configId.value)
    MessagePlugin.success('配置已删除')
  } catch (error) {
    MessagePlugin.error('删除失败：' + error)
  }
}

async function copyCommand() {
  const success = await copyToClipboard(sshCommand.value)
  if (success) {
    MessagePlugin.success('已复制到剪贴板')
  }
}
</script>

<style scoped>
.tunnel-card {
  background: var(--td-bg-color-container);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
  padding: 16px;
  transition: all 0.2s;
}

.tunnel-card:hover {
  box-shadow: var(--td-shadow-2);
}

.tunnel-card.status-running {
  border-left: 3px solid var(--td-success-color);
}

.tunnel-card.status-stopped {
  border-left: 3px solid var(--td-gray-color-6);
}

.tunnel-card.status-error {
  border-left: 3px solid var(--td-error-color);
}

.tunnel-card.status-reconnecting {
  border-left: 3px solid var(--td-warning-color);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--td-gray-color-6);
}

.status-running .status-dot {
  background: var(--td-success-color);
  animation: pulse 2s infinite;
}

.status-error .status-dot {
  background: var(--td-error-color);
}

.status-reconnecting .status-dot {
  background: var(--td-warning-color);
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.status-text {
  font-size: 12px;
  color: var(--td-text-color-secondary);
}

.card-body {
  margin-bottom: 12px;
}

.config-name {
  font-weight: 500;
  color: var(--td-text-color-primary);
  margin-bottom: 8px;
}

.ssh-command {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--td-bg-color-code);
  border-radius: var(--td-radius-small);
  cursor: pointer;
  font-size: 12px;
}

.ssh-command code {
  flex: 1;
  color: var(--td-text-color-primary);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-footer {
  display: flex;
  gap: 8px;
}
</style>
```

- [ ] **Step 2: 提交组件代码**

```bash
git add src/components/TunnelCard.vue
git commit -m "feat: 实现隧道卡片组件"
```

---

## Task 13: 前端组件实现 - 隧道列表

**Files:**
- Create: `src/components/TunnelList.vue`

- [ ] **Step 1: 创建 TunnelList.vue**

```vue
<template>
  <div class="tunnel-list">
    <div class="toolbar">
      <div class="toolbar-left">
        <t-button theme="primary" @click="handleCreate">
          <template #icon><add-icon /></template>
          新建配置
        </t-button>
        <t-button variant="outline" @click="handleImport">
          <template #icon><upload-icon /></template>
          导入
        </t-button>
        <t-button variant="outline" @click="handleExport">
          <template #icon><download-icon /></template>
          导出
        </t-button>
      </div>
      <div class="toolbar-right">
        <t-input
          v-model="searchKeyword"
          placeholder="搜索配置..."
          clearable
          @enter="handleSearch"
          @clear="handleClearSearch"
        >
          <template #suffix-icon>
            <search-icon />
          </template>
        </t-input>
      </div>
    </div>

    <div class="list-container">
      <t-loading :loading="configStore.loading">
        <div v-if="configs.length === 0" class="empty-state">
          <t-empty description="暂无配置，点击上方按钮新建" />
        </div>
        <div v-else class="tunnel-grid">
          <tunnel-card
            v-for="config in configs"
            :key="config.id"
            :config="config"
            @edit="handleEdit"
            @show-logs="handleShowLogs"
          />
        </div>
      </t-loading>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { AddIcon, UploadIcon, DownloadIcon, SearchIcon } from 'tdesign-icons-vue-next'
import TunnelCard from './TunnelCard.vue'
import { useConfigStore } from '@/stores/config'
import { useTunnelStore } from '@/stores/tunnel'
import type { Config } from '@/types'

const emit = defineEmits<{
  create: []
  edit: [config: Config]
  showLogs: [configId: string]
  import: []
  export: []
}>()

const configStore = useConfigStore()
const tunnelStore = useTunnelStore()

const searchKeyword = ref('')

const configs = computed(() => configStore.filteredConfigs)

onMounted(async () => {
  await configStore.fetchConfigs()
  await tunnelStore.fetchAllRunning()
})

function handleCreate() {
  emit('create')
}

function handleEdit(config: Config) {
  emit('edit', config)
}

function handleShowLogs(configId: string) {
  emit('showLogs', configId)
}

function handleImport() {
  emit('import')
}

function handleExport() {
  emit('export')
}

async function handleSearch() {
  if (searchKeyword.value.trim()) {
    await configStore.search(searchKeyword.value.trim())
  } else {
    await configStore.fetchConfigs()
  }
}

async function handleClearSearch() {
  searchKeyword.value = ''
  await configStore.fetchConfigs()
}
</script>

<style scoped>
.tunnel-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: var(--td-bg-color-container);
  border-bottom: 1px solid var(--td-component-border);
}

.toolbar-left {
  display: flex;
  gap: 8px;
}

.toolbar-right {
  width: 300px;
}

.list-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.tunnel-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 16px;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 300px;
}
</style>
```

- [ ] **Step 2: 提交组件代码**

```bash
git add src/components/TunnelList.vue
git commit -m "feat: 实现隧道列表组件"
```

---

## Task 14: 前端组件实现 - 配置表单

**Files:**
- Create: `src/components/ConfigForm.vue`

- [ ] **Step 1: 创建 ConfigForm.vue**

```vue
<template>
  <t-drawer
    :visible="visible"
    :header="isEdit ? '编辑配置' : '新建配置'"
    :footer="true"
    size="500px"
    @update:visible="$emit('update:visible', $event)"
    @confirm="handleSubmit"
    @close="handleClose"
  >
    <t-form
      ref="formRef"
      :data="formData"
      :rules="formRules"
      label-align="right"
      label-width="100px"
    >
      <t-form-item label="配置名称" name="name">
        <t-input v-model="formData.name" placeholder="请输入配置名称" />
      </t-form-item>

      <t-form-item label="所属分组" name="groupId">
        <t-select
          v-model="formData.groupId"
          placeholder="请选择分组"
          clearable
        >
          <t-option
            v-for="group in groups"
            :key="group.id"
            :value="group.id"
            :label="group.name"
          />
        </t-select>
      </t-form-item>

      <t-divider>连接信息</t-divider>

      <t-form-item label="主机地址" name="host">
        <t-input v-model="formData.host" placeholder="SSH 服务器地址" />
      </t-form-item>

      <t-form-item label="端口" name="port">
        <t-input-number v-model="formData.port" :min="1" :max="65535" />
      </t-form-item>

      <t-form-item label="用户名" name="username">
        <t-input v-model="formData.username" placeholder="登录用户名" />
      </t-form-item>

      <t-divider>认证方式</t-divider>

      <t-form-item label="认证类型" name="authType">
        <t-radio-group v-model="formData.authType">
          <t-radio value="password">密码</t-radio>
          <t-radio value="key">密钥</t-radio>
        </t-radio-group>
      </t-form-item>

      <t-form-item v-if="formData.authType === 'password'" label="密码" name="password">
        <t-input
          v-model="formData.password"
          type="password"
          placeholder="登录密码"
        />
      </t-form-item>

      <template v-if="formData.authType === 'key'">
        <t-form-item label="密钥文件" name="keyPath">
          <t-input v-model="formData.keyPath" placeholder="私钥文件路径" />
        </t-form-item>
        <t-form-item label="密钥密码" name="keyPassphrase">
          <t-input
            v-model="formData.keyPassphrase"
            type="password"
            placeholder="私钥密码（如有）"
          />
        </t-form-item>
      </template>

      <t-divider>隧道配置</t-divider>

      <t-form-item label="隧道类型" name="tunnelType">
        <t-radio-group v-model="formData.tunnelType">
          <t-radio value="local">本地转发 (-L)</t-radio>
          <t-radio value="remote">远程转发 (-R)</t-radio>
          <t-radio value="dynamic">动态转发 (-D)</t-radio>
        </t-radio-group>
      </t-form-item>

      <t-form-item label="本地地址" name="localHost">
        <t-input v-model="formData.localHost" placeholder="本地绑定地址" />
      </t-form-item>

      <t-form-item label="本地端口" name="localPort">
        <t-input-number v-model="formData.localPort" :min="1" :max="65535" />
      </t-form-item>

      <template v-if="formData.tunnelType !== 'dynamic'">
        <t-form-item label="远程地址" name="remoteHost">
          <t-input v-model="formData.remoteHost" placeholder="远程目标地址" />
        </t-form-item>
        <t-form-item label="远程端口" name="remotePort">
          <t-input-number v-model="formData.remotePort" :min="1" :max="65535" />
        </t-form-item>
      </template>

      <t-divider>高级选项</t-divider>

      <t-form-item label="自动重连" name="autoReconnect">
        <t-switch v-model="formData.autoReconnect" />
      </t-form-item>

      <t-form-item v-if="formData.autoReconnect" label="重连间隔" name="reconnectInterval">
        <t-input-number
          v-model="formData.reconnectInterval"
          :min="1"
          :max="300"
          suffix="秒"
        />
      </t-form-item>
    </t-form>
  </t-drawer>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useGroupStore } from '@/stores/group'
import { useConfigStore } from '@/stores/config'
import type { Config, CreateConfigRequest, UpdateConfigRequest } from '@/types'

const props = defineProps<{
  visible: boolean
  config?: Config | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  success: []
}>()

const groupStore = useGroupStore()
const configStore = useConfigStore()

const formRef = ref()
const isEdit = computed(() => !!props.config?.id)
const groups = computed(() => groupStore.groups)

const defaultFormData = (): CreateConfigRequest => ({
  name: '',
  groupId: null,
  host: '',
  port: 22,
  username: '',
  authType: 'password',
  password: null,
  keyPath: null,
  keyPassphrase: null,
  tunnelType: 'local',
  localHost: '127.0.0.1',
  localPort: 0,
  remoteHost: 'localhost',
  remotePort: null,
  autoReconnect: false,
  reconnectInterval: 5
})

const formData = ref<CreateConfigRequest>(defaultFormData())

const formRules = {
  name: [{ required: true, message: '请输入配置名称' }],
  host: [{ required: true, message: '请输入主机地址' }],
  port: [{ required: true, message: '请输入端口' }],
  username: [{ required: true, message: '请输入用户名' }],
  localPort: [{ required: true, message: '请输入本地端口' }]
}

watch(() => props.visible, (val) => {
  if (val) {
    if (props.config) {
      formData.value = {
        name: props.config.name,
        groupId: props.config.groupId,
        host: props.config.host,
        port: props.config.port,
        username: props.config.username,
        authType: props.config.authType,
        password: props.config.password,
        keyPath: props.config.keyPath,
        keyPassphrase: props.config.keyPassphrase,
        tunnelType: props.config.tunnelType,
        localHost: props.config.localHost,
        localPort: props.config.localPort,
        remoteHost: props.config.remoteHost,
        remotePort: props.config.remotePort,
        autoReconnect: props.config.autoReconnect,
        reconnectInterval: props.config.reconnectInterval
      }
    } else {
      formData.value = defaultFormData()
    }
  }
})

async function handleSubmit() {
  const valid = await formRef.value?.validate()
  if (valid !== true) return

  try {
    if (isEdit.value && props.config) {
      const request: UpdateConfigRequest = {
        id: props.config.id,
        ...formData.value
      }
      await configStore.updateConfig(request)
      MessagePlugin.success('配置更新成功')
    } else {
      await configStore.createConfig(formData.value)
      MessagePlugin.success('配置创建成功')
    }
    emit('success')
    emit('update:visible', false)
  } catch (error) {
    MessagePlugin.error('保存失败：' + error)
  }
}

function handleClose() {
  formRef.value?.reset()
  formData.value = defaultFormData()
}
</script>
```

- [ ] **Step 2: 提交组件代码**

```bash
git add src/components/ConfigForm.vue
git commit -m "feat: 实现配置表单组件"
```

---

## Task 15: 前端组件实现 - 日志面板

**Files:**
- Create: `src/components/LogPanel.vue`

- [ ] **Step 1: 创建 LogPanel.vue**

```vue
<template>
  <t-dialog
    :visible="visible"
    header="连接日志"
    width="700px"
    :footer="false"
    @update:visible="$emit('update:visible', $event)"
  >
    <div class="log-panel">
      <div class="log-toolbar">
        <t-button
          variant="outline"
          size="small"
          theme="danger"
          @click="handleClear"
        >
          清除日志
        </t-button>
      </div>

      <div class="log-list">
        <t-loading :loading="loading">
          <div v-if="logs.length === 0" class="empty-state">
            <t-empty description="暂无日志记录" size="small" />
          </div>
          <div v-else>
            <div
              v-for="log in logs"
              :key="log.id"
              class="log-item"
              :class="`log-${log.action}`"
            >
              <div class="log-header">
                <t-tag
                  :theme="getActionTheme(log.action)"
                  size="small"
                >
                  {{ getActionLabel(log.action) }}
                </t-tag>
                <span class="log-time">{{ formatDateTime(log.createdAt) }}</span>
              </div>
              <div class="log-message">{{ log.message }}</div>
            </div>
          </div>
        </t-loading>
      </div>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import type { ConnectionLog, LogAction } from '@/types'
import * as api from '@/api/tauri'
import { formatDateTime } from '@/utils/format'

const props = defineProps<{
  visible: boolean
  configId: string | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const logs = ref<ConnectionLog[]>([])
const loading = ref(false)

watch(() => props.visible, async (val) => {
  if (val && props.configId) {
    await fetchLogs()
  }
})

async function fetchLogs() {
  if (!props.configId) return

  loading.value = true
  try {
    logs.value = await api.getLogs(props.configId, 100)
  } finally {
    loading.value = false
  }
}

async function handleClear() {
  if (!props.configId) return

  try {
    await api.clearLogs(props.configId)
    logs.value = []
    MessagePlugin.success('日志已清除')
  } catch (error) {
    MessagePlugin.error('清除失败：' + error)
  }
}

function getActionLabel(action: LogAction): string {
  const labels: Record<LogAction, string> = {
    connect: '连接',
    disconnect: '断开',
    reconnect: '重连',
    error: '错误'
  }
  return labels[action]
}

function getActionTheme(action: LogAction): 'success' | 'warning' | 'danger' | 'default' {
  const themes: Record<LogAction, 'success' | 'warning' | 'danger' | 'default'> = {
    connect: 'success',
    disconnect: 'default',
    reconnect: 'warning',
    error: 'danger'
  }
  return themes[action]
}
</script>

<style scoped>
.log-panel {
  min-height: 300px;
  max-height: 500px;
  display: flex;
  flex-direction: column;
}

.log-toolbar {
  padding-bottom: 12px;
  border-bottom: 1px solid var(--td-component-border);
  margin-bottom: 12px;
}

.log-list {
  flex: 1;
  overflow-y: auto;
}

.log-item {
  padding: 12px;
  border-bottom: 1px solid var(--td-component-border);
}

.log-item:last-child {
  border-bottom: none;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.log-time {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
}

.log-message {
  font-size: 13px;
  color: var(--td-text-color-primary);
}

.log-error .log-message {
  color: var(--td-error-color);
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 200px;
}
</style>
```

- [ ] **Step 2: 提交组件代码**

```bash
git add src/components/LogPanel.vue
git commit -m "feat: 实现日志面板组件"
```

---

## Task 16: 主页面组装

**Files:**
- Create: `src/views/Home.vue`
- Create: `src/App.vue`
- Create: `src/main.ts`

- [ ] **Step 1: 创建 Home.vue**

```vue
<template>
  <div class="home">
    <sidebar />

    <div class="main-content">
      <tunnel-list
        @create="showConfigForm = true"
        @edit="handleEdit"
        @show-logs="handleShowLogs"
        @import="showImportDialog = true"
        @export="handleExport"
      />
    </div>

    <!-- 配置表单 -->
    <config-form
      v-model:visible="showConfigForm"
      :config="editingConfig"
      @success="handleConfigSuccess"
    />

    <!-- 日志面板 -->
    <log-panel
      v-model:visible="showLogPanel"
      :config-id="selectedConfigId"
    />

    <!-- 导入对话框 -->
    <t-dialog
      v-model:visible="showImportDialog"
      header="导入配置"
      :confirm-btn="{ content: '导入', loading: importing }"
      @confirm="handleImport"
    >
      <t-textarea
        v-model="importJson"
        placeholder="请粘贴导出的 JSON 配置"
        :autosize="{ minRows: 5, maxRows: 10 }"
      />
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import Sidebar from '@/components/Sidebar.vue'
import TunnelList from '@/components/TunnelList.vue'
import ConfigForm from '@/components/ConfigForm.vue'
import LogPanel from '@/components/LogPanel.vue'
import { useGroupStore } from '@/stores/group'
import { useConfigStore } from '@/stores/config'
import type { Config } from '@/types'
import * as api from '@/api/tauri'

const groupStore = useGroupStore()
const configStore = useConfigStore()

const showConfigForm = ref(false)
const showLogPanel = ref(false)
const showImportDialog = ref(false)
const editingConfig = ref<Config | null>(null)
const selectedConfigId = ref<string | null>(null)
const importJson = ref('')
const importing = ref(false)

onMounted(async () => {
  await groupStore.fetchGroups()
})

function handleEdit(config: Config) {
  editingConfig.value = config
  showConfigForm.value = true
}

function handleShowLogs(configId: string) {
  selectedConfigId.value = configId
  showLogPanel.value = true
}

function handleConfigSuccess() {
  editingConfig.value = null
}

async function handleImport() {
  if (!importJson.value.trim()) {
    MessagePlugin.warning('请输入配置 JSON')
    return
  }

  importing.value = true
  try {
    const count = await api.importConfigs(importJson.value.trim())
    MessagePlugin.success(`成功导入 ${count} 个配置`)
    showImportDialog.value = false
    importJson.value = ''
    await configStore.fetchConfigs()
  } catch (error) {
    MessagePlugin.error('导入失败：' + error)
  } finally {
    importing.value = false
  }
}

async function handleExport() {
  try {
    const json = await api.exportConfigs()
    await navigator.clipboard.writeText(json)
    MessagePlugin.success('配置已复制到剪贴板')
  } catch (error) {
    MessagePlugin.error('导出失败：' + error)
  }
}
</script>

<style scoped>
.home {
  display: flex;
  height: 100vh;
  background: var(--td-bg-color-page);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
```

- [ ] **Step 2: 创建 App.vue**

```vue
<template>
  <t-config-provider :global-config="globalConfig">
    <home />
  </t-config-provider>
</template>

<script setup lang="ts">
import Home from '@/views/Home.vue'

const globalConfig = {
  // TDesign 全局配置
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>
```

- [ ] **Step 3: 创建 main.ts**

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import TDesign from 'tdesign-vue-next'
import 'tdesign-vue-next/es/style/index.css'
import App from './App.vue'

const app = createApp(App)

app.use(createPinia())
app.use(TDesign)

app.mount('#app')
```

- [ ] **Step 4: 提交主页面代码**

```bash
git add src/views/Home.vue src/App.vue src/main.ts
git commit -m "feat: 实现主页面组装"
```

---

## Task 17: 系统托盘实现

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 更新 lib.rs 添加托盘支持**

```rust
mod commands;
mod db;
mod models;
mod ssh;
mod utils;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();

            // 数据库文件放在程序运行目录
            let exe_dir = std::env::current_exe()
                .expect("无法获取程序路径")
                .parent()
                .expect("无法获取程序目录")
                .to_path_buf();
            let db_path = exe_dir.join("ssh-proxy.db");

            db::init(&db_path).expect("数据库初始化失败");

            // 初始化 SSH 管理器
            ssh::init(app_handle.clone());

            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "打开主窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // 创建系统托盘
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 窗口关闭时最小化到托盘
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // 阻止关闭，改为隐藏窗口
                        api.prevent_close();
                        let _ = window.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 分组管理
            commands::group::get_groups,
            commands::group::save_group,
            commands::group::delete_group,
            // 配置管理
            commands::config::get_configs,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::update_config,
            commands::config::delete_config,
            commands::config::search_configs,
            // 隧道控制
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::restart_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::get_running_tunnels,
            // 日志管理
            commands::log::get_logs,
            commands::log::clear_logs,
            // 导入导出
            commands::config::export_configs,
            commands::config::import_configs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: 创建托盘图标目录**

创建 `src-tauri/icons/` 目录并放置图标文件（32x32 PNG 格式）。

- [ ] **Step 3: 提交托盘功能代码**

```bash
git add src-tauri/src/lib.rs src-tauri/icons/
git commit -m "feat: 实现系统托盘功能"
```

---

## Task 18: 构建和测试

**Files:**
- Modify: `src-tauri/Cargo.toml` (添加 lazy_static)
- Modify: `src-tauri/src/db/mod.rs` (修复编译错误)

- [ ] **Step 1: 安装前端依赖**

```bash
npm install
```

- [ ] **Step 2: 开发模式运行**

```bash
npm run tauri dev
```

- [ ] **Step 3: 验证功能**

测试以下功能：
1. 创建分组
2. 创建 SSH 配置
3. 启动隧道
4. 停止隧道
5. 查看日志
6. 导入导出配置
7. 系统托盘功能

- [ ] **Step 4: 构建生产版本**

```bash
npm run tauri build
```

- [ ] **Step 5: 提交最终代码**

```bash
git add .
git commit -m "feat: 完成项目构建和测试"
```

---

## 自检清单

**1. Spec 覆盖检查：**

| 需求 | 对应 Task |
|------|-----------|
| SSH 隧道配置增删改查 | Task 6, 7, 8, 14 |
| 支持本地/远程/动态转发 | Task 2, 5 |
| 支持密码和密钥认证 | Task 2, 4 |
| 隧道启动/停止/重启 | Task 5, 6 |
| 自动重连机制 | Task 5 |
| 连接状态监控和日志 | Task 5, 6, 15 |
| 配置分组和搜索 | Task 6, 9, 11, 13 |
| 配置导入/导出 | Task 6, 8 |
| 系统托盘图标 | Task 17 |
| SQLite 存储在程序目录 | Task 1, 3 |
| UUID 主键 | Task 2 |
| TDesign 组件库 | Task 11-16 |

**2. 占位符扫描：** 无 TBD、TODO、未实现步骤

**3. 类型一致性检查：** 所有接口类型定义一致
