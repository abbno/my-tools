# Tool Update Server 设计文档

## 1. 项目概述

**目标：** 开发一个简单的版本管理 Web 服务，支持多个软件的版本管理和安装包分发。

**技术栈：**
- 后端：Rust + Actix-web
- 模板：Askama（编译时 HTML 模板）
- 存储：本地文件目录 + JSON 元数据

**核心功能：**
- 软件版本信息查询 API（供客户端调用）
- 安装包文件下载
- Web 管理页面（软件列表、版本管理、文件上传）
- 无认证机制（信任环境）

---

## 2. 数据模型

### 软件信息

```rust
pub struct App {
    pub id: String,        // 软件标识，如 "ssh-tunnel-manager"
    pub name: String,      // 软件名称，如 "SSH Tunnel Manager"
    pub created_at: String, // 创建时间
}
```

### 版本信息

```rust
pub struct Version {
    pub app_id: String,       // 软件标识
    pub version: String,      // 版本号，如 "0.2.0"
    pub release_date: String, // 发布日期
    pub changelog: Vec<ChangelogItem>, // 更新日志
    pub download_url: String, // 下载链接（相对路径）
    pub file_name: String,    // 文件名
    pub file_size: u64,       // 文件大小（字节）
    pub created_at: String,   // 创建时间
}
```

### 更新日志项

```rust
pub struct ChangelogItem {
    pub item_type: String,    // 类型：feature/fix/improve
    pub description: String,  // 描述
}
```

### 数据存储结构

```
data/
├── apps.json                 # 软件列表
├── {app_id}/
│   ├── versions.json         # 该软件的版本列表
│   └── files/
│       └── {app_id}_{version}.exe  # 安装包文件
```

---

## 3. API 接口设计

### 查询接口（供客户端调用）

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/version/{app_id}` | GET | 获取软件最新版本信息 |
| `/api/version/{app_id}/{version}` | GET | 获取指定版本信息 |
| `/download/{app_id}/{filename}` | GET | 下载安装包文件 |

**版本信息返回格式：**

```json
{
  "version": "0.2.0",
  "release_date": "2026-05-10",
  "download_url": "/download/ssh-tunnel-manager/ssh-tunnel-manager_0.2.0.exe",
  "changelog": [
    { "type": "feature", "description": "新增在线升级功能" },
    { "type": "fix", "description": "修复连接状态显示异常" }
  ]
}
```

### 管理接口（Web 页面使用）

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/apps` | GET | 获取所有软件列表 |
| `/api/apps` | POST | 创建新软件 |
| `/api/apps/{app_id}` | GET | 获取单个软件信息 |
| `/api/apps/{app_id}/versions` | GET | 获取软件所有版本 |
| `/api/apps/{app_id}/versions` | POST | 上传新版本（含文件） |
| `/api/apps/{app_id}/versions/{version}` | DELETE | 删除指定版本 |

---

## 4. Web 管理页面

### 页面路由

| 页面 | 路径 | 功能 |
|------|------|------|
| 首页 | `/` | 软件列表，显示所有软件及其最新版本 |
| 软件详情 | `/app/{app_id}` | 版本列表，上传新版本，删除版本 |
| 新建软件 | `/app/new` | 创建新软件条目 |

### 页面设计风格

**工业实用风格（Industrial/Utilitarian）**

- 深色主题，开发者友好
- 强功能性，数据清晰
- 高对比度，精致细节
- 微妙玻璃效果和渐变

**配色方案：**

| 用途 | 颜色 |
|------|------|
| 背景 | #0a0a0f（深紫黑） |
| 卡片 | #15151f（玻璃效果） |
| 强调 | #00ff88（亮绿） |
| 次要 | #3d3d5c（灰紫） |
| 边框 | #2a2a3a（微光边框） |
| 文字 | #ffffff（白色） |
| 文字次要 | #8888aa（灰白） |

