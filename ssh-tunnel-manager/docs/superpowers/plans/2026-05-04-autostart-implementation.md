# 开机启动功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 SSH 隧道管理器增加软件开机启动和隧道开机启动功能。

**Architecture:** 软件开机启动通过 Windows 注册表实现，隧道开机启动在应用 setup 阶段执行并带重试机制。前端通过 Tauri 命令与后端交互。

**Tech Stack:** Tauri 2、Rust、Vue 3、TDesign、SQLite

---

## 文件结构

| 文件 | 变更类型 | 责任 |
|------|----------|------|
| `src-tauri/src/models/config.rs` | 修改 | Config 结构新增 auto_start 字段 |
| `src-tauri/src/db/mod.rs` | 修改 | 数据库迁移脚本 |
| `src-tauri/src/db/sqlite.rs` | 修改 | SQL 映射新增列 |
| `src-tauri/src/utils/autostart.rs` | 新建 | 软件开机启动注册表操作 |
| `src-tauri/src/utils/mod.rs` | 修改 | 导出 autostart 模块 |
| `src-tauri/src/commands/autostart.rs` | 新建 | Tauri 命令封装 |
| `src-tauri/src/commands/mod.rs` | 修改 | 导出 autostart 命令 |
| `src-tauri/src/commands/config.rs` | 修改 | 新增请求字段 |
| `src-tauri/src/ssh/autostart.rs` | 新建 | 隧道开机启动逻辑 + 重试机制 |
| `src-tauri/src/ssh/mod.rs` | 修改 | 导出 autostart 模块 |
| `src-tauri/src/lib.rs` | 修改 | 托盘菜单 + setup 调用 |
| `src/types/index.ts` | 修改 | 前端类型定义 |
| `src/api/tauri.ts` | 修改 | API 封装 + DTO 转换 |
| `src/components/ConfigForm.vue` | 修改 | 新增开关 |
| `src/components/TunnelCard.vue` | 修改 | 新增按钮 |
| `src/stores/config.ts` | 修改 | 新增 setAutoStart 方法 |

---

### Task 1: 后端数据模型变更

**Files:**
- Modify: `src-tauri/src/models/config.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: 在 Config 结构新增 auto_start 字段**

修改 `src-tauri/src/models/config.rs`，在 Config 结构体中新增字段：

```rust
// 在 Config 结构体中，is_favorite 字段后添加：
    #[serde(default)]
    pub auto_start: bool,
```

同时修改 CreateConfigRequest 和 UpdateConfigRequest 结构体：

```rust
// 在 CreateConfigRequest 结构体末尾添加：
    pub auto_start: Option<bool>,
```

```rust
// 在 UpdateConfigRequest 结构体末尾添加：
    pub auto_start: Option<bool>,
```

- [ ] **Step 2: 提交变更**

```bash
git add src-tauri/src/models/config.rs
git commit -m "feat(models): Config 新增 auto_start 字段"
```

---

### Task 2: 数据库迁移

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/db/sqlite.rs`

- [ ] **Step 1: 在 migrate_database 函数添加 auto_start 列迁移**

修改 `src-tauri/src/db/mod.rs`，在 `migrate_database()` 函数末尾添加：

```rust
    // 检查 auto_start 列是否存在
    let auto_start_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'auto_start'",
        [],
        |row| row.get(0),
    )?;

    if !auto_start_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN auto_start INTEGER DEFAULT 0", [])?;
    }

    Ok(())
```

- [ ] **Step 2: 修改 sqlite.rs 的 map_config_row 函数**

修改 `src-tauri/src/db/sqlite.rs`，在 `map_config_row` 函数中：

1. 修改 SQL 查询，添加 auto_start 列（约第 148 行开始的查询语句）：

```rust
// 将 SELECT 语句改为：
    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         ORDER BY created_at DESC"
    )?;
```

2. 在 Ok(Config { ... }) 中添加字段映射（约第 137 行后）：

