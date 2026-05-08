---
title: Skill Manager 实施计划
date: 2026-05-06
spec: docs/superpowers/specs/2026-05-04-skill-manager-design.md
---

# Skill Manager 实施计划

## 实施概述

本文档将设计文档中的 5 个开发阶段分解为可执行的实施步骤，每个步骤包含具体的文件修改和验收标准。

---

## 阶段 1：基础框架

### 步骤 1.1：初始化 Tauri + Vue3 + TDesign 项目

**目标**：创建完整的项目骨架，配置开发环境

**文件修改**：
- 创建 `package.json` - 配置 Vue3、TDesign、Pinia 依赖
- 创建 `vite.config.ts` - Vite 构建配置
- 创建 `src/main.ts` - Vue 应用入口
- 创建 `src/App.vue` - 根组件，包含路由容器
- 创建 `src-tauri/tauri.conf.json` - Tauri 配置（窗口尺寸、权限）
- 创建 `src-tauri/src/main.rs` - Rust 入口，注册所有 command
- 创建 `src-tauri/src/lib.rs` - Tauri 应用构建
- 创建 `src-tauri/Cargo.toml` - Rust 依赖配置

**验收标准**：
- 运行 `pnpm tauri dev` 能启动应用窗口
- 窗口显示空白页面，无报错

### 步骤 1.2：实现 Git 检测功能

**目标**：启动时检测 Git 是否安装，未安装显示引导对话框

**文件修改**：
- 创建 `src-tauri/src/commands/system.rs` - check_git_installed、get_system_info
- 修改 `src-tauri/src/main.rs` - 注册 system commands
- 创建 `src/api/tauri.ts` - invoke 封装，包含 checkGitInstalled、getSystemInfo
- 创建 `src/components/GitInstallDialog.vue` - Git 未安装引导对话框
- 修改 `src/App.vue` - 启动时调用检测，条件显示对话框

**验收标准**：
- 已安装 Git：应用正常启动，不显示对话框
- 未安装 Git：显示引导对话框，包含安装命令和下载链接
- 点击"已安装，重新检测"能重新检测并关闭对话框

### 步骤 1.3：实现配置读写

**目标**：实现 config.json 的读取和保存功能

**文件修改**：
- 创建 `src-tauri/src/models/config.rs` - Config、Repository、Agent、Settings 结构体
- 创建 `src-tauri/src/commands/config.rs` - read_config、save_config
- 创建 `src-tauri/src/config_manager.rs` - 配置文件路径、默认配置生成
- 修改 `src-tauri/src/main.rs` - 注册 config commands
- 创建 `src/stores/config.ts` - Pinia store，管理配置状态
- 修改 `src/api/tauri.ts` - 添加 readConfig、saveConfig

**验收标准**：
- 首次启动：创建 `~/.skill-manager/config.json`，包含默认配置
- 再次启动：读取已有配置，状态正确加载
- 修改配置后保存：config.json 文件更新

### 步骤 1.4：实现主界面布局

**目标**：搭建主界面三栏布局骨架

**文件修改**：
- 创建 `src/views/Home.vue` - 主界面，三栏布局容器
- 创建 `src/components/RepoList.vue` - 左侧仓库列表（占位）
- 创建 `src/components/SkillList.vue` - 右侧左栏技能列表（占位）
- 创建 `src/components/SkillDetail.vue` - 右侧右栏技能详情（占位）
- 创建 `src/stores/skills.ts` - 技能数据状态（占位）
- 修改 `src/App.vue` - 添加路由，Home 作为默认页面

**验收标准**：
- 应用启动显示三栏布局
- 左侧栏宽度约 200px，显示仓库列表占位
- 右侧左栏宽度约 300px，显示技能列表占位
- 右侧右栏显示技能详情占位

---

## 阶段 2：仓库管理

### 步骤 2.1：实现添加仓库对话框

**目标**：完成添加仓库的完整流程（输入 → 预览 → 选择 → 保存）

**文件修改**：
- 创建 `src/views/AddRepoDialog.vue` - 添加仓库对话框
  - 步骤 1：输入表单（名称、URL、认证方式、同步间隔）
  - 步骤 2：预览技能列表（左侧可勾选，右侧详情）
  - 步骤 3：保存到配置
- 创建 `src/components/AuthFields.vue` - 认证方式输入组件
- 修改 `src/stores/config.ts` - 添加 addRepository 方法
- 修改 `src/api/tauri.ts` - 添加 fetchRepoSkills

**验收标准**：
- 点击"添加仓库"按钮打开对话框
- 输入 URL 和认证信息后，点击"下一步"能预览技能列表
- 勾选技能后，点击"保存"能添加到配置并刷新仓库列表

### 步骤 2.2：实现 Git clone/pull

**目标**：封装 Git 操作，支持 clone 和 pull

