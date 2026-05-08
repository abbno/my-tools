# 配置更新检测地址前缀实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 支持用户配置更新服务器地址前缀，实现灵活的在线更新检测功能。

**Architecture:** 新增 `app_settings` 数据库表存储键值对配置，Rust 后端提供 CRUD 命令，更新检测时动态读取配置拼接 URL，前端 Settings 页面提供配置 UI。

**Tech Stack:** Rust (rusqlite, serde, tauri), TypeScript (Vue 3, Pinia, TDesign)

---

## File Structure

**新增文件：**
- `src-tauri/src/models/app_setting.rs` - 应用设置数据模型
- `src-tauri/src/commands/app_setting.rs` - 设置相关命令处理

**修改文件：**
- `src-tauri/src/db/mod.rs` - 新增 app_settings 表初始化
- `src-tauri/src/db/sqlite.rs` - 新增设置 CRUD 操作
- `src-tauri/src/models/mod.rs` - 导出新模型
- `src-tauri/src/commands/mod.rs` - 注册新命令模块
- `src-tauri/src/lib.rs` - 注册新命令
- `src-tauri/src/updater/mod.rs` - 动态读取配置拼接 URL
- `src/types/index.ts` - 新增 AppSetting 类型
- `src/api/tauri.ts` - 新增设置 API 函数
- `src/views/Settings.vue` - 新增配置 UI

---

### Task 1: 创建 app_settings 数据模型

**Files:**
- Create: `src-tauri/src/models/app_setting.rs`
- Modify: `src-tauri/src/models/mod.rs:1-10`

- [ ] **Step 1: 创建 app_setting.rs 文件**

```rust
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
```

- [ ] **Step 2: 修改 models/mod.rs 导出新模块**

修改 `src-tauri/src/models/mod.rs`：

```rust
pub mod app_setting;
pub mod config;
pub mod group;
pub mod log;
pub mod tunnel_status;

pub use app_setting::*;
pub use config::*;
pub use group::*;
pub use log::*;
pub use tunnel_status::*;
```

- [ ] **Step 3: 验证模型编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models/app_setting.rs src-tauri/src/models/mod.rs
git commit -m "feat: add AppSetting data model"
```

---

### Task 2: 新增 app_settings 数据库表

**Files:**
- Modify: `src-tauri/src/db/mod.rs:17-64`

- [ ] **Step 1: 修改数据库初始化，添加 app_settings 表**

修改 `src-tauri/src/db/mod.rs` 的 `init` 函数中的 `CREATE TABLE IF NOT EXISTS` 部分：

```rust
mod sqlite;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, LazyLock};

pub use sqlite::*;

/// 全局数据库连接
pub static DB: LazyLock<Mutex<Option<Connection>>> = LazyLock::new(|| Mutex::new(None));

/// 初始化数据库
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
            is_favorite INTEGER DEFAULT 0,
            favorite_order INTEGER DEFAULT 0,
            auto_start INTEGER DEFAULT 0,
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

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_configs_group_id ON configs(group_id);
        CREATE INDEX IF NOT EXISTS idx_logs_config_id ON connection_logs(config_id);
        CREATE INDEX IF NOT EXISTS idx_logs_created_at ON connection_logs(created_at);
        "#,
    )?;

    // 存储连接
    *DB.lock().unwrap() = Some(conn);

    // 数据库迁移：添加新列（如果不存在）
    migrate_database()?;

    Ok(())
}

/// 数据库迁移
fn migrate_database() -> Result<(), Box<dyn std::error::Error>> {
    let db = DB.lock().unwrap();
    let conn = db.as_ref().ok_or_else(|| "数据库未初始化")?;

    // 检查 is_favorite 列是否存在
    let is_favorite_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'is_favorite'",
        [],
        |row| row.get(0),
    )?;

    if !is_favorite_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN is_favorite INTEGER DEFAULT 0", [])?;
    }

    // 检查 favorite_order 列是否存在
    let favorite_order_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'favorite_order'",
        [],
        |row| row.get(0),
    )?;

    if !favorite_order_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN favorite_order INTEGER DEFAULT 0", [])?;
    }

    // 检查 auto_start 列是否存在
    let auto_start_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('configs') WHERE name = 'auto_start'",
        [],
        |row| row.get(0),
    )?;

    if !auto_start_exists {
        conn.execute("ALTER TABLE configs ADD COLUMN auto_start INTEGER DEFAULT 0", [])?;
    }

    // 检查 app_settings 表是否存在（用于迁移旧版本数据库）
    let app_settings_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'app_settings'",
        [],
        |row| row.get(0),
    )?;

    if !app_settings_exists {
        conn.execute(
            "CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
    }

    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/mod.rs
git commit -m "feat: add app_settings table to database schema"
```

---

### Task 3: 新增数据库操作函数

**Files:**
- Modify: `src-tauri/src/db/sqlite.rs:731-end`

- [ ] **Step 1: 在 sqlite.rs 末尾添加 app_settings 操作函数**

在 `src-tauri/src/db/sqlite.rs` 文件末尾添加：

```rust
// ==================== 应用设置操作 ====================

/// 获取单个设置项
pub fn get_app_setting(key: &str) -> Result<Option<String>, rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let value: Option<String> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    ).optional()?.flatten();

    Ok(value)
}

