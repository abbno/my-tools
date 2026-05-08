# 常用隧道功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 SSH 隧道管理器增加常用隧道功能，支持标记、筛选、拖拽排序。

**Architecture:** 在 Config 模型新增 is_favorite/favorite_order 字段，后端新增 3 个 API，前端侧边栏新增常用区域、卡片新增收藏按钮、表单新增复选框。

**Tech Stack:** Rust (rusqlite, serde), Vue 3 (Pinia, TDesign), TypeScript

---

## 文件结构概览

| 文件 | 变更类型 | 职责 |
|------|----------|------|
| `src-tauri/src/models/config.rs` | 修改 | 新增 is_favorite、favorite_order 字段 |
| `src-tauri/src/db/mod.rs` | 修改 | 数据库表结构新增两列 |
| `src-tauri/src/db/sqlite.rs` | 修改 | 新增常用相关查询方法 |
| `src-tauri/src/commands/config.rs` | 修改 | 新增 get_favorites、set_favorite、reorder_favorites |
| `src-tauri/src/lib.rs` | 修改 | 注册新命令 |
| `src/types/index.ts` | 修改 | Config 接口新增字段 |
| `src/api/tauri.ts` | 修改 | 新增 API 方法 |
| `src/stores/config.ts` | 修改 | 新增 favorites 状态和方法 |
| `src/components/Sidebar.vue` | 修改 | 新增常用隧道区域 |
| `src/components/TunnelCard.vue` | 修改 | 新增收藏按钮 |
| `src/components/ConfigForm.vue` | 修改 | 新增常用复选框 |

---

### Task 1: 后端模型层 - Config 结构体新增字段

**Files:**
- Modify: `src-tauri/src/models/config.rs:5-25`

- [ ] **Step 1: 修改 Config 结构体**

在 Config 结构体中新增 is_favorite 和 favorite_order 字段：

```rust
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
    pub is_favorite: bool,           // 新增：是否为常用
    pub favorite_order: i32,         // 新增：常用排序序号
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

- [ ] **Step 2: 修改 CreateConfigRequest 结构体**

新增 is_favorite 字段（favorite_order 由后端自动管理）：

```rust
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
    pub is_favorite: Option<bool>,   // 新增：可选，默认 false
}
```

- [ ] **Step 3: 修改 UpdateConfigRequest 结构体**

同样新增 is_favorite 字段：

```rust
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
    pub is_favorite: Option<bool>,   // 新增：可选
}
```

- [ ] **Step 4: 提交变更**

```bash
git add src-tauri/src/models/config.rs
git commit -m "feat(models): Config 新增 is_favorite 和 favorite_order 字段"
```

---

### Task 2: 后端数据库层 - 表结构和查询方法

**Files:**
- Modify: `src-tauri/src/db/mod.rs:26-46`
- Modify: `src-tauri/src/db/sqlite.rs`

- [ ] **Step 1: 更数据库表结构**

在 configs 表定义中新增两列：

```rust
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
    is_favorite INTEGER DEFAULT 0,           -- 新增
    favorite_order INTEGER DEFAULT 0,        -- 新增
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id)
);
```

- [ ] **Step 2: 修改 map_config_row 函数**

更新行映射以包含新字段（在第 97 行附近）：

```rust
fn map_config_row(row: &Row) -> Result<Config, rusqlite::Error> {
    let auth_type_str: String = row.get(6)?;
    let tunnel_type_str: String = row.get(10)?;

    // 解密密码
    let password: Option<String> = row.get::<_, Option<String>>(7)?;
    let decrypted_password = match password {
        Some(ref encrypted) => crypto::decrypt(encrypted).ok(),
        None => None,
    };

    // 解密密钥密码
    let key_passphrase: Option<String> = row.get::<_, Option<String>>(9)?;
    let decrypted_key_passphrase = match key_passphrase {
        Some(ref encrypted) => crypto::decrypt(encrypted).ok(),
        None => None,
    };

    Ok(Config {
        id: row.get(0)?,
        name: row.get(1)?,
        group_id: row.get(2)?,
        host: row.get(3)?,
        port: row.get(4)?,
        username: row.get(5)?,
        auth_type: parse_auth_type(&auth_type_str),
        password: decrypted_password,
        key_path: row.get(8)?,
        key_passphrase: decrypted_key_passphrase,
        tunnel_type: parse_tunnel_type(&tunnel_type_str),
        local_host: row.get(11)?,
        local_port: row.get(12)?,
        remote_host: row.get(13)?,
        remote_port: row.get(14)?,
        auto_reconnect: row.get::<_, i32>(15)? != 0,
        reconnect_interval: row.get(16)?,
        is_favorite: row.get::<_, i32>(19)? != 0,      // 新增
        favorite_order: row.get(20)?,                   // 新增
        created_at: parse_datetime(&row.get::<_, String>(17)?),
        updated_at: parse_datetime(&row.get::<_, String>(18)?),
    })
}
```

- [ ] **Step 3: 更新所有 SELECT 语句**

在 get_configs、get_config_by_id、get_configs_by_group、search_configs 的 SELECT 语句中添加新字段：

```sql
SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
       tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
       reconnect_interval, created_at, updated_at, is_favorite, favorite_order