```rust
// 在 favorite_order 字段后添加：
        auto_start: row.get::<_, i32>(21)? != 0,
```

注意：调整其他查询语句的列序号，auto_start 是第 22 列（索引 21）。

- [ ] **Step 3: 修改 get_config_by_id 查询**

修改 `src-tauri/src/db/sqlite.rs` 中 `get_config_by_id` 函数的 SQL：

```rust
    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE id = ?1"
    )?;
```

- [ ] **Step 4: 修改 get_configs_by_group 查询**

修改 `src-tauri/src/db/sqlite.rs` 中 `get_configs_by_group` 函数的 SQL：

```rust
    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE group_id = ?1
         ORDER BY created_at DESC"
    )?;
```

- [ ] **Step 5: 修改 get_favorites 查询**

修改 `src-tauri/src/db/sqlite.rs` 中 `get_favorites` 函数的 SQL：

```rust
    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE is_favorite = 1
         ORDER BY favorite_order ASC"
    )?;
```

- [ ] **Step 6: 修改 save_config 函数**

修改 `src-tauri/src/db/sqlite.rs` 中 `save_config` 函数：

1. UPDATE 语句添加 auto_start：

```rust
// UPDATE 语句改为：
    conn.execute(
        "UPDATE configs SET
            name = ?1, group_id = ?2, host = ?3, port = ?4, username = ?5,
            auth_type = ?6, password = ?7, key_path = ?8, key_passphrase = ?9,
            tunnel_type = ?10, local_host = ?11, local_port = ?12,
            remote_host = ?13, remote_port = ?14, auto_reconnect = ?15,
            reconnect_interval = ?16, updated_at = ?17, is_favorite = ?18, favorite_order = ?19,
            auto_start = ?20
        WHERE id = ?21",
        params![
            config.name,
            config.group_id,
            config.host,
            config.port,
            config.username,
            auth_type_to_string(&config.auth_type),
            encrypted_password,
            config.key_path,
            encrypted_key_passphrase,
            tunnel_type_to_string(&config.tunnel_type),
            config.local_host,
            config.local_port,
            config.remote_host,
            config.remote_port,
            if config.auto_reconnect { 1 } else { 0 },
            config.reconnect_interval,
            config.updated_at.to_rfc3339(),
            if config.is_favorite { 1 } else { 0 },
            config.favorite_order,
            if config.auto_start { 1 } else { 0 },
            config.id,
        ],
    )?;
```

2. INSERT 语句添加 auto_start：

```rust
// INSERT 语句改为：
    conn.execute(
        "INSERT INTO configs (
            id, name, group_id, host, port, username, auth_type, password,
            key_path, key_passphrase, tunnel_type, local_host, local_port,
            remote_host, remote_port, auto_reconnect, reconnect_interval,
            created_at, updated_at, is_favorite, favorite_order, auto_start
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            config.id,
            config.name,
            config.group_id,
            config.host,
            config.port,
            config.username,
            auth_type_to_string(&config.auth_type),
            encrypted_password,
            config.key_path,
            encrypted_key_passphrase,
            tunnel_type_to_string(&config.tunnel_type),
            config.local_host,
            config.local_port,
            config.remote_host,
            config.remote_port,
            if config.auto_reconnect { 1 } else { 0 },
            config.reconnect_interval,
            config.created_at.to_rfc3339(),
            config.updated_at.to_rfc3339(),
            if config.is_favorite { 1 } else { 0 },
            config.favorite_order,
            if config.auto_start { 1 } else { 0 },
        ],
    )?;
```

- [ ] **Step 7: 新增 set_auto_start 函数**

在 `src-tauri/src/db/sqlite.rs` 末尾添加：