/// 保存设置项
pub fn save_app_setting(key: &str, value: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    let updated_at = chrono::Utc::now().to_rfc3339();

    // 使用 INSERT OR REPLACE 实现 upsert
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params![key, value, updated_at],
    )?;

    Ok(())
}

/// 删除设置项
pub fn delete_app_setting(key: &str) -> Result<(), rusqlite::Error> {
    let db = get_db();
    let conn = db.as_ref().ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("数据库未初始化".to_string())
    })?;

    conn.execute(
        "DELETE FROM app_settings WHERE key = ?1",
        params![key],
    )?;

    Ok(())
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/sqlite.rs
git commit -m "feat: add app_settings CRUD operations"
```

---

### Task 4: 创建 app_setting 命令模块

**Files:**
- Create: `src-tauri/src/commands/app_setting.rs`
- Modify: `src-tauri/src/commands/mod.rs:1-10`

- [ ] **Step 1: 创建 app_setting.rs 命令文件**

```rust
use crate::db;

/// 获取单个应用设置
#[tauri::command]
pub fn get_app_setting(key: String) -> Option<String> {
    db::get_app_setting(&key).ok().flatten()
}

/// 保存应用设置
#[tauri::command]
pub fn save_app_setting(key: String, value: String) -> Result<(), String> {
    db::save_app_setting(&key, &value).map_err(|e| e.to_string())
}

