# 添加仓库流程重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构添加仓库流程，支持分支选择、自动生成名称、日志系统、设置页面重构。

**Architecture:** 后端新增分支获取和日志系统，修改 Repository 模型；前端改为 4 步流程，新增设置页面路由。

**Tech Stack:** Vue 3 + TDesign, Tauri + Rust, log4rs, NSIS

---

## 文件结构

**新建文件：**
- `src-tauri/src/logger.rs` - 日志系统初始化
- `src-tauri/nsis/installer.nsi` - NSIS 安装器模板
- `src/views/Settings.vue` - 设置页面

**修改文件：**
- `src-tauri/Cargo.toml` - 添加 log4rs 依赖
- `src-tauri/src/lib.rs` - 初始化日志、注册新命令
- `src-tauri/src/models/config.rs` - Repository 添加 branch 字段
- `src-tauri/src/git.rs` - 新增 fetch_remote_branches、checkout_branch，修改 clone_repo
- `src-tauri/src/commands/repo.rs` - 新增 fetch_branches，修改现有命令
- `src-tauri/tauri.conf.json` - NSIS 配置
- `src/api/tauri.ts` - 新增 fetchBranches，修改现有函数，Repository 类型
- `src/views/AddRepoDialog.vue` - 改为 4 步流程
- `src/router.ts` - 新增 /settings 路由
- `src/App.vue` - 设置按钮跳转路由

**删除文件：**
- `src/components/SettingsDialog.vue` - 不再使用弹框

---

### Task 1: 添加日志依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 log4rs 和 log 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 部分添加：

```toml
log4rs = "1.3"
log = "0.4"
```

完整的 dependencies 部分：

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
which = "6"
log4rs = "1.3"
log = "0.4"
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add log4rs and log dependencies"
```

---

### Task 2: 创建日志系统模块

**Files:**
- Create: `src-tauri/src/logger.rs`

- [ ] **Step 1: 创建 logger.rs 文件**

```rust
use log4rs::config::{Appender, Config, Root, Logger};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::trigger::time::TimeTrigger;
use log4rs::append::rolling_file::policy::compound::roller::FixedWindowRoller;
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use tauri::Manager;

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
    
    // 日志格式：[时间 级别] 消息
    let encoder = PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)} {l}] {m}\n");
    
    // 大小触发：100MB
    let size_trigger = SizeTrigger::new(100 * 1024 * 1024);
    
    // 时间触发：每天
    let time_trigger = TimeTrigger::new(
        log4rs::append::rolling_file::policy::compound::trigger::time::Frequency::Daily
    );
    
    // 滚动器：最多保留 5 个文件，命名格式 app.log.1, app.log.2, ...
    let roller = FixedWindowRoller::builder()
        .base(1)
        .count(5)
        .build(logs_dir.join("app.log.{}").to_string_lossy())
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
        .encoder(Box::new(encoder))
        .build(log_file, Box::new(policy))
        .map_err(|e| e.to_string())?;
    
    // 配置
    let config = Config::builder()
        .appender(Appender::builder().build("main", Box::new(appender)))
        .build(Root::builder().appender("main").build(LevelFilter::Info))
        .map_err(|e| e.to_string())?;
    
    // 初始化
    log4rs::init_config(config).map_err(|e| e.to_string())?;
    
    log::info!("Logger initialized, logs directory: {}", logs_dir.to_string_lossy());
    
    Ok(())
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/logger.rs
git commit -m "feat: add logger module with log4rs"
```

---

### Task 3: 在 lib.rs 中初始化日志

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加 logger 模块声明**

在 `src-tauri/src/lib.rs` 开头添加 `logger` 模块：

```rust
pub mod commands;
pub mod models;
pub mod git;
pub mod skill_parser;
pub mod symlink;
pub mod scheduler;
pub mod logger;  // 新增