```rust
/// 设置配置的开机启动状态
pub fn set_auto_start(config_id: &str, auto_start: bool) -> Result<Config, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "UPDATE configs SET auto_start = ?1 WHERE id = ?2",
        params![if auto_start { 1 } else { 0 }, config_id],
    )?;

    // 查询更新后的配置
    let config = conn.query_row(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path,
                key_passphrase, tunnel_type, local_host, local_port, remote_host, remote_port,
                auto_reconnect, reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs WHERE id = ?1",
        params![config_id],
        map_config_row,
    )?;

    Ok(config)
}

/// 获取所有开机启动的配置
pub fn get_auto_start_configs() -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order, auto_start
         FROM configs
         WHERE auto_start = 1"
    )?;

    let configs = stmt.query_map([], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}
```

- [ ] **Step 8: 在 db/mod.rs 导出新函数**

修改 `src-tauri/src/db/mod.rs`，确保 `pub use sqlite::*;` 会导出新函数。

- [ ] **Step 9: 提交变更**

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/db/sqlite.rs
git commit -m "feat(db): 数据库支持 auto_start 字段"
```

---

### Task 3: 软件开机启动模块

**Files:**
- Create: `src-tauri/src/utils/autostart.rs`
- Modify: `src-tauri/src/utils/mod.rs`

- [ ] **Step 1: 创建 autostart.rs 模块**

创建 `src-tauri/src/utils/autostart.rs`：

```rust
use std::error::Error;
use winreg::RegKey;
use winreg::enums::*;

const REG_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "SSHTunnelManager";

/// 检查是否已设置开机启动
pub fn is_autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = hkcu.open_subkey(REG_KEY_PATH).ok();

    if let Some(key) = path {
        key.get_value(APP_NAME).ok().is_some()
    } else {
        false
    }
}

/// 启用开机启动
pub fn enable_autostart() -> Result<(), Box<dyn Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.create_subkey(REG_KEY_PATH)?;

    key.set_value(APP_NAME, &exe_path_str)?;

    Ok(())
}

/// 禁用开机启动
pub fn disable_autostart() -> Result<(), Box<dyn Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = hkcu.open_subkey(REG_KEY_PATH).ok();

    if let Some(key) = path {
        // 删除注册表项（忽略不存在的情况）
        let _ = key.delete_value(APP_NAME);
    }

    Ok(())
}
```

- [ ] **Step 2: 添加 winreg 依赖**

修改 `src-tauri/Cargo.toml`，在 dependencies 中添加：

```toml
winreg = "0.52"
```

- [ ] **Step 3: 导出 autostart 模块**

修改 `src-tauri/src/utils/mod.rs`：

```rust
// 工具函数模块
pub mod crypto;
pub mod autostart;
```

- [ ] **Step 4: 提交变更**

```bash
git add src-tauri/src/utils/autostart.rs src-tauri/src/utils/mod.rs src-tauri/Cargo.toml
git commit -m "feat(utils): 软件开机启动注册表操作模块"
```

---

### Task 4: Tauri 命令封装

**Files:**
- Create: `src-tauri/src/commands/autostart.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 autostart 命令模块**

创建 `src-tauri/src/commands/autostart.rs`：

```rust
use crate::utils::autostart;

/// 获取软件开机启动状态
#[tauri::command]
pub fn get_autostart_status() -> bool {
    autostart::is_autostart_enabled()
}

/// 设置软件开机启动状态
#[tauri::command]
pub fn set_autostart(enable: bool) -> Result<(), String> {
    if enable {
        autostart::enable_autostart().map_err(|e| e.to_string())?;
    } else {
        autostart::disable_autostart().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

- [ ] **Step 2: 新增隧道开机启动命令**

在 `src-tauri/src/commands/autostart.rs` 中添加：

```rust
use crate::db;

/// 设置隧道开机启动状态
#[tauri::command]
pub fn set_tunnel_autostart(config_id: String, auto_start: bool) -> Result<crate::models::Config, String> {
    db::set_auto_start(&config_id, auto_start).map_err(|e| e.to_string())
}

