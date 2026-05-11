# 添加仓库流程重构设计规范

## 目标

修改添加仓库流程，支持选择分支，自动生成仓库名称，本地按分支独立存储。

## 背景

当前添加仓库流程不支持分支选择，仓库名称需手动输入。用户希望：
- 输入 URL 后获取远程分支列表
- 选择要使用的分支
- 仓库名称自动生成（格式：`owner/repo(branch)`）
- 同一仓库的不同分支作为独立仓库管理

## 架构

采用最小改动方案：
- 数据模型：Repository 新增 `branch` 字段
- 后端：新增 `fetch_branches` 命令，修改现有命令支持分支参数
- 前端：AddRepoDialog.vue 改为 4 步流程
- Git 操作：新增分支获取函数，clone 时指定分支

## 技术栈

- Vue 3 Composition API + TDesign Vue-next
- Tauri + Rust
- Git 命令行工具

---

## 第一部分：数据模型变更

### Repository 模型修改

**文件：** `src-tauri/src/models/config.rs`

```rust
pub struct Repository {
    pub id: String,
    pub name: String,        // 自动生成，格式：owner/repo(branch)
    pub url: String,
    pub branch: String,      // 新增：分支名称
    pub auth: AuthConfig,
    pub sync_interval: u64,
    pub selected_skills: Vec<String>,
    pub last_sync: Option<DateTime<Utc>>,
    pub enabled: bool,
}
```

### 名称生成规则

从 URL 解析 owner/repo 部分，组合分支名：

| URL | 分支 | 生成的名称 |
|-----|------|-----------|
| `https://github.com/MiniMax-AI/skills` | `main` | `MiniMax-AI/skills(main)` |
| `https://gitlab.com/company/project` | `dev` | `company/project(dev)` |
| `https://gitee.com/user/repo` | `feature-x` | `user/repo(feature-x)` |

解析逻辑：
1. 去掉协议前缀（`https://`、`http://`、`git@`）
2. 去掉域名部分（`github.com`、`gitlab.com` 等）
3. 去掉 `.git` 后缀（如有）
4. 得到 `owner/repo` 格式
5. 组合分支名：`owner/repo(branch)`

### 本地存储路径

- 路径：`~/.skill-manager/repos/{repo_id}/`
- repo_id 使用 UUID，不依赖名称
- 同一仓库的不同分支有独立目录和独立 repo_id

---

## 第二部分：后端 API 变更

### 新增命令：fetch_branches

**文件：** `src-tauri/src/commands/repo.rs`

```rust
#[tauri::command]
pub fn fetch_branches(url: String, auth: AuthConfig) -> Result<Vec<String>, String>
```

功能：
- 使用 `git ls-remote --heads <url>` 获取远程分支列表
- 解析输出，提取分支名（`refs/heads/main` → `main`）
- 处理认证（通过环境变量）
- 返回分支名数组

### 修改命令：fetch_repo_skills

**文件：** `src-tauri/src/commands/repo.rs`

```rust
#[tauri::command]
pub fn fetch_repo_skills(url: String, branch: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String>
```

变更：
- 新增 `branch` 参数
- clone 时使用 `git clone --branch <branch> --depth 1 <url> <path>`
- 浅克隆加速预览，扫描后清理临时目录

### 修改命令：sync_repository

**文件：** `src-tauri/src/commands/repo.rs`

```rust
#[tauri::command]
pub fn sync_repository(repo_id: String, url: String, branch: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String>
```

变更：
- 新增 `branch` 参数
- clone 时指定分支
- pull 前确保 checkout 到正确分支

---

## 第三部分：前端流程变更

### AddRepoDialog.vue 4 步流程

**文件：** `src/views/AddRepoDialog.vue`

**Step 1: 输入 URL + 认证**

| 组件 | 说明 |
|------|------|
| 仓库 URL 输入框 | 必填，placeholder: `https://github.com/user/repo` |
| 认证方式选择 | 无需认证 / 令牌 / 用户名和密码 |
| 令牌输入框 | 认证方式为"令牌"时显示 |
| 用户名/密码输入框 | 认证方式为"用户名和密码"时显示 |
| "获取分支"按钮 | 点击后调用 `fetch_branches` |

交互：
- 点击"获取分支"后显示加载状态
- 成功后自动进入 Step 2
- 失败显示错误提示，可重试

**Step 2: 选择分支**

| 组件 | 说明 |
|------|------|
| 分支下拉框 | 展示获取到的分支列表 |
| 默认值 | 第一个分支（通常是 main 或 master） |

交互：
- 选择分支后点击"继续"进入 Step 3

**Step 3: 选择技能**