use tauri::Manager;
use scheduler::start_scheduler;
```

- [ ] **Step 2: 在 setup 中初始化日志**

修改 `setup` 函数：

```rust
.setup(|app| {
    #[cfg(debug_assertions)]
    {
        let window = app.get_webview_window("main").unwrap();
        window.open_devtools();
    }

    // 初始化日志系统
    logger::init_logger(app.handle())?;

    // Start background scheduler
    start_scheduler(app.handle().clone());

    Ok(())
})
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: initialize logger in app setup"
```

---

### Task 4: 修改 Repository 模型添加 branch 字段

**Files:**
- Modify: `src-tauri/src/models/config.rs`

- [ ] **Step 1: 添加 branch 字段**

在 `Repository` 结构体中添加 `branch` 字段：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub url: String,
    pub branch: String,      // 新增：分支名称
    pub auth: AuthConfig,
    pub sync_interval: u64,
    pub selected_skills: Vec<String>,
    pub last_sync: Option<DateTime<Utc>>,
    pub enabled: bool,
}
```

- [ ] **Step 2: 修改 Repository::new 函数**

```rust
impl Repository {
    pub fn new(name: String, url: String, branch: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            url,
            branch,
            auth: AuthConfig::default(),
            sync_interval: 3600,
            selected_skills: Vec::new(),
            last_sync: None,
            enabled: true,
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models/config.rs
git commit -m "feat: add branch field to Repository model"
```

---

### Task 5: 新增 fetch_remote_branches 函数

**Files:**
- Modify: `src-tauri/src/git.rs`

- [ ] **Step 1: 添加 fetch_remote_branches 函数**

在 `src-tauri/src/git.rs` 中添加：

```rust
use log::{info, error};

/// Fetch remote branches using git ls-remote
pub fn fetch_remote_branches(url: &str, auth: &AuthConfig) -> Result<Vec<String>, String> {
    info!("Fetching remote branches for: {}", url);
    
    let mut cmd = Command::new("git");
    cmd.arg("ls-remote");
    cmd.arg("--heads");
    cmd.arg(url);

    // Add authentication
    if auth.auth_type == "token" {
        if let Some(token) = &auth.token {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_PASSWORD", token);
        }
    } else if auth.auth_type == "username-password" {
        if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
        }
    }

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let branches: Vec<String> = stdout
                    .lines()
                    .filter_map(|line| {
                        // Format: <hash>	refs/heads/<branch>
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let ref_path = parts[1];
                            // Extract branch name from refs/heads/xxx
                            if ref_path.starts_with("refs/heads/") {
                                Some(ref_path.strip_prefix("refs/heads/").unwrap().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                
                info!("Found {} branches: {:?}", branches.len(), branches);
                Ok(branches)
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                error!("Failed to fetch branches: {}", stderr);
                Err(stderr.to_string())
            }
        }
        Err(e) => {
            error!("Failed to execute git ls-remote: {}", e);
            Err(format!("Failed to execute git ls-remote: {}", e))
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/git.rs
git commit -m "feat: add fetch_remote_branches function"
```

---

### Task 6: 修改 clone_repo 函数支持分支参数

**Files:**
- Modify: `src-tauri/src/git.rs`

- [ ] **Step 1: 修改 clone_repo 函数签名和实现**

将 `clone_repo` 函数修改为支持分支参数：

```rust
/// Clone a repository to the specified path with specific branch
pub fn clone_repo(url: &str, branch: &str, path: &PathBuf, auth: &AuthConfig) -> GitResult {
    info!("Cloning repository {} branch {} to {}", url, branch, path.to_string_lossy());
    
    let mut cmd = Command::new("git");
    cmd.arg("clone");
    cmd.arg("--branch");
    cmd.arg(branch);
    cmd.arg(url);
    cmd.arg(path);

    // Add authentication via environment variables
    if auth.auth_type == "token" {
        if let Some(token) = &auth.token {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_PASSWORD", token);
        }
    } else if auth.auth_type == "username-password" {
        if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
        }
    }

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                info!("Repository cloned successfully");
                GitResult {
                    success: true,
                    message: "Repository cloned successfully".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                error!("Clone failed: {}", stderr);
                GitResult {
                    success: false,
                    message: stderr.to_string(),
                }
            }
        }
        Err(e) => {
            error!("Failed to execute git clone: {}", e);
            GitResult {
                success: false,
                message: format!("Failed to execute git clone: {}", e),
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/git.rs
git commit -m "feat: modify clone_repo to support branch parameter"
```

---

### Task 7: 新增 checkout_branch 函数