/// 获取所有开机启动的隧道配置
#[tauri::command]
pub fn get_autostart_tunnels() -> Result<Vec<crate::models::Config>, String> {
    db::get_auto_start_configs().map_err(|e| e.to_string())
}
```

- [ ] **Step 3: 导出 autostart 命令**

修改 `src-tauri/src/commands/mod.rs`：

```rust
// 分组管理命令
pub mod group;
// 配置管理命令
pub mod config;
// 隧道控制命令
pub mod tunnel;
// 日志管理命令
pub mod log;
// 开机启动命令
pub mod autostart;
```

- [ ] **Step 4: 修改 config 命令支持 auto_start**

修改 `src-tauri/src/commands/config.rs`：

1. 在 `save_config` 命令中处理 auto_start：

找到 `save_config` 函数，在构建 Config 时添加 auto_start 字段：

```rust
// 在 Config 结构体构建中添加：
    auto_start: request.auto_start.unwrap_or(false),
```

2. 在 `update_config` 命令中处理 auto_start：

找到 `update_config` 函数，在获取现有配置后更新：

```rust
// 更新 auto_start 字段
    if let Some(auto_start) = request.auto_start {
        existing_config.auto_start = auto_start;
    }
```

- [ ] **Step 5: 在 lib.rs 注册新命令**

修改 `src-tauri/src/lib.rs`，在 `invoke_handler` 中添加：

```rust
        .invoke_handler(tauri::generate_handler![
            // ... 现有命令 ...
            commands::autostart::get_autostart_status,
            commands::autostart::set_autostart,
            commands::autostart::set_tunnel_autostart,
            commands::autostart::get_autostart_tunnels,
        ])
```

- [ ] **Step 6: 提交变更**

```bash
git add src-tauri/src/commands/autostart.rs src-tauri/src/commands/mod.rs src-tauri/src/commands/config.rs src-tauri/src/lib.rs
git commit -m "feat(commands): 开机启动 Tauri 命令"
```

---

### Task 5: 托盘菜单集成

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 build_tray_menu 添加开机启动菜单项**

修改 `src-tauri/src/lib.rs` 中 `build_tray_menu` 函数：

1. 在函数开头获取开机启动状态：

```rust
use crate::utils::autostart;

// 在函数开头添加：
    let autostart_enabled = autostart::is_autostart_enabled();
```

2. 在菜单末尾添加开机启动菜单项（在 show_item 之前）：

```rust
// 在分隔线和基础菜单项部分，添加：
    use tauri::menu::CheckMenuItemBuilder;

    let autostart_item = CheckMenuItemBuilder::new("开机启动")
        .checked(autostart_enabled)
        .enabled(true)
        .id("autostart")
        .build(app)?;
    menu.append(&autostart_item)?;
```

- [ ] **Step 2: 在菜单事件处理中添加开机启动逻辑**

修改 `src-tauri/src/lib.rs` 中 `on_menu_event` 处理：

```rust
                        "autostart" => {
                            // 获取当前状态
                            let current = autostart::is_autostart_enabled();
                            // 切换状态
                            if current {
                                let _ = autostart::disable_autostart();
                            } else {
                                let _ = autostart::enable_autostart();
                            }
                            // 更新菜单
                            update_tray_menu(app);
                        }
```

- [ ] **Step 3: 提交变更**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(tray): 托盘菜单开机启动选项"
```

---

### Task 6: 隧道开机启动流程

**Files:**
- Create: `src-tauri/src/ssh/autostart.rs`
- Modify: `src-tauri/src/ssh/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建隧道开机启动模块**

创建 `src-tauri/src/ssh/autostart.rs`：

```rust
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use tauri::AppHandle;

use crate::db;
use crate::ssh::{start_ssh_tunnel, start_monitor_with_defaults, TUNNELS};
use crate::models::TunnelStatus;

/// 重试任务
struct RetryTask {
    config_id: String,
    retry_count: u32,
}