**文件修改**：
- 创建 `src-tauri/src/git.rs` - Git 操作封装
  - clone_repo(url, path, auth) - 克隆仓库
  - pull_repo(path, auth) - 拉取更新
  - build_auth_env(auth) - 构建认证环境变量
- 创建 `src-tauri/src/commands/repo.rs` - fetch_repo_skills、sync_repository
- 修改 `src-tauri/src/main.rs` - 注册 repo commands
- 修改 `src/api/tauri.ts` - 添加 syncRepository

**验收标准**：
- 添加新仓库：能成功 clone 到 `~/.skill-manager/repos/{repo-id}/`
- 已有仓库：能成功 pull 更新
- 认证仓库：能使用 Token 或用户名密码认证
- 失败时：返回错误信息，前端显示错误提示

### 步骤 2.3：实现 SKILL.md 解析

**目标**：解析技能目录中的 SKILL.md，提取元数据

**文件修改**：
- 创建 `src-tauri/src/skill_parser.rs` - SKILL.md 解析
  - parse_skill_md(content) - 解析 frontmatter，提取 name、description
  - scan_skills(repo_path) - 扫描仓库目录，返回所有技能元数据
- 创建 `src-tauri/src/models/skill.rs` - SkillMeta 结构体
- 修改 `src-tauri/src/commands/repo.rs` - fetch_repo_skills 返回技能列表

**验收标准**：
- 能正确解析包含 frontmatter 的 SKILL.md
- 能扫描仓库目录，返回所有技能的元数据列表
- 无 SKILL.md 或格式错误的目录被跳过

### 步骤 2.4：完善仓库列表和技能列表

**目标**：让仓库列表和技能列表显示真实数据

**文件修改**：
- 修改 `src/components/RepoList.vue` - 显示仓库列表，支持点击切换
- 修改 `src/components/SkillList.vue` - 显示当前仓库的技能列表
- 修改 `src/components/SkillDetail.vue` - 显示选中技能的详情
- 修改 `src/stores/skills.ts` - 管理当前仓库的技能数据
- 修改 `src/views/Home.vue` - 添加"添加仓库"按钮，集成各组件

**验收标准**：
- 仓库列表显示所有已添加的仓库
- 点击仓库切换，技能列表显示对应技能
- 点击技能，详情区显示 SKILL.md 内容预览

---

## 阶段 3：软连接管理

### 步骤 3.1：实现软连接创建/删除

**目标**：跨平台软连接管理（Windows Junction/Symlink，macOS/Linux Symlink）

**文件修改**：
- 创建 `src-tauri/src/symlink.rs` - 软连接操作
  - create_symlink(src, target) - 创建软连接
  - remove_symlink(target) - 删除软连接
  - is_symlink(path) - 检查是否为软连接
  - windows_create_junction(src, target) - Windows Junction 实现
- 创建 `src-tauri/src/commands/symlink.rs` - create_symlink、remove_symlink、check_symlinks
- 修改 `src-tauri/src/main.rs` - 注册 symlink commands
- 创建 `src-tauri/src/models/symlink.rs` - SymlinkStatus 结构体
- 修改 `src/api/tauri.ts` - 添加 createSymlink、removeSymlink、checkSymlinks

**验收标准**：
- Windows：优先创建 Junction，失败时返回需要管理员权限的错误
- macOS/Linux：创建标准 symlink
- 已存在的目标：先删除再创建
- 能正确删除软连接（不删除源文件）

### 步骤 3.2：实现 Agent 目录检测和部署

**目标**：检测 Agent 目录是否存在，部署技能到启用的 Agent

**文件修改**：
- 创建 `src-tauri/src/commands/deploy.rs` - deploy_skill、undeploy_skill
  - deploy_skill(skill_name, repo_id, agents) - 部署技能到多个 Agent
  - undeploy_skill(skill_name) - 从所有 Agent 移除技能
- 创建 `src-tauri/src/agent_manager.rs` - Agent 目录管理
  - ensure_agent_dir(agent_path) - 确保 Agent 目录存在
  - get_agent_skill_path(agent_path, skill_name) - 获取技能目标路径
- 修改 `src-tauri/src/main.rs` - 注册 deploy commands
- 修改 `src/api/tauri.ts` - 添加 deploySkill、undeploySkill
- 修改 `src/stores/config.ts` - 添加 deploySkill、undeploySkill 方法

**验收标准**：
- 部署技能：在启用的 Agent 目录创建软连接
- Agent 目录不存在：自动创建
- 移除技能：删除所有 Agent 目录中的软连接

### 步骤 3.3：实现冲突处理

**目标**：同名技能冲突时弹窗提示用户选择

**文件修改**：
- 创建 `src/components/ConflictDialog.vue` - 冲突处理对话框
  - 显示冲突技能名称、各仓库版本信息
  - 选项：选择仓库版本、保留现有版本、跳过