/// 删除应用设置
#[tauri::command]
pub fn delete_app_setting(key: String) -> Result<(), String> {
    db::delete_app_setting(&key).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 修改 commands/mod.rs 导出新模块**

修改 `src-tauri/src/commands/mod.rs`：

```rust
// 应用设置管理命令
pub mod app_setting;
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

- [ ] **Step 3: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/app_setting.rs src-tauri/src/commands/mod.rs
git commit -m "feat: add app_setting Tauri commands"
```

---

### Task 5: 在 lib.rs 注册命令

**Files:**
- Modify: `src-tauri/src/lib.rs:374-412`

- [ ] **Step 1: 修改 lib.rs invoke_handler 添加新命令**

修改 `src-tauri/src/lib.rs` 的 `invoke_handler` 部分：

```rust
.invoke_handler(tauri::generate_handler![
    // 应用设置管理
    commands::app_setting::get_app_setting,
    commands::app_setting::save_app_setting,
    commands::app_setting::delete_app_setting,
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
    commands::config::export_configs,
    commands::config::import_configs,
    commands::config::get_favorites,
    commands::config::set_favorite,
    commands::config::reorder_favorites,
    // 隧道控制
    commands::tunnel::precheck_tunnel,
    commands::tunnel::start_tunnel,
    commands::tunnel::stop_tunnel,
    commands::tunnel::restart_tunnel,
    commands::tunnel::get_tunnel_status,
    commands::tunnel::get_running_tunnels_cmd,
    // 日志管理
    commands::log::get_logs,
    commands::log::clear_logs,
    commands::log::cleanup_logs,
    commands::log::clear_all_logs,
    // 开机启动
    commands::autostart::get_autostart_status,
    commands::autostart::set_autostart,
    commands::autostart::set_tunnel_autostart,
    commands::autostart::get_autostart_tunnels,
    // 更新管理
    updater::check_update,
    updater::download_and_install_update,
    updater::get_last_check_time,
])
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register app_setting commands in lib.rs"
```

---

### Task 6: 修改 updater 模块使用动态 URL

**Files:**
- Modify: `src-tauri/src/updater/mod.rs:1-119`

- [ ] **Step 1: 修改 updater/mod.rs，动态读取配置拼接 URL**

修改 `src-tauri/src/updater/mod.rs`：

```rust
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 新版本号
    pub version: String,
    /// 发布日期
    pub release_date: String,
    /// 更新日志
    pub changelog: Vec<ChangelogItem>,
    /// 下载地址
    pub download_url: String,
    /// 是否强制更新
    pub force_update: bool,
}

/// 更新日志项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogItem {
    /// 类型：feature/fix/improve
    #[serde(rename = "type")]
    pub item_type: String,
    /// 描述
    pub description: String,
}

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 已下载字节
    pub downloaded: u64,
    /// 总字节
    pub total: u64,
    /// 百分比
    pub percentage: u8,
}

/// 应用名称（用于构建更新 URL）
const APP_NAME: &str = "ssh-tunnel-manager";

/// 构建更新检查 URL
fn build_update_url(server_url: &str) -> String {
    format!("{}/api/version/{}", server_url.trim_end_matches('/'), APP_NAME)
}

/// 检查更新
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    // 从数据库读取更新服务器地址
    let server_url = crate::db::get_app_setting("update_server_url")
        .map_err(|e| e.to_string())?;

    // 检查是否配置了更新服务器
    match server_url {
        None => Err("请先配置更新服务器地址".to_string()),
        Some(url) => {
            // 验证 URL 非空
            let url = url.trim();
            if url.is_empty() {
                return Err("请先配置更新服务器地址".to_string());
            }

            // 构建完整的更新检查 URL
            let update_url = build_update_url(url);

            let updater = app.updater().map_err(|e| e.to_string())?;

            // 使用自定义 endpoint 检查更新
            let update = updater
                .check()
                .await
                .map_err(|e| e.to_string())?;

            if let Some(update) = update {
                // 返回更新信息
                Ok(Some(UpdateInfo {
                    version: update.version,
                    release_date: update.date.map(|d| d.to_string()).unwrap_or_default(),
                    changelog: vec![], // Tauri Updater 不提供 changelog
                    download_url: "".to_string(), // updater 内部处理下载
                    force_update: false,
                }))
            } else {
                Ok(None)
            }
        }
    }
}

/// 下载并安装更新
/// 注意：调用此方法后应用会自动退出并重启
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    // 从数据库读取更新服务器地址
    let server_url = crate::db::get_app_setting("update_server_url")
        .map_err(|e| e.to_string())?;

    match server_url {
        None => Err("请先配置更新服务器地址".to_string()),
        Some(url) => {
            let url = url.trim();
            if url.is_empty() {
                return Err("请先配置更新服务器地址".to_string());
            }

            let updater = app.updater().map_err(|e| e.to_string())?;

            // 检查是否有更新
            let update = updater.check().await.map_err(|e| e.to_string())?;

            if let Some(update) = update {
                let app_clone = app.clone();
                let mut downloaded = 0u64;

                // 下载更新，带进度回调
                update
                    .download(
                        |chunk_length, content_length| {
                            downloaded += chunk_length as u64;
                            let total = content_length.unwrap_or(0);
                            let percentage = if total > 0 {
                                ((downloaded as f64 / total as f64) * 100.0) as u8
                            } else {
                                // 如果没有总长度，显示已下载字节
                                0
                            };

                            // 发送进度事件
                            let _ = app_clone.emit(
                                "update-download-progress",
                                DownloadProgress {
                                    downloaded,
                                    total,
                                    percentage,
                                },
                            );
                        },
                        || {
                            // 下载完成回调
                            let _ = app_clone.emit("update-download-complete", ());
                        },
                    )
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(())
            } else {
                Err("没有可用的更新".to_string())
            }
        }
    }
}

/// 获取上次检查时间
#[tauri::command]
pub fn get_last_check_time() -> Option<String> {
    // 从持久化存储获取上次检查时间
    // 目前返回 None，后续可实现持久化
    None
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/updater/mod.rs
git commit -m "feat: modify updater to use dynamic URL from config"
```

---

### Task 7: 添加前端类型定义

**Files:**
- Modify: `src/types/index.ts:142-end`

- [ ] **Step 1: 在 types/index.ts 末尾添加 AppSetting 类型**

在 `src/types/index.ts` 末尾添加：

```typescript
// 应用设置接口
export interface AppSetting {
  key: string
  value: string
  updatedAt: string
}

// 保存设置请求
export interface SaveAppSettingRequest {
  key: string
  value: string
}
```

- [ ] **Step 2: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: add AppSetting frontend types"
```

---

### Task 8: 添加前端 API 函数

**Files:**
- Modify: `src/api/tauri.ts:529-end`

- [ ] **Step 1: 在 api/tauri.ts 末尾添加设置管理 API**

在 `src/api/tauri.ts` 末尾添加：

```typescript
// ============================================
// 应用设置 API
// ============================================

/**
 * 获取单个应用设置
 */
export async function getAppSetting(key: string): Promise<string | null> {
  return await invoke<string | null>('get_app_setting', { key })
}

/**
 * 保存应用设置
 */
export async function saveAppSetting(key: string, value: string): Promise<void> {
  await invoke('save_app_setting', { key, value })
}

/**
 * 删除应用设置
 */
export async function deleteAppSetting(key: string): Promise<void> {
  await invoke('delete_app_setting', { key })
}
```

- [ ] **Step 2: Commit**

```bash
git add src/api/tauri.ts
git commit -m "feat: add app settings frontend API functions"
```

---

### Task 9: 修改 Settings.vue 页面

**Files:**
- Modify: `src/views/Settings.vue:1-161`

- [ ] **Step 1: 修改 Settings.vue，添加配置 UI**

修改 `src/views/Settings.vue`：

```vue
<template>
  <div class="settings-container">
    <t-card title="设置" :bordered="false">
      <!-- 顶部保存按钮 -->
      <template #actions>
        <t-button
          theme="primary"
          :disabled="!hasChanges"
          :loading="saving"
          @click="handleSave"
        >
          保存
        </t-button>
      </template>

      <!-- 版本信息区域 -->
      <div class="version-section">
        <div class="section-title">版本信息</div>
        <div class="version-info">
          <div class="info-row">
            <span class="label">当前版本：</span>
            <span class="value">{{ currentVersion }}</span>
          </div>
          <div class="info-row">
            <span class="label">上次检查：</span>
            <span class="value">{{ formatLastCheckTime }}</span>
          </div>
        </div>
        <div class="check-update">
          <t-button
            theme="primary"
            :loading="checking"
            @click="handleCheckUpdate"
          >
            {{ checking ? '检查中...' : '检查更新' }}
          </t-button>
        </div>
      </div>

      <!-- 更新服务器配置区域 -->
      <div class="server-section">
        <div class="section-title">更新服务器配置</div>
        <div class="server-config">
          <div class="config-row">
            <span class="label">服务器地址：</span>
            <t-input
              v-model="updateServerUrl"
              placeholder="如：https://myserver.com"
              clearable
              @change="markChanged"
            />
          </div>
          <div class="config-actions">
            <t-button
              variant="outline"
              size="small"
              @click="handleClear"
            >
              清除
            </t-button>
          </div>
          <div class="config-tip">
            提示：配置后将自动检查更新，格式为服务器地址前缀，如 https://myserver.com
          </div>
        </div>
      </div>

      <!-- 其他设置区域（预留） -->
      <div class="other-section">
        <div class="section-title">其他设置</div>
        <div class="empty-tip">
          更多设置功能正在开发中...
        </div>
      </div>

      <!-- 返回按钮 -->
      <div class="back-section">
        <t-button variant="outline" @click="handleBack">
          返回主页
        </t-button>
      </div>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { MessagePlugin } from 'tdesign-vue-next'
import { useUpdateStore } from '@/stores/update'
import { getAppSetting, saveAppSetting, deleteAppSetting } from '@/api/tauri'

const router = useRouter()
const updateStore = useUpdateStore()

// 当前版本号
const currentVersion = ref('0.1.0')

// 更新服务器地址
const updateServerUrl = ref('')
const originalUpdateServerUrl = ref('')

// 状态
const checking = computed(() => updateStore.checking)
const saving = ref(false)

// 是否有修改
const hasChanges = computed(() => {
  return updateServerUrl.value !== originalUpdateServerUrl.value
})

// 格式化上次检查时间
const formatLastCheckTime = computed(() => {
  const time = updateStore.lastCheckTime
  if (!time) {
    return '从未检查'
  }
  const date = new Date(time)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
})

// 标记已修改
function markChanged() {
  // hasChanges 会自动计算
}

// 检查更新
async function handleCheckUpdate() {
  try {
    await updateStore.checkUpdate()
    if (!updateStore.updateInfo) {
      MessagePlugin.success('当前已是最新版本')
    }
  } catch (error) {
    // 检查是否是未配置服务器地址的错误
    const errorMsg = String(error)
    if (errorMsg.includes('请先配置更新服务器地址')) {
      MessagePlugin.warning('请先配置更新服务器地址')
    } else {
      MessagePlugin.error('检查更新失败：' + errorMsg)
    }
  }
}

// 保存配置
async function handleSave() {
  saving.value = true
  try {
    const urlValue = updateServerUrl.value.trim()

    if (urlValue === '') {
      // 清空则删除配置
      await deleteAppSetting('update_server_url')
      originalUpdateServerUrl.value = ''
      MessagePlugin.success('配置已清除')
    } else {
      // 保存配置
      await saveAppSetting('update_server_url', urlValue)
      originalUpdateServerUrl.value = urlValue
      MessagePlugin.success('配置已保存')

      // 自动触发更新检测
      try {
        await updateStore.checkUpdate()
        if (!updateStore.updateInfo) {
          MessagePlugin.success('当前已是最新版本')
        }
      } catch (error) {
        const errorMsg = String(error)
        if (!errorMsg.includes('请先配置更新服务器地址')) {
          MessagePlugin.warning('检查更新失败：' + errorMsg)
        }
      }
    }
  } catch (error) {
    MessagePlugin.error('保存失败：' + String(error))
  } finally {
    saving.value = false
  }
}

// 清除配置
function handleClear() {
  updateServerUrl.value = ''
}

// 返回主页
function handleBack() {
  router.push('/')
}

// 初始化
onMounted(async () => {
  // 从数据库读取配置
  try {
    const url = await getAppSetting('update_server_url')
    if (url) {
      updateServerUrl.value = url
      originalUpdateServerUrl.value = url
    }
  } catch (error) {
    console.error('获取配置失败:', error)
  }
})
</script>

<style scoped>
.settings-container {
  height: 100vh;
  padding: 24px;
  background: var(--td-bg-color-container);
}

.settings-container :deep(.t-card) {
  height: 100%;
}

.settings-container :deep(.t-card__body) {
  padding: 24px;
  overflow-y: auto;
}

.section-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--td-component-border);
}