FROM configs
```

- [ ] **Step 4: 更新 save_config 函数的 INSERT 和 UPDATE**

INSERT 语句：

```rust
conn.execute(
    "INSERT INTO configs (
        id, name, group_id, host, port, username, auth_type, password,
        key_path, key_passphrase, tunnel_type, local_host, local_port,
        remote_host, remote_port, auto_reconnect, reconnect_interval,
        is_favorite, favorite_order, created_at, updated_at
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
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
        if config.is_favorite { 1 } else { 0 },
        config.favorite_order,
        config.created_at.to_rfc3339(),
        config.updated_at.to_rfc3339(),
    ],
)?;
```

UPDATE 语句：

```rust
conn.execute(
    "UPDATE configs SET
        name = ?1, group_id = ?2, host = ?3, port = ?4, username = ?5,
        auth_type = ?6, password = ?7, key_path = ?8, key_passphrase = ?9,
        tunnel_type = ?10, local_host = ?11, local_port = ?12,
        remote_host = ?13, remote_port = ?14, auto_reconnect = ?15,
        reconnect_interval = ?16, is_favorite = ?17, favorite_order = ?18,
        updated_at = ?19
    WHERE id = ?20",
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
        if config.is_favorite { 1 } else { 0 },
        config.favorite_order,
        config.updated_at.to_rfc3339(),
        config.id,
    ],
)?;
```

- [ ] **Step 5: 新增 get_favorites 函数**

```rust
/// 获取常用配置列表（按 favorite_order 排序）
pub fn get_favorites() -> Result<Vec<Config>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(
        "SELECT id, name, group_id, host, port, username, auth_type, password, key_path, key_passphrase,
                tunnel_type, local_host, local_port, remote_host, remote_port, auto_reconnect,
                reconnect_interval, created_at, updated_at, is_favorite, favorite_order
         FROM configs
         WHERE is_favorite = 1
         ORDER BY favorite_order ASC"
    )?;

    let configs = stmt.query_map([], map_config_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(configs)
}
```

- [ ] **Step 6: 新增 set_favorite 函数**

```rust
/// 设置配置的常用状态
pub fn set_favorite(config_id: &str, is_favorite: bool) -> Result<Config, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    if is_favorite {
        // 标记为常用：获取当前最大 order，设置为 max + 1
        let max_order: i32 = conn.query_row(
            "SELECT COALESCE(MAX(favorite_order), 0) FROM configs WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )?;

        conn.execute(
            "UPDATE configs SET is_favorite = 1, favorite_order = ?1 WHERE id = ?2",
            params![max_order + 1, config_id],
        )?;
    } else {
        // 取消常用：先获取该配置的 order，用于后续重排
        let old_order: i32 = conn.query_row(
            "SELECT favorite_order FROM configs WHERE id = ?1",
            params![config_id],
            |row| row.get(0),
        )?;

        // 设置为非常用
        conn.execute(
            "UPDATE configs SET is_favorite = 0, favorite_order = 0 WHERE id = ?1",
            params![config_id],
        )?;

        // 重排其他常用项（填补空缺）
        conn.execute(
            "UPDATE configs SET favorite_order = favorite_order - 1
             WHERE is_favorite = 1 AND favorite_order > ?1",
            params![old_order],
        )?;
    }

    // 返回更新后的配置
    get_config_by_id(config_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("配置不存在".to_string())
    })
}
```

- [ ] **Step 7: 新增 reorder_favorites 函数**

```rust
/// 批量更新常用配置的排序
pub fn reorder_favorites(orders: &[(String, i32)]) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    for (config_id, order) in orders {
        conn.execute(
            "UPDATE configs SET favorite_order = ?1 WHERE id = ?2 AND is_favorite = 1",
            params![order, config_id],
        )?;
    }

    Ok(())
}
```

- [ ] **Step 8: 提交变更**

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/db/sqlite.rs
git commit -m "feat(db): 数据库新增常用隧道相关字段和方法"
```