- 修改 `src-tauri/src/commands/deploy.rs` - deploy_skill 返回冲突信息
- 创建 `src-tauri/src/models/conflict.rs` - ConflictInfo 结构体
- 修改 `src/stores/skills.ts` - 添加冲突检测逻辑

**验收标准**：
- 部署时发现同名技能：弹出冲突对话框
- 用户选择仓库版本：替换现有软连接
- 用户选择保留现有：跳过此技能
- 用户选择跳过：不部署此技能

---

## 阶段 4：定时同步

### 步骤 4.1：实现后台定时任务

**目标**：Rust 后端定时检查同步需求

**文件修改**：
- 创建 `src-tauri/src/scheduler.rs` - 定时任务调度
  - start_scheduler(app_handle) - 启动定时器
  - check_sync_needed(config) - 检查是否需要同步
  - spawn_sync_task(repo_id) - 异步执行同步
- 修改 `src-tauri/src/lib.rs` - 应用启动时启动调度器
- 创建 `src-tauri/src/commands/sync.rs` - sync_all_repositories、get_sync_status
- 修改 `src-tauri/src/main.rs` - 注册 sync commands

**验收标准**：
- 应用启动后，每 5 分钟检查是否需要同步
- 需要 sync 的仓库自动执行 pull
- 同步完成后更新 lastSync 时间

### 步骤 4.2：实现同步进度显示

**目标**：前端显示同步状态和进度

**文件修改**：
- 创建 `src/stores/sync.ts` - 同步状态管理
  - syncing: boolean - 是否正在同步
  - progress: Map<repo_id, SyncProgress> - 各仓库同步进度
- 修改 `src/components/RepoList.vue` - 显示同步状态图标
- 创建 `src/components/SyncProgress.vue` - 同步进度条
- 修改 `src/views/Home.vue` - 添加"立即同步全部"按钮
- 修改 `src/api/tauri.ts` - 添加事件监听（sync-start、sync-progress、sync-complete）

**验收标准**：
- 同步时：仓库列表显示同步图标
- 同步完成：图标消失，显示成功提示
- 失败时：显示错误信息

### 步骤 4.3：完善错误处理

**目标**：优雅处理各类错误，提供重试机制

**文件修改**：
- 修改 `src-tauri/src/git.rs` - 添加重试逻辑（网络错误自动重试 3 次）
- 修改 `src-tauri/src/symlink.rs` - 权限错误返回特定错误码
- 创建 `src/components/ErrorDialog.vue` - 错误提示对话框
- 修改 `src/stores/sync.ts` - 错误状态管理

**验收标准**：
- 网络错误：自动重试 3 次，间隔递增
- 认证失败：提示检查配置
- 权限问题：提示以管理员权限运行
- Git 失败：显示错误信息，允许手动重试

---

## 阶段 5：完善优化

### 步骤 5.1：实现设置页面

**目标**：Agent 配置、自动同步开关、默认同步间隔

**文件修改**：
- 创建 `src/views/Settings.vue` - 设置页面
  - Agent 启用/禁用配置
  - 自动同步开关
  - 默认同步间隔选择
- 修改 `src/App.vue` - 添加设置页面路由
- 修改 `src/components/RepoList.vue` - 添加设置入口按钮
- 修改 `src/stores/config.ts` - 添加 updateSettings 方法

**验收标准**：
- 点击设置按钮进入设置页面
- Agent 配置：勾选启用/禁用，保存后生效
- 自动同步：关闭后不再自动同步
- 默认同步间隔：修改后影响新添加的仓库

### 步骤 5.2：实现搜索过滤

**目标**：按技能名称、描述搜索过滤

**文件修改**：
- 修改 `src/views/Home.vue` - 添加搜索输入框
- 修改 `src/components/SkillList.vue` - 支持过滤显示
- 修改 `src/stores/skills.ts` - 添加 searchQuery 状态

**验收标准**：
- 输入搜索词：技能列表实时过滤
- 搜索匹配名称和描述
- 清空搜索词：恢复完整列表

### 步骤 5.3：测试和修复

**目标**：全面测试，修复发现的问题

**测试清单**：
- Git 检测：已安装/未安装场景
- 仓库添加：公开仓库/私有仓库认证
- 技能同步：首次 clone/后续 pull
- 软连接：Windows Junction/macOS symlink
- 冲突处理：同名技能冲突场景
- 定时同步：自动触发/手动触发
- 设置保存：配置持久化

**验收标准**：
- 所有测试清单项通过
- 无明显 bug 和性能问题

---

## 实施顺序建议

按阶段顺序实施，每个阶段完成后进行阶段性验收：

1. **阶段 1** → 验收：应用能启动，显示主界面骨架
2. **阶段 2** → 验收：能添加仓库，显示技能列表
3. **阶段 3** → 验收：能部署技能到 Agent 目录
4. **阶段 4** → 验收：能自动同步，显示进度
5. **阶段 5** → 验收：功能完整，体验优化

每个步骤建议独立提交，便于追踪和回滚。