### 首页布局

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ▓▓ VERSION SERVER                              [+ NEW APP]    │
│  ═══════════════════════════════════════════════════════════    │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                                                         │   │
│  │  ████████ SSH TUNNEL MANAGER                           │   │
│  │  ─────────────────────────────────────────────────────  │   │
│  │  LATEST    v0.2.0          2026-05-10                  │   │
│  │  STATUS    ● ACTIVE                                   │   │
│  │                                          [ DETAILS → ] │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ... 其他软件卡片 ...                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 软件详情页布局

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ← BACK TO HOME                                                 │
│                                                                 │
│  ████████ SSH TUNNEL MANAGER                                   │
│  ═══════════════════════════════════════════════════════════    │
│                                                                 │
│  [⬆ UPLOAD NEW VERSION]                                        │
│                                                                 │
│  VERSION HISTORY                                                │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  v0.2.0                          2026-05-10             │   │
│  │  ─────────────────────────────────────────────────────  │   │
│  │  ● feature  新增在线升级功能                           │   │
│  │  ● fix      修复连接状态显示异常                       │   │
│  │                                                         │   │
│  │  FILE: ssh-tunnel-manager_0.2.0.exe (15.2 MB)          │   │
│  │                                          [↓] [✕ DELETE]│   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ... 其他版本卡片 ...                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. 技术实现

### 项目结构

```
tool-update-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口，启动服务
│   ├── models.rs            # 数据结构定义
│   ├── storage.rs           # 文件存储操作
│   ├── handlers.rs          # HTTP 路由处理
│   └── templates.rs         # Askama 模板定义
├── templates/
│   ├── index.html           # 首页模板
│   ├── app_detail.html      # 软件详情页模板
│   └── new_app.html         # 新建软件页模板
├── static/
│   └── style.css            # 样式文件
└── data/                    # 数据存储目录（运行时创建）
    ├── apps.json
    └── {app_id}/
        ├── versions.json
        └── files/
```

### Cargo.toml 依赖

```toml
[package]
name = "tool-update-server"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-files = "0.6"
actix-multipart = "0.6"
askama = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"

[profile.release]
opt-level = "s"
strip = true
lto = true
```

### 关键模块职责

| 模块 | 职责 |
|------|------|
| main.rs | 服务启动、路由配置 |
| models.rs | 数据结构定义、序列化 |
| storage.rs | 文件读写、JSON 元数据管理 |
| handlers.rs | HTTP 请求处理、业务逻辑 |
| templates.rs | Askama 模板结构体 |

---

## 6. 服务配置

### 默认配置

- 监听端口：8080
- 数据目录：`./data`（相对可执行文件）
- 静态文件：`./static`
- 模板文件：`./templates`

### 启动命令

```bash
# 开发模式
cargo run

# 生产模式（构建后）
cargo build --release
./tool-update-server
```

### 环境变量（可选）

| 变量 | 说明 | 默认值 |
|------|------|------|
| SERVER_PORT | 监听端口 | 8080 |
| DATA_DIR | 数据目录 | ./data |

---

## 7. 错误处理

| 场景 | 处理方式 |
|------|----------|
| 软件不存在 | 返回 404，显示友好错误页面 |
| 版本不存在 | 返回 404 |
| 文件上传失败 | 返回 500，提示重试 |
| 文件过大 | 限制最大 500MB，返回 400 |
| 数据损坏 | 尝试恢复，无法恢复则提示管理员 |

---

## 8. 客户端对接

### Tauri 配置调整

在 `tauri.conf.json` 中配置 updater endpoint：

```json
{
  "plugins": {
    "updater": {
      "endpoints": ["http://your-server:8080/api/version/{app_id}"],
      "pubkey": ""
    }
  }
}
```

### 版本信息兼容

服务返回的 JSON 格式与 Tauri Updater 要求兼容：

```json
{
  "version": "0.2.0",
  "release_date": "2026-05-10",
  "download_url": "/download/ssh-tunnel-manager/ssh-tunnel-manager_0.2.0.exe",
  "changelog": [...]
}
```

---

## 9. 后续扩展（可选）

以下功能暂不实现，保留扩展空间：

- 用户认证（API Key 或密码）
- 云存储支持（OSS、S3）
- 下载统计
- 灰度发布
- 签名验证