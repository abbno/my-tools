---
title: 开机启动功能设计
date: 2026-05-04
status: approved
---

## 需求概述

为 SSH 隧道管理器增加开机启动功能，包含两个层面：

1. **软件开机启动**：用户可在托盘菜单中设置软件是否随系统启动
2. **隧道开机启动**：用户可配置特定隧道在软件启动后自动运行

## 功能规格

| 功能 | 位置 | 交互方式 | 默认状态 |
|------|------|----------|----------|
| 软件开机启动 | 托盘菜单 | 勾选菜单项 | 关闭 |
| 隧道开机启动配置 | ConfigForm 高级选项区 | Switch 开关 | 关闭 |
| 隧道开机启动快捷操作 | TunnelCard 卡片 | 开关按钮 | 与配置同步 |

**隧道启动策略**：软件启动后立即启动标记为开机启动的隧道，失败后重试10次，间隔1分钟。

## 技术设计

### 1. 数据模型变更

**Config 结构新增字段：**

```rust
// src-tauri/src/models/config.rs
pub struct Config {
    // ... 现有字段 ...
    pub auto_start: bool,
}
```

**数据库变更：**

- `configs` 表新增 `auto_start` 列（INTEGER，默认 0）
- 迁移：`ALTER TABLE configs ADD COLUMN auto_start INTEGER DEFAULT 0`

**前端类型同步：**

```typescript
// src/types/index.ts
export interface Config {
  // ... 现有字段 ...
  autoStart: boolean
}

export interface CreateConfigRequest {
  // ... 现有字段 ...
  autoStart?: boolean
}

export interface UpdateConfigRequest {
  // ... 现有字段 ...
  autoStart?: boolean
}
```

### 2. 软件开机启动

**实现方式**：Windows 注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

**Rust 后端新增模块：**

```rust
// src-tauri/src/utils/autostart.rs
pub fn is_autostart_enabled() -> bool
pub fn enable_autostart() -> Result<(), Error>
pub fn disable_autostart() -> Result<(), Error>
```

**Tauri 命令：**

```rust
// src-tauri/src/commands/autostart.rs
#[tauri::command]
fn get_autostart_status() -> bool

#[tauri::command]
fn set_autostart(enable: bool) -> Result<(), String>
```

**托盘菜单变更：**

- 新增"开机启动"菜单项，使用 `CheckMenuItem`
- 菜单项 ID：`autostart`
- 点击时切换勾选状态并调用注册表操作

### 3. 隧道开机启动流程

**启动时机**：`lib.rs` setup 阶段，数据库初始化完成后

**启动逻辑：**

```rust
// src-tauri/src/ssh/autostart.rs
fn start_auto_start_tunnels(app: &AppHandle) {
    // 1. 查询所有 auto_start = true 的配置
    // 2. 对每个配置调用 start_ssh_tunnel
    // 3. 启动失败时加入重试队列
}
```

**重试机制：**

```rust
struct RetryTask {
    config_id: String,
    retry_count: u32,  // 最大 10
}

// tokio 异步任务，每分钟检查重试队列
// 成功后移出队列，达到10次后放弃并记录日志
```

### 4. 前端 UI 变更

**ConfigForm.vue：**

在"高级选项"区域新增开关，位置在"标记为常用"下方：

```vue
<t-form-item label="开机启动" name="autoStart">
  <t-switch v-model="formData.autoStart" />
</t-form-item>
```

**TunnelCard.vue：**

在卡片头部新增开关按钮，位置在收藏按钮左侧：

```vue
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

## 文件变更清单

| 文件 | 变更类型 |
|------|----------|
| `src-tauri/src/models/config.rs` | 新增字段 |
| `src-tauri/src/db/sqlite.rs` | 新增列映射、查询支持 |
| `src-tauri/src/db/mod.rs` | 迁移脚本 |
| `src-tauri/src/utils/autostart.rs` | 新增文件 |
| `src-tauri/src/utils/mod.rs` | 导出新模块 |
| `src-tauri/src/commands/autostart.rs` | 新增文件 |
| `src-tauri/src/commands/mod.rs` | 导出新命令 |
| `src-tauri/src/ssh/autostart.rs` | 新增文件 |
| `src-tauri/src/ssh/mod.rs` | 导出新模块 |
| `src-tauri/src/lib.rs` | 托盘菜单变更、启动流程 |
| `src/types/index.ts` | 新增字段 |
| `src/components/ConfigForm.vue` | 新增开关 |
| `src/components/TunnelCard.vue` | 新增按钮 |
| `src/stores/config.ts` | 新增 API 调用 |
| `src/api/tauri.ts` | 新增命令封装 |