**Files:**
- Modify: `src-tauri/src/git.rs`

- [ ] **Step 1: 添加 checkout_branch 函数**

在 `src-tauri/src/git.rs` 中添加：

```rust
/// Checkout to a specific branch in a repository
pub fn checkout_branch(path: &PathBuf, branch: &str) -> GitResult {
    info!("Checking out branch {} in {}", branch, path.to_string_lossy());
    
    let mut cmd = Command::new("git");
    cmd.arg("checkout");
    cmd.arg(branch);
    cmd.current_dir(path);

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                info!("Checkout successful");
                GitResult {
                    success: true,
                    message: "Checkout successful".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                error!("Checkout failed: {}", stderr);
                GitResult {
                    success: false,
                    message: stderr.to_string(),
                }
            }
        }
        Err(e) => {
            error!("Failed to execute git checkout: {}", e);
            GitResult {
                success: false,
                message: format!("Failed to execute git checkout: {}", e),
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/git.rs
git commit -m "feat: add checkout_branch function"
```

---

### Task 8: 新增 fetch_branches 命令

**Files:**
- Modify: `src-tauri/src/commands/repo.rs`

- [ ] **Step 1: 添加 fetch_branches 命令**

在 `src-tauri/src/commands/repo.rs` 中添加导入和命令：

首先更新导入：

```rust
use crate::git::{clone_repo, pull_repo, get_repo_path, is_git_repo, fetch_remote_branches, checkout_branch};
use crate::skill_parser::scan_skills;
use crate::symlink::{create_symlink, get_skill_source_path, get_skill_target_path, ensure_agent_dir};
use crate::models::{AuthConfig, SkillMeta};
use log::{info, error};
```

添加命令：

```rust
#[tauri::command]
pub fn fetch_branches(url: String, auth: AuthConfig) -> Result<Vec<String>, String> {
    info!("Fetching branches for URL: {}", url);
    fetch_remote_branches(&url, &auth)
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands/repo.rs
git commit -m "feat: add fetch_branches command"
```

---

### Task 9: 修改 fetch_repo_skills 命令支持分支

**Files:**
- Modify: `src-tauri/src/commands/repo.rs`

- [ ] **Step 1: 修改 fetch_repo_skills 函数**

```rust
#[tauri::command]
pub fn fetch_repo_skills(url: String, branch: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String> {
    info!("Fetching repo skills for {} branch {}", url, branch);
    
    // Generate a temporary repo ID for preview
    let temp_repo_id = "preview".to_string();
    let temp_path = get_repo_path(&temp_repo_id)?;

    // Clone to temporary location with specific branch
    let result = clone_repo(&url, &branch, &temp_path, &auth);
    if !result.success {
        error!("Clone failed: {}", result.message);
        return Err(result.message);
    }

    // Scan for skills
    info!("Scanning skills in {}", temp_path.to_string_lossy());
    let skills = scan_skills(&temp_path, &temp_repo_id);

    // Clean up temporary clone
    if temp_path.exists() {
        std::fs::remove_dir_all(&temp_path).ok();
        info!("Cleaned up temporary clone");
    }

    info!("Found {} skills", skills.len());
    Ok(skills)
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands/repo.rs
git commit -m "feat: modify fetch_repo_skills to support branch parameter"
```

---

### Task 10: 修改 sync_repository 命令支持分支

**Files:**
- Modify: `src-tauri/src/commands/repo.rs`

- [ ] **Step 1: 修改 sync_repository 函数**

```rust
#[tauri::command]
pub fn sync_repository(repo_id: String, url: String, branch: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String> {
    info!("Syncing repository {} branch {}", repo_id, branch);
    
    let repo_path = get_repo_path(&repo_id)?;

    if is_git_repo(&repo_path) {
        // Ensure we're on the correct branch
        let checkout_result = checkout_branch(&repo_path, &branch);
        if !checkout_result.success {
            error!("Checkout failed: {}", checkout_result.message);
            return Err(checkout_result.message);
        }
        
        // Pull existing repo
        let result = pull_repo(&repo_path, &auth);
        if !result.success {
            error!("Pull failed: {}", result.message);
            return Err(result.message);
        }
        info!("Pull successful");
    } else {
        // Clone new repo with specific branch
        let result = clone_repo(&url, &branch, &repo_path, &auth);
        if !result.success {
            error!("Clone failed: {}", result.message);
            return Err(result.message);
        }
        info!("Clone successful");
    }

    // Scan for skills
    info!("Scanning skills in {}", repo_path.to_string_lossy());
    let skills = scan_skills(&repo_path, &repo_id);
    info!("Found {} skills", skills.len());

    Ok(skills)
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands/repo.rs
git commit -m "feat: modify sync_repository to support branch parameter"
```