---

### Task 3: 后端命令层 - 新增 Tauri Command

**Files:**
- Modify: `src-tauri/src/commands/config.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 修改 ConfigDto 结构体**

在 ConfigDto 中新增字段：

```rust
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
    pub is_favorite: bool,           // 新增
    pub favorite_order: i32,         // 新增
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 2: 修改 ConfigDto 的 From 实现**

```rust
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
            is_favorite: config.is_favorite,         // 新增
            favorite_order: config.favorite_order,   // 新增
            created_at: config.created_at.to_rfc3339(),
            updated_at: config.updated_at.to_rfc3339(),
        }
    }
}
```

- [ ] **Step 3: 修改 save_config 命令**

更新创建配置时设置新字段：

```rust
#[tauri::command]
pub fn save_config(request: CreateConfigRequest) -> Result<ConfigDto, String> {
    let now = Utc::now();
    let is_favorite = request.is_favorite.unwrap_or(false);

    // 如果标记为常用，需要计算 favorite_order
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
        created_at: now,
        updated_at: now,
    };

    db::save_config(&config).map_err(|e| e.to_string())?;
    Ok(ConfigDto::from(config))
}
```

需要在 sqlite.rs 中添加 get_max_favorite_order 函数：

```rust
/// 获取当前最大常用排序号
pub fn get_max_favorite_order() -> Result<i32, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.query_row(
        "SELECT COALESCE(MAX(favorite_order), 0) FROM configs WHERE is_favorite = 1",
        [],
        |row| row.get(0),
    )
}
```

- [ ] **Step 4: 修改 update_config 命令**

```rust
#[tauri::command]
pub fn update_config(request: UpdateConfigRequest) -> Result<ConfigDto, String> {
    let existing = db::get_config_by_id(&request.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "配置不存在".to_string())?;

    let now = Utc::now();

    // 处理常用状态变更
    let (is_favorite, favorite_order) = match request.is_favorite {
        Some(true) if !existing.is_favorite => {
            // 新标记为常用
            let order = db::get_max_favorite_order().map_err(|e| e.to_string())? + 1;
            (true, order)
        }
        Some(false) if existing.is_favorite => {
            // 取消常用
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
        created_at: existing.created_at,
        updated_at: now,
    };

    db::save_config(&config).map_err(|e| e.to_string())?;

    // 如果取消常用，需要重排其他常用项
    if request.is_favorite == Some(false) && existing.is_favorite {
        db::reorder_after_remove_favorite(existing.favorite_order)
            .map_err(|e| e.to_string())?;
    }

    Ok(ConfigDto::from(config))
}
```

需要在 sqlite.rs 中添加：

```rust
/// 取消常用后重排其他常用项
pub fn reorder_after_remove_favorite(removed_order: i32) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "UPDATE configs SET favorite_order = favorite_order - 1
         WHERE is_favorite = 1 AND favorite_order > ?1",
        params![removed_order],
    )?;

    Ok(())
}
```

- [ ] **Step 5: 新增 get_favorites 命令**

```rust
/// 获取常用配置列表
#[tauri::command]
pub fn get_favorites() -> Result<Vec<ConfigDto>, String> {
    let configs = db::get_favorites().map_err(|e| e.to_string())?;
    Ok(configs.into_iter().map(ConfigDto::from).collect())
}
```

- [ ] **Step 6: 新增 set_favorite 命令**

```rust
/// 设置配置的常用状态
#[tauri::command]
pub fn set_favorite(config_id: String, is_favorite: bool) -> Result<ConfigDto, String> {
    let config = db::set_favorite(&config_id, is_favorite).map_err(|e| e.to_string())?;
    Ok(ConfigDto::from(config))
}
```