| 组件 | 说明 |
|------|------|
| 加载状态 | 调用 `fetch_repo_skills(url, branch, auth)` |
| 技能列表 | 多选框，显示技能名和描述 |
| 全选/取消全选 | 快捷操作 |
| 默认状态 | 全选 |

交互：
- 点击"继续"进入 Step 4
- 至少选择一个技能才能继续

**Step 4: 确认**

| 信息 | 来源 |
|------|------|
| 名称 | 自动生成：`owner/repo(branch)` |
| URL | Step 1 输入 |
| 分支 | Step 2 选择 |
| 认证 | Step 1 选择 |
| 同步间隔 | 默认 1 小时（可在设置中修改） |
| 已选技能 | Step 3 选择 |

交互：
- 点击"保存仓库"完成添加
- 自动跳转到新添加的仓库

### UI 变化

1. **去掉"仓库名称"输入框** - 改为自动生成
2. **去掉"同步间隔"选择** - 使用默认值，后续在设置中修改
3. **Steps 组件** - 从 3 步改为 4 步
4. **Step 1 按钮** - "继续"改为"获取分支"
5. **Step 2 新增** - 分支选择下拉框

---

## 第四部分：Git 操作变更

### 新增函数：fetch_remote_branches

**文件：** `src-tauri/src/git.rs`

```rust
pub fn fetch_remote_branches(url: &str, auth: &AuthConfig) -> Result<Vec<String>, String>
```

实现：
```rust
let mut cmd = Command::new("git");
cmd.arg("ls-remote");
cmd.arg("--heads");
cmd.arg(url);

// 认证处理
if auth.auth_type == "token" {
    cmd.env("GIT_ASKPASS", "echo");
    cmd.env("GIT_PASSWORD", auth.token.as_ref().unwrap());
} else if auth.auth_type == "username-password" {
    cmd.env("GIT_ASKPASS", "echo");
    cmd.env("GIT_USERNAME", auth.username.as_ref().unwrap());
    cmd.env("GIT_PASSWORD", auth.password.as_ref().unwrap());
}

let output = cmd.output()?;
// 解析 refs/heads/xxx 格式
```

### 修改函数：clone_repo

**文件：** `src-tauri/src/git.rs`

```rust
pub fn clone_repo(url: &str, branch: &str, path: &PathBuf, auth: &AuthConfig) -> GitResult
```

变更：
- 新增 `branch` 参数
- 命令：`git clone --branch <branch> <url> <path>`
- 可选 `--depth 1` 用于浅克隆预览

### 新增函数：checkout_branch

**文件：** `src-tauri/src/git.rs`

```rust
pub fn checkout_branch(path: &PathBuf, branch: &str) -> GitResult
```

实现：
```rust
let mut cmd = Command::new("git");
cmd.arg("checkout");
cmd.arg(branch);
cmd.current_dir(path);
```

---

## 第五部分：前端 API 调用变更

### tauri.ts 新增

**文件：** `src/api/tauri.ts`

```typescript
export async function fetchBranches(url: string, auth: AuthConfig): Promise<string[]> {
  return invoke<string[]>('fetch_branches', { url, auth })
}
```

### tauri.ts 修改

```typescript
export async function fetchRepoSkills(url: string, branch: string, auth: AuthConfig): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('fetch_repo_skills', { url, branch, auth })
}

export async function syncRepository(repoId: string, url: string, branch: string, auth: AuthConfig): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('sync_repository', { repoId, url, branch, auth })
}
```

### Repository 类型修改

```typescript
export interface Repository {
  id: string
  name: string
  url: string
  branch: string       // 新增
  auth: AuthConfig
  sync_interval: number
  selected_skills: string[]
  last_sync: string | null
  enabled: boolean
}
```

---

## 第六部分：日志系统

### 依赖

**Cargo.toml 添加：**

```toml
[dependencies]
log4rs = "1.3"
log = "0.4"
```

### 日志配置

**文件：** `src-tauri/src/logger.rs`

