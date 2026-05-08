# 配置更新检测地址前缀设计文档

## 1. 功能概述

**目标：** 支持用户配置更新服务器地址前缀，实现灵活的在线更新检测。

**核心需求：**
- 默认不配置，不检测更新
- 用户可配置更新服务器地址前缀
- 配置保存到数据库
- 保存后自动触发更新检测
- 支持清除配置，恢复默认不检测状态

---

## 2. 数据模型设计

### 新增 app_settings 表（键值对结构）

```sql
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

**初始设置项：**

| key | value | 说明 |
|------|------|------|
| update_server_url | 用户配置的值 | 更新服务器地址前缀，如 `https://myserver.com` |

---

## 3. 后端设计

### 3.1 新增 Tauri 命令

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `get_app_setting` | key: String | Option<String> | 获取单个设置项 |
| `save_app_setting` | key: String, value: String | - | 保存设置项 |
| `delete_app_setting` | key: String | - | 删除设置项 |

### 3.2 文件结构变更

新增文件：
- `src-tauri/src/models/app_setting.rs` - 应用设置数据模型
- `src-tauri/src/commands/app_setting.rs` - 设置相关命令处理

修改文件：
- `src-tauri/src/db/sqlite.rs` - 新增 app_settings 表初始化和 CRUD 操作
- `src-tauri/src/updater/mod.rs` - 动态读取配置拼接 URL
- `src-tauri/src/commands/mod.rs` - 注册新命令
- `src-tauri/src/lib.rs` - 注册新命令

### 3.3 更新检测 URL 格式

```
{update_server_url}/api/version/{app_name}
```

**示例：**
```
https://myserver.com/api/version/ssh-tunnel-manager
```

**参数说明：**
- `update_server_url`：用户配置的服务器地址前缀
- `app_name`：固定的简短应用名称 `ssh-tunnel-manager`

### 3.4 check_update 命令逻辑修改

```
调用 check_update
    │
    ▼
从数据库读取 update_server_url
    │
    ├── 未配置 → 返回错误："请先配置更新服务器地址"
    │
    ▼ 已配置
拼接完整 URL：{update_server_url}/api/version/ssh-tunnel-manager
    │
    ▼
使用该 URL 检查更新
    │
    ├── 有更新 → 返回 UpdateInfo
    └── 无更新 → 返回 None
```

---

## 4. 前端设计

### 4.1 Settings.vue 页面布局

```
┌─────────────────────────────────────────────┐
│ 设置                           [保存]       │
├─────────────────────────────────────────────┤
│ 版本信息                                    │
│ 当前版本：0.1.0                             │
│ 上次检查：2026-05-08 10:30                 │
│ [检查更新]                                  │
├─────────────────────────────────────────────┤
│ 更新服务器配置                              │
│ 服务器地址：                                │
│ [https://myserver.com                ]      │
│ [清除]                                      │
│ 提示：配置后将自动检查更新                  │
├─────────────────────────────────────────────┤
│ 其他设置                                    │
│ 更多设置功能正在开发中...                   │
├─────────────────────────────────────────────┤
│ [返回主页]                                  │
└─────────────────────────────────────────────┘
```

### 4.2 交互逻辑

**页面加载：**
- 从数据库读取 `update_server_url`，填充输入框

**保存按钮：**
- 保存所有修改的配置项到数据库
- 如果 `update_server_url` 有变更且非空，自动触发更新检测
- 如果输入框为空，删除 `update_server_url` 配置项

**清除按钮：**
- 清空输入框内容（保存时将删除配置项）

**检查更新按钮：**
- 从数据库读取 `update_server_url`
- 未配置时提示"请先配置更新服务器地址"
- 已配置时检查更新

### 4.3 文件结构变更

新增/修改文件：
- `src/api/tauri.ts` - 新增 `getAppSetting`、`saveAppSetting`、`deleteAppSetting` API
- `src/stores/update.ts` - 新增配置相关状态和方法
- `src/views/Settings.vue` - 新增更新服务器配置区域和保存逻辑

---

## 5. 核心流程设计

### 5.1 保存配置后流程

```
用户点击"保存"
    │
    ▼
保存 update_server_url 到数据库
    │
    ▼
检查 update_server_url 是否有效（非空）
    │
    ├── 无效 → 删除配置，不触发检测
    │
    ▼ 有效
拼接完整 URL：{update_server_url}/api/version/ssh-tunnel-manager
    │
    ▼
调用 Tauri Updater 检查更新
    │
    ├── 有更新 → 显示更新弹窗
    └── 无更新 → 提示"当前已是最新版本"
```

### 5.2 检查更新按钮流程

```
用户点击"检查更新"
    │
    ▼
从数据库读取 update_server_url
    │
    ├── 未配置 → 提示"请先配置更新服务器地址"
    │
    ▼ 已配置
拼接完整 URL 并检查更新
    │
    ├── 有更新 → 显示更新弹窗
    └── 无更新 → 提示"当前已是最新版本"
```

---

## 6. 技术要点

### 6.1 Tauri Updater 动态 Endpoint

Tauri Updater 默认从 `tauri.conf.json` 的 `plugins.updater.endpoints` 读取静态配置。需要改为运行时动态传入 URL。

**实现方式：**
- 使用 `UpdaterExt::builder()` 构建 Updater
- 调用 `.endpoints()` 方法传入动态 URL
- 不再依赖 `tauri.conf.json` 的静态配置

### 6.2 配置持久化

配置存储在 SQLite 数据库的 `app_settings` 表中，与其他配置数据一起管理，便于备份和迁移。

---

## 7. 后续扩展

`app_settings` 表采用键值对结构，后续可轻松扩展其他配置项，如：
- `auto_check_update`：是否自动检查更新
- `check_interval`：检查间隔时间
- 其他应用级别设置