- [ ] **Step 7: 新增 reorder_favorites 命令**

```rust
/// 批量更新常用配置排序
#[tauri::command]
pub fn reorder_favorites(orders: Vec<(String, i32)>) -> Result<(), String> {
    db::reorder_favorites(&orders).map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 8: 在 lib.rs 注册新命令**

在 lib.rs 的 invoke_handler 中添加新命令：

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // 分组命令
            commands::group::get_groups,
            commands::group::save_group,
            commands::group::delete_group,
            // 配置命令
            commands::config::get_configs,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::update_config,
            commands::config::delete_config,
            commands::config::search_configs,
            commands::config::export_configs,
            commands::config::import_configs,
            commands::config::get_favorites,        // 新增
            commands::config::set_favorite,         // 新增
            commands::config::reorder_favorites,    // 新增
            // 隧道命令
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::restart_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::get_running_tunnels_cmd,
            // 日志命令
            commands::log::get_logs,
            commands::log::clear_logs,
            commands::log::clear_all_logs,
            commands::log::cleanup_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 9: 提交变更**

```bash
git add src-tauri/src/commands/config.rs src-tauri/src/lib.rs
git commit -m "feat(commands): 新增常用隧道相关命令"
```

---

### Task 4: 前端类型定义 - TypeScript 接口更新

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: 修改 Config 接口**

```typescript
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
  isFavorite: boolean        // 新增
  favoriteOrder: number      // 新增
  createdAt: string
  updatedAt: string
}
```

- [ ] **Step 2: 修改 CreateConfigRequest 接口**

```typescript
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
  isFavorite?: boolean       // 新增：可选
}
```

- [ ] **Step 3: 修改 UpdateConfigRequest 接口**

```typescript
export interface UpdateConfigRequest extends CreateConfigRequest {
  id: string
  isFavorite?: boolean       // 新增：可选
}
```

- [ ] **Step 4: 提交变更**

```bash
git add src/types/index.ts
git commit -m "feat(types): Config 接口新增 isFavorite 和 favoriteOrder"
```

---

### Task 5: 前端 API 层 - 新增 API 方法

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 修改 ConfigDto 接口**

```typescript
interface ConfigDto {
  id: string
  name: string
  group_id: string | null
  host: string
  port: number
  username: string
  auth_type: string
  password: string | null
  key_path: string | null
  key_passphrase: string | null
  tunnel_type: string
  local_host: string
  local_port: number
  remote_host: string | null
  remote_port: number | null
  auto_reconnect: boolean
  reconnect_interval: number
  is_favorite: boolean          // 新增
  favorite_order: number        // 新增
  created_at: string
  updated_at: string
}
```

- [ ] **Step 2: 修改 configDtoToConfig 函数**

```typescript
function configDtoToConfig(dto: ConfigDto): Config {
  return {
    id: dto.id,
    name: dto.name,
    groupId: dto.group_id,
    host: dto.host,
    port: dto.port,
    username: dto.username,
    authType: dto.auth_type as Config['authType'],
    password: dto.password,
    keyPath: dto.key_path,
    keyPassphrase: dto.key_passphrase,
    tunnelType: dto.tunnel_type as Config['tunnelType'],
    localHost: dto.local_host,
    localPort: dto.local_port,
    remoteHost: dto.remote_host,
    remotePort: dto.remote_port,
    autoReconnect: dto.auto_reconnect,
    reconnectInterval: dto.reconnect_interval,
    isFavorite: dto.is_favorite,          // 新增
    favoriteOrder: dto.favorite_order,    // 新增
    createdAt: dto.created_at,
    updatedAt: dto.updated_at,
  }
}
```

- [ ] **Step 3: 修改 createConfigRequestToDto 函数**

```typescript
function createConfigRequestToDto(req: CreateConfigRequest): Record<string, unknown> {
  return {
    name: req.name,
    group_id: req.groupId,
    host: req.host,
    port: req.port,
    username: req.username,
    auth_type: req.authType,
    password: req.password,
    key_path: req.keyPath,
    key_passphrase: req.keyPassphrase,
    tunnel_type: req.tunnelType,
    local_host: req.localHost,
    local_port: req.localPort,
    remote_host: req.remoteHost,
    remote_port: req.remotePort,
    auto_reconnect: req.autoReconnect,
    reconnect_interval: req.reconnectInterval,
    is_favorite: req.isFavorite ?? false,   // 新增
  }
}
```

- [ ] **Step 4: 新增 getFavorites 函数**

```typescript
/**
 * 获取常用配置列表
 */