/// 重试队列
static RETRY_QUEUE: LazyLock<Mutex<HashMap<String, RetryTask>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

const MAX_RETRY_COUNT: u32 = 10;
const RETRY_INTERVAL_SECS: u64 = 60;

/// 启动所有开机启动的隧道
pub fn start_auto_start_tunnels(app: &AppHandle) {
    use crate::ssh::APP_HANDLE;

    // 获取所有开机启动的配置
    let configs = match db::get_auto_start_configs() {
        Ok(c) => c,
        Err(e) => {
            println!("获取开机启动配置失败: {}", e);
            return;
        }
    };

    if configs.is_empty() {
        return;
    }

    println!("启动 {} 个开机启动隧道...", configs.len());

    let app_clone = app.clone();

    // 启动每个隧道
    for config in configs {
        let config_id = config.id.clone();
        let auto_reconnect = config.auto_reconnect;
        let reconnect_interval = config.reconnect_interval;

        // 尝试启动隧道
        let result = start_ssh_tunnel(&config);

        match result {
            Ok(_) => {
                // 启动监控
                start_monitor_with_defaults(config_id.clone(), auto_reconnect, reconnect_interval);
                println!("隧道 {} 启动成功", config.name);
            }
            Err(e) => {
                println!("隧道 {} 启动失败: {}，加入重试队列", config.name, e);
                // 加入重试队列
                let mut queue = RETRY_QUEUE.lock().unwrap();
                queue.insert(config_id.clone(), RetryTask {
                    config_id: config_id,
                    retry_count: 0,
                });
            }
        }
    }

    // 启动重试任务
    spawn_retry_task(app_clone);
}

/// 启动重试任务
fn spawn_retry_task(app: AppHandle) {
    use tokio::time::{interval, Duration};

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(RETRY_INTERVAL_SECS));

        loop {
            ticker.tick();

            let mut queue = RETRY_QUEUE.lock().unwrap();
            if queue.is_empty() {
                continue;
            }

            // 获取需要重试的任务
            let retry_tasks: Vec<RetryTask> = queue.values().cloned().collect();
            queue.clear();

            for task in retry_tasks {
                if task.retry_count >= MAX_RETRY_COUNT {
                    println!("隧道 {} 重试次数已达上限，放弃重试", task.config_id);
                    continue;
                }

                // 获取配置
                let config = db::get_config_by_id(&task.config_id);
                if let Ok(Some(cfg)) = config {
                    let result = start_ssh_tunnel(&cfg);

                    match result {
                        Ok(_) => {
                            start_monitor_with_defaults(cfg.id.clone(), cfg.auto_reconnect, cfg.reconnect_interval);
                            println!("隧道 {} 重试启动成功", cfg.name);
                        }
                        Err(e) => {
                            println!("隧道 {} 重试启动失败 (第 {} 次): {}", cfg.name, task.retry_count + 1, e);
                            // 重新加入队列，增加计数
                            queue.insert(task.config_id.clone(), RetryTask {
                                config_id: task.config_id,
                                retry_count: task.retry_count + 1,
                            });
                        }
                    }
                }
            }
        }
    });
}

/// 检查是否有重试任务
pub fn has_retry_tasks() -> bool {
    let queue = RETRY_QUEUE.lock().unwrap();
    !queue.is_empty()
}
```

- [ ] **Step 2: 导出 autostart 模块**

修改 `src-tauri/src/ssh/mod.rs`：

```rust
mod sidecar;
mod monitor;
mod autostart;

// ... 其他代码 ...

pub use autostart::*;
```

- [ ] **Step 3: 在 setup 中调用启动函数**

修改 `src-tauri/src/lib.rs`，在 setup 函数中添加：

```rust
            // 初始化 SSH 管理器
            ssh::init(app_handle.clone());

            // 启动开机启动的隧道
            ssh::start_auto_start_tunnels(app.handle());
