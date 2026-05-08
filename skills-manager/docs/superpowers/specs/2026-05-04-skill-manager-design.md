---
title: Skill Manager 设计文档
date: 2026-05-04
status: reviewed
---

# Skill Manager - AI 技能管理工具

## 项目概述

Skill Manager 是一个桌面应用程序，用于管理 AI 技能（Skills）。支持从 GitHub/GitLab/私有 GitLab 同步技能仓库，并通过软连接将技能部署到多个 AI Agent 工具的技能目录。

### 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | 最新版 | 桌面应用框架、Rust 后端 |
| Vue 3 | 最新版 | 前端框架 |
| TDesign | 最新版 | UI 组件库 |
| Pinia | 最新版 | 状态管理 |
| pnpm | 最新版 | 包管理器 |

### 目标 Agent 目录

| Agent | 路径 | 说明 |
|-------|------|------|
| Claude Code | `~/.claude/skills/` | 独立目录 |
| Codex CLI | `~/.agents/skills/` | 与 Central 共用 |
| Gemini CLI | `~/.gemini/skills/` | 独立目录 |
| Trae | `~/.trae/skills/` | 独立目录 |
| OpenCode | `~/.opencode/skills/` | 独立目录 |
| Trae CN | `~/.trae-cn/skills/` | 独立目录 |
| Central | `~/.agents/skills/` | 与 Codex CLI 共用，默认启用 |

---

## 核心功能

### 1. 仓库管理

- 支持添加多个 Git 仓库（GitHub、GitLab、私有 GitLab）
- 每个仓库可配置：名称、URL、认证方式、同步间隔
- 认证方式：无需认证、Token、用户名密码（均适用于 GitHub/GitLab）
- 添加仓库后可预览技能列表，选择需要的技能保存

### 2. 技能同步

- **定时同步**：每个仓库独立配置同步间隔（5分钟 ~ 每天）
- **手动同步**：一键立即同步全部或单个仓库
- **后台检查**：每 5 分钟检查是否需要同步

### 3. 软连接管理

- 技能以软连接形式部署到目标 Agent 目录
- 源目录：`~/.skill-manager/repos/{repo-id}/{skill-name}/`（使用仓库ID作为目录名，避免名称冲突）
- Windows 优先使用 Junction（不需要管理员权限），失败则提示以管理员权限运行
- macOS/Linux 使用标准 symlink

### 4. 冲突处理

- 同名技能发现时弹窗提示用户选择
- 选项：选择某个仓库版本、保留现有版本、跳过此技能

### 5. 技能浏览

- 主界面：左侧仓库列表 + 右侧技能列表 + 技能详情
- 支持按技能名称、描述搜索过滤
- 点击技能查看详情（SKILL.md 内容预览）

---

## 架构设计