export async function getFavorites(): Promise<Config[]> {
  const dtos = await invoke<ConfigDto[]>('get_favorites')
  return dtos.map(configDtoToConfig)
}
```

- [ ] **Step 5: 新增 setFavorite 函数**

```typescript
/**
 * 设置配置的常用状态
 */
export async function setFavorite(configId: string, isFavorite: boolean): Promise<Config> {
  const dto = await invoke<ConfigDto>('set_favorite', { configId, isFavorite })
  return configDtoToConfig(dto)
}
```

- [ ] **Step 6: 新增 reorderFavorites 函数**

```typescript
/**
 * 批量更新常用配置排序
 * @param orders 排序数据数组，每项包含 configId 和 order
 */
export async function reorderFavorites(orders: { configId: string; order: number }[]): Promise<void> {
  // 转换为后端期望的元组格式
  const ordersTuple = orders.map(o => [o.configId, o.order])
  await invoke('reorder_favorites', { orders: ordersTuple })
}
```

- [ ] **Step 7: 提交变更**

```bash
git add src/api/tauri.ts
git commit -m "feat(api): 新增常用隧道相关 API 方法"
```

---

### Task 6: 前端状态管理 - Store 更新

**Files:**
- Modify: `src/stores/config.ts`

- [ ] **Step 1: 新增 favorites 状态和方法**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Config, CreateConfigRequest, UpdateConfigRequest } from '@/types'
import * as api from '@/api/tauri'

export const useConfigStore = defineStore('config', () => {
  const configs = ref<Config[]>([])
  const loading = ref(false)
  const selectedGroupId = ref<string | null>(null)
  const selectedFavoriteId = ref<string | null>(null)  // 新增：选中的常用项

  // 过滤后的配置列表
  const filteredConfigs = computed(() => {
    // 如果选中了常用项（单个）
    if (selectedFavoriteId.value) {
      return configs.value.filter(c => c.id === selectedFavoriteId.value)
    }

    // 如果选中了"常用"标题（显示全部常用）
    if (selectedFavoriteId.value === 'all') {
      return configs.value.filter(c => c.isFavorite)
                     .sort((a, b) => a.favoriteOrder - b.favoriteOrder)
    }

    // 按分组筛选
    if (selectedGroupId.value) {
      return configs.value.filter(c => c.groupId === selectedGroupId.value)
    }

    return configs.value
  })

  // 常用配置列表（排序后）
  const favorites = computed(() => {
    return configs.value
      .filter(c => c.isFavorite)
      .sort((a, b) => a.favoriteOrder - b.favoriteOrder)
  })

  // 获取配置列表
  async function fetchConfigs(groupId?: string) {
    loading.value = true
    try {
      configs.value = await api.getConfigs(groupId)
    } finally {
      loading.value = false
    }
  }

  // 创建配置
  async function createConfig(request: CreateConfigRequest) {
    const config = await api.saveConfig(request)
    configs.value.push(config)
    return config
  }

  // 更新配置
  async function updateConfig(request: UpdateConfigRequest) {
    const config = await api.updateConfig(request)
    const index = configs.value.findIndex(c => c.id === request.id)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }

  // 删除配置
  async function removeConfig(id: string) {
    await api.deleteConfig(id)
    configs.value = configs.value.filter(c => c.id !== id)
  }

  // 搜索配置
  async function search(keyword: string) {
    loading.value = true
    try {
      configs.value = await api.searchConfigs(keyword)
    } finally {
      loading.value = false
    }
  }

  // 设置选中分组
  function setSelectedGroup(groupId: string | null) {
    selectedGroupId.value = groupId
    selectedFavoriteId.value = null  // 清除常用筛选
  }

  // 新增：设置常用筛选
  function setSelectedFavorite(favoriteId: string | null) {
    selectedFavoriteId.value = favoriteId
    selectedGroupId.value = null  // 清除分组筛选
  }

  // 新增：设置常用状态
  async function setFavorite(configId: string, isFavorite: boolean) {
    const config = await api.setFavorite(configId, isFavorite)
    const index = configs.value.findIndex(c => c.id === configId)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }

  // 新增：重排常用顺序
  async function reorderFavorites(orders: { configId: string; order: number }[]) {
    await api.reorderFavorites(orders)
    // 更新本地状态
    for (const { configId, order } of orders) {
      const index = configs.value.findIndex(c => c.id === configId)
      if (index !== -1) {
        configs.value[index].favoriteOrder = order
      }
    }
  }

  return {
    configs,
    loading,
    selectedGroupId,
    selectedFavoriteId,
    filteredConfigs,
    favorites,
    fetchConfigs,
    createConfig,
    updateConfig,
    removeConfig,
    search,
    setSelectedGroup,
    setSelectedFavorite,    // 新增
    setFavorite,            // 新增
    reorderFavorites,       // 新增
  }
})
```