---

### Task 11: 注册新命令到 lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 注册 fetch_branches 命令**

修改 `invoke_handler` 部分：

```rust
.invoke_handler(tauri::generate_handler![
    commands::check_git_installed,
    commands::get_system_info,
    commands::read_config,
    commands::save_config,
    commands::fetch_branches,          // 新增
    commands::fetch_repo_skills,
    commands::sync_repository,
    commands::deploy_skill,
    commands::undeploy_skill,
    symlink::create_skill_symlink,
    symlink::remove_skill_symlink,
    symlink::check_symlinks,
    scheduler::sync_all_repositories,
    scheduler::get_sync_status,
])
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: register fetch_branches command"
```

---

### Task 12: 修改前端 API - Repository 类型

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 修改 Repository 接口**

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

- [ ] **Step 2: Commit**

```bash
git add src/api/tauri.ts
git commit -m "feat: add branch field to Repository interface"
```

---

### Task 13: 修改前端 API - 添加 fetchBranches

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: 添加 fetchBranches 函数**

在 `src/api/tauri.ts` 中添加：

```typescript
export async function fetchBranches(url: string, auth: AuthConfig): Promise<string[]> {
  return invoke<string[]>('fetch_branches', { url, auth })
}
```

- [ ] **Step 2: 修改 fetchRepoSkills 函数**

```typescript
export async function fetchRepoSkills(url: string, branch: string, auth: AuthConfig): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('fetch_repo_skills', { url, branch, auth })
}
```

- [ ] **Step 3: 修改 syncRepository 函数**

```typescript
export async function syncRepository(repoId: string, url: string, branch: string, auth: AuthConfig): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('sync_repository', { repoId, url, branch, auth })
}
```

- [ ] **Step 4: Commit**

```bash
git add src/api/tauri.ts
git commit -m "feat: add fetchBranches and modify API functions for branch support"
```

---

### Task 14: 重构 AddRepoDialog.vue - Step 1

**Files:**
- Modify: `src/views/AddRepoDialog.vue`

- [ ] **Step 1: 修改 template 部分**

将整个 template 替换为 4 步流程：