```

位置：在 `ssh::init(app_handle.clone());` 之后添加。

- [ ] **Step 4: 提交变更**

```bash
git add src-tauri/src/ssh/autostart.rs src-tauri/src/ssh/mod.rs src-tauri/src/lib.rs
git commit -m "feat(ssh): 隧道开机启动流程及重试机制"
```

---

### Task 7: 前端类型定义

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: 在 Config 接口添加 autoStart 字段**

修改 `src/types/index.ts`，在 Config 接口中添加：

```typescript
// 在 favoriteOrder 字段后添加：
  autoStart: boolean
```

- [ ] **Step 2: 在请求类型添加字段**

修改 CreateConfigRequest 和 UpdateConfigRequest：

```typescript
// 在 CreateConfigRequest 中添加：
  autoStart?: boolean

// UpdateConfigRequest 继承 CreateConfigRequest，无需单独添加
```

- [ ] **Step 3: 提交变更**

```bash
git add src/types/index.ts
git commit -m "feat(types): 前端类型新增 autoStart"
```

---

### Task 8: API 封装

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 修改 ConfigDto 和转换函数**

修改 `src/api/tauri.ts`：

1. 在 ConfigDto 接口添加：

```typescript
interface ConfigDto {
  // ... 现有字段 ...
  auto_start: boolean
}
```

2. 在 configDtoToConfig 函数添加：

```typescript
function configDtoToConfig(dto: ConfigDto): Config {
  return {
    // ... 现有字段 ...
    autoStart: dto.auto_start,
  }
}
```

3. 在 createConfigRequestToDto 函数添加：

```typescript
function createConfigRequestToDto(req: CreateConfigRequest): Record<string, unknown> {
  return {
    // ... 现有字段 ...
    auto_start: req.autoStart ?? false,
  }
}
```

- [ ] **Step 2: 新增开机启动 API 函数**

在 `src/api/tauri.ts` 末尾添加：

```typescript
// ============================================
// 开机启动 API
// ============================================

/**
 * 获取软件开机启动状态
 */
export async function getAutostartStatus(): Promise<boolean> {
  return await invoke<boolean>('get_autostart_status')
}

/**
 * 设置软件开机启动状态
 */
export async function setAutostart(enable: boolean): Promise<void> {
  await invoke('set_autostart', { enable })
}

/**
 * 设置隧道开机启动状态
 */
export async function setTunnelAutostart(configId: string, autoStart: boolean): Promise<Config> {
  const dto = await invoke<ConfigDto>('set_tunnel_autostart', { configId, autoStart })
  return configDtoToConfig(dto)
}

/**
 * 获取所有开机启动的隧道配置
 */
export async function getAutostartTunnels(): Promise<Config[]> {
  const dtos = await invoke<ConfigDto[]>('get_autostart_tunnels')
  return dtos.map(configDtoToConfig)
}
```

- [ ] **Step 3: 提交变更**

```bash
git add src/api/tauri.ts
git commit -m "feat(api): 开机启动 API 封装"
```

---

### Task 9: ConfigStore 集成

**Files:**
- Modify: `src/stores/config.ts`

- [ ] **Step 1: 新增 setAutoStart 方法**

修改 `src/stores/config.ts`，在 return 前添加：

```typescript
  // 设置开机启动
  async function setAutoStart(configId: string, autoStart: boolean) {
    const config = await api.setTunnelAutostart(configId, autoStart)
    const index = configs.value.findIndex(c => c.id === configId)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }
```

- [ ] **Step 2: 导出新方法**

在 return 中添加：

```typescript
    setAutoStart,