- [ ] **Step 2: 提交变更**

```bash
git add src/stores/config.ts
git commit -m "feat(store): config store 新增常用隧道状态和方法"
```

---

### Task 7: 前端侧边栏 - 常用隧道区域

**Files:**
- Modify: `src/components/Sidebar.vue`

- [ ] **Step 1: 导入新图标和 store**

在 script setup 中添加：

```typescript
import { StarFilledIcon, StarIcon } from 'tdesign-icons-vue-next'
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()
```

- [ ] **Step 2: 添加常用区域模板**

在分组标题之前插入常用区域：

```vue
<template>
  <div class="sidebar">
    <!-- 常用隧道区域 -->
    <div class="favorites-section">
      <div
        class="favorites-header"
        :class="{ active: configStore.selectedFavoriteId === 'all' }"
        @click="selectFavorite('all')"
      >
        <StarFilledIcon class="star-icon" />
        <span class="favorites-title">常用</span>
        <span class="favorites-count">{{ configStore.favorites.length }}</span>
      </div>

      <!-- 常用隧道列表（支持拖拽） -->
      <div
        v-for="config in configStore.favorites"
        :key="config.id"
        class="favorite-item"
        :class="{ active: configStore.selectedFavoriteId === config.id }"
        draggable="true"
        @click="selectFavorite(config.id)"
        @dragstart="handleDragStart($event, config)"
        @dragover.prevent
        @drop="handleDrop($event, config)"
      >
        <span class="favorite-name">{{ config.name }}</span>
      </div>
    </div>

    <!-- 分组区域 -->
    <div class="sidebar-header">
      <span class="sidebar-title">分组</span>
      <t-button
        variant="text"
        shape="circle"
        size="small"
        @click="showCreateDialog = true"
      >
        <template #icon>
          <AddIcon />
        </template>
      </t-button>
    </div>
    <!-- ... 其余代码不变 ... -->
  </div>
</template>
```

- [ ] **Step 3: 添加拖拽处理函数**

```typescript
// 拖拽状态
const draggedItem = ref<Config | null>(null)

// 选择常用筛选
function selectFavorite(favoriteId: string | null): void {
  configStore.setSelectedFavorite(favoriteId)
}

// 拖拽开始
function handleDragStart(event: DragEvent, config: Config): void {
  draggedItem.value = config
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
  }
}

// 拖拽放置
async function handleDrop(event: DragEvent, targetConfig: Config): void {
  if (!draggedItem.value || draggedItem.value.id === targetConfig.id) {
    draggedItem.value = null
    return
  }

  // 计算新顺序
  const favorites = configStore.favorites
  const draggedIndex = favorites.findIndex(f => f.id === draggedItem.value!.id)
  const targetIndex = favorites.findIndex(f => f.id === targetConfig.id)

  if (draggedIndex === -1 || targetIndex === -1) {
    draggedItem.value = null
    return
  }

  // 重新计算所有常用项的 order
  const newOrders: { configId: string; order: number }[] = []
  const reorderedFavorites = [...favorites]
  reorderedFavorites.splice(draggedIndex, 1)
  reorderedFavorites.splice(targetIndex, 0, draggedItem.value!)

  reorderedFavorites.forEach((config, index) => {
    newOrders.push({ configId: config.id, order: index + 1 })
  })

  // 调用 API 更新顺序
  try {
    await configStore.reorderFavorites(newOrders)
  } catch (error) {
    console.error('重排常用失败:', error)
  }

  draggedItem.value = null
}
```