```vue
<template>
  <t-drawer
    v-model:visible="visible"
    :header="'添加仓库'"
    :footer="true"
    size="480px"
    @close="onClose"
  >
    <!-- Steps indicator -->
    <t-steps :current="currentStep" class="steps-indicator">
      <t-step-item title="输入地址" />
      <t-step-item title="选择分支" />
      <t-step-item title="选择技能" />
      <t-step-item title="确认" />
    </t-steps>

    <!-- Step 0: Input URL + Auth -->
    <div v-if="currentStep === 0" class="step-content">
      <t-form :data="formData" :rules="formRules" ref="formRef" label-align="top">
        <t-form-item label="仓库 URL" name="url">
          <t-input v-model="formData.url" placeholder="https://github.com/user/repo" />
        </t-form-item>

        <t-form-item label="认证方式">
          <t-select v-model="formData.authType">
            <t-option value="none" label="无需认证" />
            <t-option value="token" label="令牌" />
            <t-option value="username-password" label="用户名和密码" />
          </t-select>
        </t-form-item>

        <t-form-item v-if="formData.authType === 'token'" label="访问令牌">
          <t-input v-model="formData.token" type="password" placeholder="ghp_xxx or glpat_xxx" />
        </t-form-item>

        <template v-if="formData.authType === 'username-password'">
          <t-form-item label="用户名">
            <t-input v-model="formData.username" placeholder="用户名" />
          </t-form-item>
          <t-form-item label="密码">
            <t-input v-model="formData.password" type="password" placeholder="密码" />
          </t-form-item>
        </template>
      </t-form>
    </div>

    <!-- Step 1: Select Branch -->
    <div v-if="currentStep === 1" class="step-content">
      <div v-if="loadingBranches" class="loading-container">
        <t-loading text="正在获取分支..." />
      </div>

      <t-alert v-else-if="branchError" theme="error" :message="branchError">
        <template #operation>
          <t-link theme="primary" @click="retryFetchBranches">重试</t-link>
        </template>
      </t-alert>

      <div v-else>
        <t-form label-align="top">
          <t-form-item label="选择分支">
            <t-select v-model="formData.branch" :options="branchOptions" />
          </t-form-item>
        </t-form>
      </div>
    </div>

    <!-- Step 2: Select Skills -->
    <div v-if="currentStep === 2" class="step-content">
      <div v-if="loading" class="loading-container">
        <t-loading text="正在获取仓库中的技能..." />
      </div>

      <t-alert v-else-if="error" theme="error" :message="error">
        <template #operation>
          <t-link theme="primary" @click="retryFetchSkills">重试</t-link>
        </template>
      </t-alert>

      <div v-else class="skills-panel">
        <div class="skills-header">
          <span>{{ skills.length }} 个技能</span>
          <t-link theme="primary" @click="selectAll">
            {{ selectedSkills.length === skills.length ? '取消全选' : '全选' }}
          </t-link>
        </div>
        <t-checkbox-group v-model="selectedSkills" class="skills-list">
          <t-checkbox
            v-for="skill in skills"
            :key="skill.path"
            :value="skill.path"
            class="skill-item"
          >
            <div class="skill-info">
              <span class="skill-name">{{ skill.name }}</span>
              <span class="skill-desc">{{ skill.description || '暂无描述' }}</span>
            </div>
          </t-checkbox>
        </t-checkbox-group>
      </div>
    </div>

    <!-- Step 3: Summary -->
    <div v-if="currentStep === 3" class="step-content">
      <t-descriptions title="汇总" :column="2" bordered>
        <t-descriptions-item label="名称">{{ generatedName }}</t-descriptions-item>
        <t-descriptions-item label="地址" :span="2">
          <code class="url-code">{{ formData.url }}</code>
        </t-descriptions-item>
        <t-descriptions-item label="分支">{{ formData.branch }}</t-descriptions-item>
        <t-descriptions-item label="认证">
          <t-tag theme="primary" variant="light">
            {{ formData.authType === 'none' ? '无' : formData.authType === 'token' ? '令牌' : '用户/密码' }}
          </t-tag>
        </t-descriptions-item>
        <t-descriptions-item label="已选技能" :span="2">
          <div class="selected-skills">
            <t-tag
              v-for="path in selectedSkills.slice(0, 5)"
              :key="path"
              theme="default"
              variant="light"
            >
              {{ getSkillName(path) }}
            </t-tag>
            <t-tag v-if="selectedSkills.length > 5" theme="primary" variant="light">
              +{{ selectedSkills.length - 5 }} 更多
            </t-tag>
          </div>
        </t-descriptions-item>
      </t-descriptions>
    </div>

    <!-- Footer buttons -->
    <template #footer>
      <div class="dialog-footer">
        <t-button variant="outline" @click="onCancel">
          {{ currentStep === 0 ? '取消' : '返回' }}
        </t-button>
        <t-button
          v-if="currentStep === 0"
          theme="primary"
          :loading="loadingBranches"
          :disabled="!formData.url"
          @click="onFetchBranches"
        >
          获取分支
        </t-button>
        <t-button
          v-else-if="currentStep < 3"
          theme="primary"
          :loading="currentStep === 1 && loading"
          :disabled="currentStep === 2 && selectedSkills.length === 0"
          @click="onConfirm"
        >
          继续
        </t-button>
        <t-button
          v-else
          theme="primary"
          @click="onSave"
        >
          保存仓库
        </t-button>
      </div>
    </template>
  </t-drawer>
</template>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AddRepoDialog.vue
git commit -m "feat: refactor AddRepoDialog template to 4-step flow"
```

---

### Task 15: 重构 AddRepoDialog.vue - Script 部分