```

- [ ] **Step 3: 提交变更**

```bash
git add src/stores/config.ts
git commit -m "feat(store): config store 新增 setAutoStart"
```

---

### Task 10: ConfigForm 新增开关

**Files:**
- Modify: `src/components/ConfigForm.vue`

- [ ] **Step 1: 在表单数据添加 autoStart**

修改 `src/components/ConfigForm.vue`：

1. 在 defaultFormData 函数中添加：

```typescript
const defaultFormData = () => ({
  // ... 现有字段 ...
  autoStart: false
})
```

2. 在 fillFormData 函数中添加：

```typescript
function fillFormData(config: Config | null | undefined) {
  if (config) {
    formData.value = {
      // ... 现有字段 ...
      autoStart: config.autoStart
    }
  } else {
    // ... 新建模式 ...
  }
}
```

- [ ] **Step 2: 在请求构建中添加字段**

在 handleSubmit 函数的 requestData 构建中添加：

```typescript
    const requestData: CreateConfigRequest | UpdateConfigRequest = {
      // ... 现有字段 ...
      autoStart: formData.value.autoStart
    }
```

- [ ] **Step 3: 在模板添加开关组件**

在"高级选项"区域，isFavorite 开关下方添加：

```vue
      <t-form-item label="开机启动" name="autoStart">
        <t-switch v-model="formData.autoStart" />
      </t-form-item>
```

- [ ] **Step 4: 提交变更**

```bash
git add src/components/ConfigForm.vue
git commit -m "feat(ConfigForm): 新增开机启动开关"
```

---

### Task 11: TunnelCard 新增按钮

**Files:**
- Modify: `src/components/TunnelCard.vue`

- [ ] **Step 1: 导入 PowerIcon**

在 script setup 的 import 区域添加：

```typescript
import { PowerIcon } from 'tdesign-icons-vue-next'
```

- [ ] **Step 2: 在卡片头部添加按钮**

在模板的 card-header 区域，收藏按钮左侧添加：

```vue
        <!-- 开机启动开关 -->
        <t-button
          :class="['autostart-btn', { 'is-autostart': config.autoStart }]"
          variant="text"
          shape="circle"
          size="small"
          @click.stop="handleToggleAutoStart"
        >
          <template #icon>
            <PowerIcon />
          </template>
        </t-button>
```

位置：在 `<t-tag>` 和 `<span class="config-name">` 之后，收藏按钮之前。

- [ ] **Step 3: 添加处理函数**

在 script setup 中添加：

```typescript
async function handleToggleAutoStart() {
  try {
    await configStore.setAutoStart(props.config.id, !props.config.autoStart)
  } catch (error) {
    console.error('切换开机启动状态失败:', error)
  }
}
```

- [ ] **Step 4: 添加样式**

在 style 区域添加：

```css
.autostart-btn {
  color: var(--td-text-color-placeholder);
  transition: color 0.2s;
  cursor: pointer;
}

.autostart-btn:hover {
  color: var(--td-brand-color);
}

.autostart-btn.is-autostart {
  color: var(--td-success-color);
}
```

- [ ] **Step 5: 提交变更**

```bash
git add src/components/TunnelCard.vue
git commit -m "feat(TunnelCard): 新增开机启动快捷按钮"
```

---

### Task 12: 最终验证

- [ ] **Step 1: 验证后端编译**

```bash
cd src-tauri && cargo build
```

Expected: 编译成功，无错误

- [ ] **Step 2: 验证前端编译**

```bash
pnpm build
```

Expected: 编译成功，无错误

- [ ] **Step 3: 提交最终变更**

```bash
git status
git add -A
git commit -m "feat: 开机启动功能完整实现"
```

---

## 自检清单

| 需求 | 任务覆盖 |
|------|----------|
| Config 新增 auto_start 字段 | Task 1, 2, 7 |
| 数据库迁移 | Task 2 |
| 软件开机启动（注册表） | Task 3, 4, 5 |
| 托盘菜单勾选控制 | Task 5 |
| 隧道开机启动（setup阶段） | Task 6 |
| 重试机制（10次，1分钟间隔） | Task 6 |
| ConfigForm 开关 | Task 10 |
| TunnelCard 按钮 | Task 11 |
| API 封装 | Task 8 |
| Store 集成 | Task 9 |