- [ ] **Step 4: 添加样式**

```css
/* 常用区域样式 */
.favorites-section {
  padding: 8px 0;
  border-bottom: 1px solid var(--td-component-border);
}

.favorites-header {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.favorites-header:hover {
  background-color: var(--td-bg-color-container-hover);
}

.favorites-header.active {
  background-color: var(--td-brand-color-light);
}

.star-icon {
  color: var(--td-brand-color);
  margin-right: 8px;
}

.favorites-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.favorites-header.active .favorites-title {
  color: var(--td-brand-color);
}

.favorites-count {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background-color: var(--td-bg-color-secondarycontainer);
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: auto;
  min-width: 20px;
  text-align: center;
}

.favorites-header.active .favorites-count {
  background-color: var(--td-brand-color-focus);
  color: var(--td-brand-color);
}

.favorite-item {
  display: flex;
  align-items: center;
  padding: 8px 16px 8px 32px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.favorite-item:hover {
  background-color: var(--td-bg-color-container-hover);
}

.favorite-item.active {
  background-color: var(--td-brand-color-light);
}

.favorite-name {
  font-size: 13px;
  color: var(--td-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.favorite-item.active .favorite-name {
  color: var(--td-brand-color);
}
```

- [ ] **Step 5: 提交变更**

```bash
git add src/components/Sidebar.vue
git commit -m "feat(ui): 侧边栏新增常用隧道区域"
```

---

### Task 8: 前端隧道卡片 - 收藏按钮

**Files:**
- Modify: `src/components/TunnelCard.vue`

- [ ] **Step 1: 导入图标和 store**

```typescript
import { StarFilledIcon, StarIcon } from 'tdesign-icons-vue-next'
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()
```

- [ ] **Step 2: 添加收藏按钮到卡片头部**

修改 card-header 部分：

```vue
<div class="card-header">
  <t-tag :theme="tunnelTypeTheme" variant="light" size="small">
    {{ tunnelTypeLabel }}
  </t-tag>
  <span class="config-name">{{ config.name }}</span>

  <!-- 收藏按钮 -->
  <t-button
    class="favorite-btn"
    variant="text"
    shape="circle"
    size="small"
    @click.stop="handleToggleFavorite"
  >
    <template #icon>
      <StarFilledIcon v-if="config.isFavorite" />
      <StarIcon v-else />
    </template>
  </t-button>
</div>
```

- [ ] **Step 3: 添加收藏切换函数**

```typescript
// 切换收藏状态
async function handleToggleFavorite() {
  try {
    await configStore.setFavorite(props.config.id, !props.config.isFavorite)
  } catch (error) {
    console.error('切换收藏状态失败:', error)
  }
}
```

- [ ] **Step 4: 添加样式**

```css
.favorite-btn {
  margin-left: auto;
  color: var(--td-text-color-placeholder);
  transition: color 0.2s;
}

.favorite-btn:hover {
  color: var(--td-brand-color);
}

/* 已收藏状态 */
.card-header :deep(.t-button.favorite-btn) {
  color: var(--td-brand-color);
}
```

- [ ] **Step 5: 提交变更**

```bash
git add src/components/TunnelCard.vue
git commit -m "feat(ui): 隧道卡片新增收藏按钮"
```

---

### Task 9: 前端配置表单 - 常用复选框

**Files:**
- Modify: `src/components/ConfigForm.vue`

- [ ] **Step 1: 在表单数据中添加 isFavorite**

修改 defaultFormData 函数：

```typescript
const defaultFormData = () => ({
  name: '',
  groupId: null as string | null,
  host: '',
  port: 22,
  username: '',
  authType: 'password' as AuthType,
  password: '',
  keyPath: '',
  keyPassphrase: '',
  tunnelType: 'local' as TunnelType,
  localHost: '127.0.0.1',
  localPort: 8080,
  remoteHost: 'localhost',
  remotePort: 80,
  autoReconnect: false,
  reconnectInterval: 10,
  isFavorite: false,    // 新增
})
```

- [ ] **Step 2: 在高级选项区域添加复选框**

在 autoReconnect 之前添加：