**Files:**
- Modify: `src/views/AddRepoDialog.vue`

- [ ] **Step 1: 修改 script 部分**

```vue
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import { fetchBranches, fetchRepoSkills, type SkillMeta, type AuthConfig } from '@/api/tauri'
import { v4 as uuidv4 } from 'uuid'

const configStore = useConfigStore()
const skillsStore = useSkillsStore()

const visible = defineModel<boolean>('visible', { default: false })

const currentStep = ref(0)
const loadingBranches = ref(false)
const branchError = ref<string | null>(null)
const branches = ref<string[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const skills = ref<SkillMeta[]>([])
const selectedSkills = ref<string[]>([])
const formRef = ref()

const formData = ref({
  url: '',
  branch: '',
  authType: 'none',
  token: '',
  username: '',
  password: '',
})

const formRules = {
  url: [{ required: true, message: 'URL is required' }],
}

const branchOptions = computed(() => 
  branches.value.map(b => ({ label: b, value: b }))
)

// Generate name from URL and branch
const generatedName = computed(() => {
  const url = formData.value.url
  const branch = formData.value.branch
  
  // Parse URL: https://github.com/owner/repo -> owner/repo
  let name = url
    .replace(/^https?:\/\//, '')
    .replace(/^git@/, '')
    .replace(/\.git$/, '')
  
  // Remove domain
  const parts = name.split('/')
  if (parts.length >= 3) {
    name = parts.slice(2).join('/')
  }
  
  return `${name}(${branch})`
})

// Reset state when dialog opens
watch(visible, (val) => {
  if (val) {
    currentStep.value = 0
    loadingBranches.value = false
    branchError.value = null
    branches.value = []
    loading.value = false
    error.value = null
    skills.value = []
    selectedSkills.value = []
    formData.value = {
      url: '',
      branch: '',
      authType: 'none',
      token: '',
      username: '',
      password: '',
    }
  }
})

function getSkillName(path: string): string {
  const skill = skills.value.find(s => s.path === path)
  return skill?.name || path
}

function selectAll() {
  if (selectedSkills.value.length === skills.value.length) {
    selectedSkills.value = []
  } else {
    selectedSkills.value = skills.value.map(s => s.path)
  }
}

function getAuthConfig(): AuthConfig {
  return {
    type: formData.value.authType as 'none' | 'token' | 'username-password',
    token: formData.value.authType === 'token' ? formData.value.token : undefined,
    username: formData.value.authType === 'username-password' ? formData.value.username : undefined,
    password: formData.value.authType === 'username-password' ? formData.value.password : undefined,
  }
}

async function onFetchBranches() {
  const valid = await formRef.value?.validate()
  if (valid !== true) return
  
  loadingBranches.value = true
  branchError.value = null
  
  try {
    const auth = getAuthConfig()
    branches.value = await fetchBranches(formData.value.url, auth)
    if (branches.value.length > 0) {
      formData.value.branch = branches.value[0]
      currentStep.value = 1
    } else {
      branchError.value = '未找到任何分支'
    }
  } catch (e) {
    branchError.value = String(e)
  } finally {
    loadingBranches.value = false
  }
}

function retryFetchBranches() {
  branchError.value = null
  onFetchBranches()
}

async function fetchSkillsPreview() {
  loading.value = true
  error.value = null
  try {
    const auth = getAuthConfig()
    skills.value = await fetchRepoSkills(formData.value.url, formData.value.branch, auth)
    selectedSkills.value = skills.value.map(s => s.path)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function retryFetchSkills() {
  error.value = null
  fetchSkillsPreview()
}

async function onConfirm() {
  if (currentStep.value === 1) {
    // Go to fetch skills
    currentStep.value = 2
    await fetchSkillsPreview()
  } else if (currentStep.value === 2) {
    if (selectedSkills.value.length === 0) {
      MessagePlugin.warning('请至少选择一个技能')
      return
    }
    currentStep.value = 3
  }
}

async function onSave() {
  const auth = getAuthConfig()
  configStore.addRepository({
    id: uuidv4(),
    name: generatedName.value,
    url: formData.value.url,
    branch: formData.value.branch,
    auth,
    sync_interval: 3600,
    selected_skills: selectedSkills.value,
    last_sync: null,
    enabled: true,
  })
  MessagePlugin.success('仓库添加成功')
  
  // Select the new repo
  skillsStore.setCurrentRepo(configStore.config?.repositories?.slice(-1)[0]?.id || null)
  
  visible.value = false
}

function onCancel() {
  if (currentStep.value === 0) {
    visible.value = false
  } else {
    currentStep.value--
  }
}

function onClose() {
  visible.value = false
}
</script>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AddRepoDialog.vue
git commit -m "feat: refactor AddRepoDialog script for 4-step flow"
```