```rust
use log4rs::config::{Appender, Config, Root};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::trigger::time::TimeTrigger;
use log4rs::append::rolling_file::policy::compound::roller::FixedWindowRoller;
use log::LevelFilter;
use std::path::PathBuf;

pub fn init_logger(app_handle: &tauri::AppHandle) -> Result<(), String> {
    // 获取应用资源目录下的 logs
    let logs_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("logs");
    
    // 确保 logs 目录存在
    std::fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    
    let log_file = logs_dir.join("app.log");
    
    // 配置滚动策略
    // 大小触发：100MB
    let size_trigger = SizeTrigger::new(100 * 1024 * 1024);
    
    // 时间触发：每天
    let time_trigger = TimeTrigger::new(
        log4rs::append::rolling_file::policy::compound::trigger::time::Frequency::Daily
    );
    
    // 滚动器：最多保留 5 个文件
    let roller = FixedWindowRoller::builder()
        .build(logs_dir.join("app.log.%d").to_string_lossy(), 5)
        .map_err(|e| e.to_string())?;
    
    // 复合策略：大小或时间任一满足就轮转
    let policy = CompoundPolicy::new(
        Box::new(size_trigger),
        Box::new(time_trigger),
        Box::new(roller),
    );
    
    // 创建滚动文件 appender
    let appender = RollingFileAppender::builder()
        .append(true)
        .build(log_file, Box::new(policy))
        .map_err(|e| e.to_string())?;
    
    // 配置
    let config = Config::builder()
        .appender(Appender::builder().build("main", Box::new(appender)))
        .build(Root::builder().appender("main").build(LevelFilter::Info))
        .map_err(|e| e.to_string())?;
    
    // 初始化
    log4rs::init_config(config).map_err(|e| e.to_string())?;
    
    Ok(())
}
```

### 日志文件命名

轮转后的文件命名规则：
- 当前日志：`app.log`
- 第 1 个备份：`app.log.1`
- 第 2 个备份：`app.log.2`
- ...
- 第 5 个备份：`app.log.5`

超过 5 个时，最旧的（`app.log.5`）会被删除。

### 日志格式

```
[2026-05-11 10:30:45 INFO] Fetching branches for repository: https://github.com/user/repo
[2026-05-11 10:30:46 INFO] Found 3 branches: main, dev, feature-x
[2026-05-11 10:30:47 ERROR] Failed to clone repository: authentication failed
```

格式：`[时间 级别] 消息`

### 日志使用示例

```rust
use log::{info, error, warn, debug};

// 分支获取
info!("Fetching branches for repository: {}", url);
info!("Found {} branches: {:?}", branches.len(), branches);
error!("Failed to fetch branches: {}", err);

// 仓库克隆
info!("Cloning repository {} branch {} to {}", url, branch, path);
error!("Clone failed: {}", err);

// 技能扫描
info!("Scanning skills in {}", path);
info!("Found {} skills", skills.len());
```

### 日志初始化时机

在 `lib.rs` 的 `setup` 函数中初始化：

```rust
.setup(|app| {
    // 初始化日志系统
    logger::init_logger(app.handle())?;
    
    // ... 其他初始化
    Ok(())
})
```

---

## 第七部分：打包配置变更

### NSIS 安装器配置

**文件：** `src-tauri/tauri.conf.json`

修改 bundle 配置：

```json
{
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "nsis": {
        "installMode": "perMachine",
        "template": "./nsis/installer.nsi"
      }
    }
  }
}
```

### 自定义 NSIS 模板

**文件：** `src-tauri/nsis/installer.nsi`

复制 ssh-tunnel-manager 的 NSIS 模板，关键修改：

```nsis
Function .onInit
  ; ... 其他初始化代码

  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    ; Set default install location - custom directory
    StrCpy $INSTDIR "D:\Programs\${PRODUCTNAME}"

    Call RestorePreviousInstallLocation
  ${EndIf}
FunctionEnd
```

**默认安装目录：** `D:\Programs\Skill Manager`

### 日志目录说明

由于日志使用 `resource_dir()` 获取应用安装目录，日志将存储在：
- 开发模式：项目目录下的 `logs/`
- 生产模式：`D:\Programs\Skill Manager\logs/`（与安装目录一致）

---

## 测试验证

1. **安装测试**
   - 打包后安装，验证默认安装目录为 `D:\Programs\Skill Manager`
   - 日志文件存储在 `D:\Programs\Skill Manager\logs/`

2. **日志轮转测试**
   - 生成大量日志触发 100MB 轮转
   - 验证日志文件数量不超过 5 个

1. **公开仓库测试**
   - 输入 `https://github.com/MiniMax-AI/skills`
   - 获取分支成功
   - 选择分支后获取技能列表

2. **私有仓库测试**
   - 使用令牌认证
   - 获取分支成功
   - clone 成功

3. **多分支测试**
   - 同一 URL 添加 main 和 dev 分支
   - 两个仓库独立存在
   - 各自有独立存储目录

4. **同步测试**
   - 点击同步按钮
   - 拉取正确分支的更新

---

## 不包含的内容

- 仓库组概念（多个分支作为一组管理）
- 分支切换功能（已添加的仓库切换到其他分支）
- GitHub/GitLab API 集成（使用 git 命令行）