### 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    前端层 (Vue3 + TDesign)                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │   主界面    │ │ 添加对话框  │ │  设置页面   │            │
│  │ 仓库+技能   │ │ 预览+选择   │ │ Agent配置  │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
│                      Pinia 状态管理                          │
└─────────────────────────────────────────────────────────────┘
                          │ Tauri invoke
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   Tauri Rust 后端                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │  Git 操作   │ │ 软连接管理  │ │  定时任务   │            │
│  │ clone/pull │ │ create/link │ │  scheduler │            │
│  └─────────────┘ └─────────────┘ └─────────────┘            │
│                      配置文件读写                            │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                      存储层                                  │
│  ~/.skill-manager/config.json    ← 配置文件                 │
│  ~/.skill-manager/repos/{repo-id}/  ← 仓库缓存（使用仓库ID） │
│  ~/.claude/skills/               ← 软连接目标               │
│  ~/.gemini/skills/               ← 软连接目标               │
│  ...                                                         │
└─────────────────────────────────────────────────────────────┘
```

### 技术决策

1. **Git 操作**：调用系统 git CLI，启动时检测 git 是否安装，未安装则引导用户安装
2. **前后端通信**：Tauri invoke，非 HTTP
3. **配置存储**：JSON 文件，不使用数据库
4. **技能元数据**：从 SKILL.md 实时解析，不持久化
5. **认证存储**：Token/密码明文存储在 config.json（适用于个人开发环境）

---

## 数据模型

### config.json 结构

```json
{
  "repositories": [
    {
      "id": "uuid",
      "name": "官方技能库",
      "url": "https://github.com/xxx/skills",
      "auth": {
        "type": "token",  // 可选值: "none" | "token" | "username-password"
        "token": "ghp_xxx",  // type=token 时使用
        "username": "",     // type=username-password 时使用
        "password": ""      // type=username-password 时使用
      },
      "syncInterval": 3600,
      "selectedSkills": ["brainstorming", "tdd"],  // 技能目录名（skill-name），对应仓库中的技能文件夹名称
      "lastSync": "2026-05-04T10:00:00Z",
      "enabled": true
    }
  ],
  "agents": [
    { "id": "claude-code", "name": "Claude Code", "path": "~/.claude/skills/", "enabled": true },
    { "id": "codex-cli", "name": "Codex CLI", "path": "~/.agents/skills/", "enabled": false },
    { "id": "gemini-cli", "name": "Gemini CLI", "path": "~/.gemini/skills/", "enabled": true },
    { "id": "trae", "name": "Trae", "path": "~/.trae/skills/", "enabled": true },
    { "id": "opencode", "name": "OpenCode", "path": "~/.opencode/skills/", "enabled": false },
    { "id": "trae-cn", "name": "Trae CN", "path": "~/.trae-cn/skills/", "enabled": true },
    { "id": "central", "name": "Central", "path": "~/.agents/skills/", "enabled": true }
  ],
  "settings": {
    "defaultSyncInterval": 3600,
    "autoSync": true,
    "checkInterval": 300
  }
}
```

### 技能目录结构

```
skill-name/
├── SKILL.md      # 必需：元数据 + 指令内容
├── scripts/      # 可选：可执行代码
├── references/   # 可选：文档资料
└── assets/       # 可选：模板、资源
```

SKILL.md 元数据：
- `name`：技能名称（必需，最长64字符）
- `description`：技能描述（必需，最长1024字符）

### Git 引导对话框

- 启动时检测 Git 是否安装
- 未安装时弹出对话框，显示：
  - Git 未安装提示信息
  - 各平台安装命令（Windows: winget install Git.Git / Mac: brew install git / Linux: apt install git）
  - 下载链接（https://git-scm.com/downloads）
  - "稍后安装"按钮（可继续使用但禁用仓库同步功能）
  - "已安装，重新检测"按钮

---

## UI 设计

### 主界面布局

- **左侧（200px）**：仓库列表导航 + 同步状态 + 立即同步按钮
- **右侧上方**：搜索栏 + 当前仓库信息
- **右侧左栏（300px）**：技能列表卡片（可点击选中）
- **右侧右栏**：技能详情（名称、描述、来源、SKILL.md 预览）

### 添加仓库对话框

**流程**：添加 → 预览 → 选择 → 保存

**输入字段**：
1. 仓库名称（自定义）
2. 仓库 URL
3. 认证方式（无需认证 / Token / 用户名密码）
4. 同步间隔

**预览区**：
- 左侧：技能列表（可勾选选择）
- 右侧：选中技能的详情预览

### 设置页面

- Agent 目标配置：勾选启用的 Agent（全局配置）
- 自动同步开关
- 默认同步间隔

### 冲突处理对话框

- 同名技能发现时弹出
- 显示各仓库版本信息
- 选项：选择仓库版本 / 保留现有版本 / 跳过

---

## 同步流程

### 流程步骤

```
定时检查 → 判断同步 → Git Pull → 解析技能 → 检查冲突 → 创建软连接 → 更新状态
```

1. **定时检查**：后台定时器每 5 分钟检查
2. **判断同步**：比较 `lastSync + syncInterval` 与当前时间
3. **Git Pull**：对需要同步的仓库执行 git pull（首次则 git clone）
4. **解析技能**：扫描仓库目录，解析每个 SKILL.md 元数据
5. **检查冲突**：同名技能检查，如有冲突则弹窗提示
6. **创建软连接**：为选中技能创建软连接到启用的 Agent 目录
7. **更新状态**：更新 lastSync 时间，刷新 UI

### 软连接策略

| 平台 | 方式 |
|------|------|
| Windows | 优先使用 Junction（不需要管理员权限），若失败则提示以管理员权限运行 |
| macOS/Linux | 标准 symlink |

已存在的软连接：先删除再创建

### 错误处理

| 错误类型 | 处理方式 |
|----------|----------|
| Git 失败 | 显示错误信息，允许重试 |
| 网络问题 | 自动重试 3 次，间隔递增 |
| 权限问题 | 提示以管理员权限运行 |
| 认证失败 | 提示检查 Token/密码配置 |

---

## Tauri 数据结构

```rust
// Git 检测状态
struct GitStatus {
    installed: bool,
    version: Option<String>,
    path: Option<String>,
}