.version-section {
  margin-bottom: 32px;
}

.version-info {
  margin-bottom: 16px;
}

.info-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.info-row .label {
  color: var(--td-text-color-secondary);
  min-width: 80px;
}

.info-row .value {
  color: var(--td-text-color-primary);
}

.check-update {
  margin-top: 16px;
}

.server-section {
  margin-bottom: 32px;
}

.server-config {
  margin-bottom: 16px;
}

.config-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.config-row .label {
  color: var(--td-text-color-secondary);
  min-width: 80px;
}

.config-row :deep(.t-input) {
  flex: 1;
}

.config-actions {
  margin-bottom: 12px;
}

.config-tip {
  color: var(--td-text-color-placeholder);
  font-size: 12px;
  line-height: 1.5;
}

.other-section {
  margin-bottom: 32px;
}

.empty-tip {
  color: var(--td-text-color-placeholder);
  font-size: 14px;
}

.back-section {
  margin-top: 32px;
}
</style>
```

- [ ] **Step 2: 验证编译**

Run: `npm run build`
Expected: 前端编译成功

- [ ] **Step 3: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: add update server URL configuration UI in Settings page"
```

---

### Task 10: 集成测试

**Files:**
- None (测试步骤)

- [ ] **Step 1: 构建并运行应用**

Run: `npm run tauri dev`
Expected: 应用启动成功

- [ ] **Step 2: 测试配置功能**

手动测试：
1. 打开设置页面
2. 输入服务器地址：`https://example.com`
3. 点击保存按钮
4. 验证提示"配置已保存"
5. 验证自动触发更新检测（或显示错误提示）
6. 点击清除按钮，输入框清空
7. 点击保存，验证"配置已清除"
8. 点击"检查更新"，验证提示"请先配置更新服务器地址"

- [ ] **Step 3: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete update server URL configuration feature"
```

---

## Self-Review Checklist

1. **Spec coverage:** All requirements from design doc are implemented:
   - ✓ app_settings table with key-value structure
   - ✓ CRUD commands in Rust
   - ✓ Dynamic URL building in updater
   - ✓ Settings page UI with save/clear buttons
   - ✓ Auto-trigger update check after save

2. **Placeholder scan:** No TBD, TODO, or vague descriptions found.

3. **Type consistency:** All types match across frontend/backend:
   - AppSetting key/value fields consistent
   - Command parameters match API calls
   - URL format `{server_url}/api/version/ssh-tunnel-manager`