---

### Task 16: 添加设置页面路由

**Files:**
- Modify: `src/router.ts`

- [ ] **Step 1: 添加 settings 路由**

```typescript
// skills-manager/src/router.ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/Home.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
    },
  ],
})

export default router
```

- [ ] **Step 2: Commit**

```bash
git add src/router.ts
git commit -m "feat: add settings route"
```

---

### Task 17: 创建 Settings.vue 页面

**Files:**
- Create: `src/views/Settings.vue`

- [ ] **Step 1: 创建 Settings.vue 文件**

```vue
<template>
  <div class="settings-container">
    <t-card title="设置" :bordered="false">
      <template #actions>
        <t-button variant="outline" @click="handleBack">
          返回主页
        </t-button>
      </template>

      <!-- Agent 配置区域 -->
      <div class="settings-section">
        <div class="section-title">Agent 配置</div>
        <div v-if="!configStore.config?.agents?.length" class="empty-tip">
          暂无已配置的 Agent
        </div>
        <div v-else class="agent-list">
          <div v-for="agent in configStore.config?.agents" :key="agent.id" class="agent-item">
            <div class="agent-info">
              <span class="agent-name">{{ agent.name }}</span>
              <code class="agent-path">{{ agent.path }}</code>
            </div>
            <t-switch
              :value="agent.enabled"
              @change="(value: boolean) => configStore.updateAgent(agent.id, value)"
            />
          </div>
        </div>
      </div>

      <!-- 同步设置区域 -->
      <div class="settings-section">
        <div class="section-title">同步设置</div>
        <div class="settings-row">
          <div class="settings-info">
            <span class="settings-label">自动同步</span>
            <span class="settings-desc">自动同步仓库</span>
          </div>
          <t-switch
            :value="configStore.config?.settings.auto_sync"
            @change="(value: boolean) => configStore.updateSettings({ auto_sync: value })"
          />
        </div>
        <div class="settings-row">
          <div class="settings-info">
            <span class="settings-label">默认同步间隔</span>
            <span class="settings-desc">自动同步的频率</span>
          </div>
          <t-select
            :value="configStore.config?.settings.default_sync_interval || 3600"
            @change="(value: number) => configStore.updateSettings({ default_sync_interval: value })"
            :options="syncIntervalOptions"
            style="width: 140px"
          />
        </div>
      </div>

      <!-- 版本信息区域 -->
      <div class="settings-section">
        <div class="section-title">版本信息</div>
        <div class="version-info">
          <div class="info-row">
            <span class="label">当前版本：</span>
            <span class="value">0.1.0</span>
          </div>
        </div>
      </div>

      <!-- 关于区域 -->
      <div class="settings-section about-section">
        <div class="about-brand">
          <span class="about-icon">◈</span>
          <span class="about-name">Skills Manager</span>
        </div>
        <p class="about-desc">AI Agent 的知识管理系统</p>
      </div>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'

const router = useRouter()
const configStore = useConfigStore()

const syncIntervalOptions = [
  { label: '5 分钟', value: 300 },
  { label: '15 分钟', value: 900 },
  { label: '30 分钟', value: 1800 },
  { label: '1 小时', value: 3600 },
  { label: '2 小时', value: 7200 },
  { label: '6 小时', value: 21600 },
  { label: '12 小时', value: 43200 },
  { label: '每天', value: 86400 },
]

function handleBack() {
  router.push('/')
}
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

.settings-section {
  margin-bottom: 32px;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-secondarycontainer);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  display: block;
}

.agent-path {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background: var(--td-bg-color-specialcomponent);
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-block;
  margin-top: 4px;
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-secondarycontainer);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
  margin-bottom: 8px;
}

.settings-info {
  flex: 1;
}

.settings-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  display: block;
}

.settings-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  display: block;
  margin-top: 2px;
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

.empty-tip {
  color: var(--td-text-color-placeholder);
  font-size: 14px;
  padding: 16px 0;
}

.about-section {
  text-align: center;
  padding: 32px 0;
}

.about-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 8px;
}

.about-icon {
  color: var(--td-brand-color);
  font-size: 24px;
}

.about-name {
  font-size: 18px;
  font-weight: 700;
  color: var(--td-text-color-primary);
}

.about-desc {
  font-size: 14px;
  color: var(--td-text-color-secondary);
  margin: 0;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: create Settings page component"
```