// 系统信息
struct SystemInfo {
    os: String,           // "windows" | "macos" | "linux"
    home_dir: String,
    skill_manager_dir: String,
}
```

---

## Tauri Command 接口

### 系统检测

```rust
// 检测 Git 是否安装
#[tauri::command]
fn check_git_installed() -> Result<GitStatus, String>

// 获取系统信息
#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String>
```

### 仓库管理

```rust
// 添加仓库并获取技能列表
#[tauri::command]
fn fetch_repo_skills(url: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String>

// 同步仓库
#[tauri::command]
fn sync_repository(repo_id: String) -> Result<SyncResult, String>

// 同步所有仓库
#[tauri::command]
fn sync_all_repositories() -> Result<Vec<SyncResult>, String>
```

### 配置管理

```rust
// 读取配置
#[tauri::command]
fn read_config() -> Result<Config, String>

// 保存配置
#[tauri::command]
fn save_config(config: Config) -> Result<(), String>
```

### 软连接管理

```rust
// 创建软连接
#[tauri::command]
fn create_symlink(skill_path: String, target_agents: Vec<String>) -> Result<(), String>

// 删除软连接
#[tauri::command]
fn remove_symlink(skill_name: String, agent_id: String) -> Result<(), String>

// 检查软连接状态
#[tauri::command]
fn check_symlinks() -> Result<Vec<SymlinkStatus>, String>
```

---

## 文件结构

```
skill-manager/
├── src/                    # Vue 前端源码
│   ├── views/
│   │   ├── Home.vue        # 主界面
│   │   ├── Settings.vue    # 设置页面
│   │   └── AddRepoDialog.vue
│   ├── components/
│   │   ├── RepoList.vue
│   │   ├── SkillList.vue
│   │   ├── SkillDetail.vue
│   │   └── ConflictDialog.vue
│   ├── stores/
│   │   ├── config.ts       # Pinia 配置状态
│   │   ├── sync.ts         # 同步状态
│   │   └── skills.ts       # 技能数据
│   ├── api/
│   │   └── tauri.ts        # Tauri invoke 封装
│   └── App.vue
│   └── main.ts
├── src-tauri/              # Rust 后端源码
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── repo.rs     # 仓库管理命令
│   │   │   ├── sync.rs     # 同步命令
│   │   │   ├── config.rs   # 配置命令
│   │   │   └── symlink.rs  # 软连接命令
│   │   ├── git.rs          # Git 操作封装
│   │   ├── scheduler.rs    # 定时任务
│   │   └── skill_parser.rs # SKILL.md 解析
│   └── tauri.conf.json
├── package.json
└── pnpm-lock.yaml
```

---

## 开发计划

### 阶段 1：基础框架

- 初始化 Tauri + Vue3 + TDesign 项目
- Git 检测功能（检测 git 是否安装，未安装则显示引导对话框）
- 实现配置读写
- 实现主界面布局

### 阶段 2：仓库管理

- 添加仓库对话框
- Git clone/pull 实现
- SKILL.md 解析

### 阶段 3：软连接管理

- 软连接创建/删除
- Agent 目录检测
- 冲突处理

### 阶段 4：定时同步

- 后台定时任务
- 同步进度显示
- 错误处理

### 阶段 5：完善优化

- 设置页面
- 搜索过滤
- 测试和修复

---

## 安全考虑

1. **认证信息存储**：Token/密码明文存储在本地 config.json（适用于个人开发环境，用户应自行保管配置文件）
2. **软连接权限**：Windows 使用 Junction 优先，失败时提示用户以管理员身份运行
3. **Git URL 验证**：验证 URL 格式，防止命令注入