```vue
<!-- 高级选项 -->
<t-divider>高级选项</t-divider>

<t-form-item label="标记为常用" name="isFavorite">
  <t-switch v-model="formData.isFavorite" />
</t-form-item>

<t-form-item label="自动重连" name="autoReconnect">
  <t-switch v-model="formData.autoReconnect" />
</t-form-item>
```

- [ ] **Step 3: 修改 fillFormData 函数**

```typescript
function fillFormData(config: Config | null | undefined) {
  if (config) {
    formData.value = {
      name: config.name,
      groupId: config.groupId,
      host: config.host,
      port: config.port,
      username: config.username,
      authType: config.authType,
      password: config.password || '',
      keyPath: config.keyPath || '',
      keyPassphrase: config.key_passphrase || '',
      tunnelType: config.tunnelType,
      localHost: config.localHost,
      localPort: config.localPort,
      remoteHost: config.remoteHost || 'localhost',
      remotePort: config.remotePort || 80,
      autoReconnect: config.autoReconnect,
      reconnectInterval: config.reconnectInterval,
      isFavorite: config.isFavorite,    // 新增
    }
  } else {
    formData.value = defaultFormData()
  }
}
```

- [ ] **Step 4: 修改 handleSubmit 函数**

在 requestData 中添加 isFavorite：

```typescript
const requestData: CreateConfigRequest | UpdateConfigRequest = {
  name: formData.value.name.trim(),
  groupId: formData.value.groupId,
  host: formData.value.host.trim(),
  port: formData.value.port,
  username: formData.value.username.trim(),
  authType: formData.value.authType,
  password: formData.value.authType === 'password' ? formData.value.password : null,
  keyPath: formData.value.authType === 'key' ? formData.value.keyPath.trim() : null,
  keyPassphrase: formData.value.authType === 'key' && formData.value.keyPassphrase
    ? formData.value.keyPassphrase
    : null,
  tunnelType: formData.value.tunnelType,
  localHost: formData.value.localHost.trim(),
  localPort: formData.value.localPort,
  remoteHost: formData.value.tunnelType !== 'dynamic' ? formData.value.remoteHost?.trim() || null : null,
  remotePort: formData.value.tunnelType !== 'dynamic' ? formData.value.remotePort : null,
  autoReconnect: formData.value.autoReconnect,
  reconnectInterval: formData.value.reconnectInterval,
  isFavorite: formData.value.isFavorite,    // 新增
}
```

- [ ] **Step 5: 提交变更**

```bash
git add src/components/ConfigForm.vue
git commit -m "feat(ui): 配置表单新增常用复选框"
```

---

### Task 10: 验证和测试

**Files:**
- 无新增文件，测试现有功能

- [ ] **Step 1: 编译后端代码**

```bash
cd src-tauri && cargo build
```

Expected: 编译成功，无错误

- [ ] **Step 2: 编译前端代码**

```bash
npm run build
```

Expected: 编译成功，无 TypeScript 错误

- [ ] **Step 3: 运行应用并手动测试**

启动应用后验证：

1. 侧边栏是否显示"常用"区域
2. 隧道卡片是否显示收藏按钮
3. 点击收藏按钮是否能标记/取消标记
4. 配置表单是否有"标记为常用"选项
5. 侧边栏常用项是否可拖拽排序
6. 点击常用项是否能筛选显示

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat: 完成常用隧道功能实现"
```

---

## 自检清单

| 检查项 | 状态 |
|--------|------|
| Config 模型新增 is_favorite、favorite_order | 已覆盖 (Task 1) |
| 数据库表结构更新 | 已覆盖 (Task 2) |
| get_favorites API | 已覆盖 (Task 3) |
| set_favorite API | 已覆盖 (Task 3) |
| reorder_favorites API | 已覆盖 (Task 3) |
| 前端类型定义更新 | 已覆盖 (Task 4) |
| 前端 API 方法 | 已覆盖 (Task 5) |
| Store 状态管理 | 已覆盖 (Task 6) |
| 侧边栏常用区域 | 已覆盖 (Task 7) |
| 隧道卡片收藏按钮 | 已覆盖 (Task 8) |
| 配置表单复选框 | 已覆盖 (Task 9) |
| 拖拽排序功能 | 已覆盖 (Task 7) |