---

### Task 18: 修改 App.vue 设置按钮跳转

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 添加 router 导入和使用**

在 script 部分：

```vue
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import { useSyncStore } from '@/stores/sync'
import { checkGitInstalled, syncAllRepositories } from '@/api/tauri'
import GitInstallDialog from '@/components/GitInstallDialog.vue'
import AddRepoDialog from '@/views/AddRepoDialog.vue'
import {
  SearchIcon,
  SettingIcon,
  FolderIcon,
  AddIcon,
  RefreshIcon,
} from 'tdesign-icons-vue-next'

const router = useRouter()
const configStore = useConfigStore()
const skillsStore = useSkillsStore()
const syncStore = useSyncStore()
```

- [ ] **Step 2: 修改设置按钮点击事件**

在 template 中修改 header 设置按钮：

```vue
<t-button
  variant="outline"
  shape="circle"
  @click="router.push('/settings')"
>
  <setting-icon />
</t-button>
```

- [ ] **Step 3: 删除 showSettings 相关代码**

删除：
- `const showSettings = ref(false)`
- SettingsDialog 组件导入（如果还有）
- `<SettingsDialog v-model:visible="showSettings" />`

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: modify settings button to navigate to settings page"
```

---

### Task 19: 删除 SettingsDialog.vue

**Files:**
- Delete: `src/components/SettingsDialog.vue`

- [ ] **Step 1: 删除文件**

```bash
git rm src/components/SettingsDialog.vue
git commit -m "refactor: remove SettingsDialog component, replaced by Settings page"
```

---

### Task 20: 配置 NSIS 安装器

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 修改 bundle 配置**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Skill Manager",
  "version": "0.1.0",
  "identifier": "com.skill-manager.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Skill Manager",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
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

- [ ] **Step 2: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: configure NSIS installer with custom template"
```

---

### Task 21: 创建 NSIS 模板文件

**Files:**
- Create: `src-tauri/nsis/installer.nsi`

- [ ] **Step 1: 创建 nsis 目录和 installer.nsi 文件**

基于 Tauri 默认模板，添加自定义安装路径：

创建文件内容（使用 Tauri 模板变量，仅修改关键部分）：

```nsis
; ... (复制 ssh-tunnel-manager 的完整模板内容)

Function .onInit
  ; ... (保持其他初始化代码)

  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    ; Set default install location - custom directory
    StrCpy $INSTDIR "D:\Programs\${PRODUCTNAME}"

    Call RestorePreviousInstallLocation
  ${EndIf}
FunctionEnd

; ... (保持其余模板内容)
```

注意：由于模板较长，实际实现时从 ssh-tunnel-manager 复制完整模板。

- [ ] **Step 2: Commit**

```bash
git add src-tauri/nsis/
git commit -m "feat: add NSIS installer template with custom install path"
```

---

### Task 22: 验证构建

**Files:**
- None (verification only)

- [ ] **Step 1: 运行 cargo check**

```bash
cd skills-manager/src-tauri && cargo check
```

Expected: No warnings

- [ ] **Step 2: 运行前端构建**

```bash
cd skills-manager && pnpm build
```

Expected: Build successful

- [ ] **Step 3: Commit (if fixes needed)**

如果需要修复任何问题，提交修复。

---

## 完成检查

完成后验证：
1. `cargo check` 无警告
2. `pnpm build` 成功
3. AddRepoDialog 支持分支选择
4. Settings 页面正常工作
5. 日志系统初始化正常