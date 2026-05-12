# Skills Manager SQLite 存储改造设计

## 背景

当前 skills-manager 使用 `~/.skill-manager/config.json` 存储配置数据。需要改造为 SQLite 数据库存储，并将数据库文件放置在可执行文件所在目录。

## 目标

1. 将 config.json 数据迁移到 SQLite 数据库
2. 数据库文件存储在可执行文件所在目录，命名为 `skills-manager.db`
3. 新增 skills 表持久化技能数据，包含本地路径和选择状态
4. 不迁移旧数据，全新开始

## 技术方案

使用 **rusqlite** 作为 SQLite 库：
- 纯 Rust 实现，无需额外系统依赖
- 静态链接，编译简单
- 与 Tauri 项目风格一致

## 数据库表结构

### repositories 表

```sql
CREATE TABLE repositories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    branch TEXT NOT NULL,
    auth_type TEXT NOT NULL DEFAULT 'none',
    auth_token TEXT,
    auth_username TEXT,
    auth_password TEXT,
    sync_interval INTEGER NOT NULL DEFAULT 3600,
    last_sync TEXT,
    enabled INTEGER NOT NULL DEFAULT 1
);
```

### agents 表

```sql
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1
);
```

### settings 表

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

默认值：
- `default_sync_interval = 3600`
- `auto_sync = true`
- `check_interval = 300`

### skills 表

```sql
CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    repo_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    path TEXT NOT NULL,
    local_path TEXT NOT NULL,
    is_selected INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
);
```

字段说明：
- `id`: 主键 UUID
- `repo_id`: 所属仓库 ID
- `name`: 技能名称
- `description`: 技能描述
- `path`: 技能目录名（如 "brainstorming"）
- `local_path`: 技能完整本地路径
- `is_selected`: 是否被用户选中（0/1）

## 文件结构

新增 `src-tauri/src/db/` 目录：

```
src-tauri/src/db/
├── mod.rs          # 导出和初始化
├── connection.rs   # 数据库连接管理
├── schema.rs       # 表结构定义和迁移
├── repositories.rs # repositories CRUD 操作
├── agents.rs       # agents CRUD 操作
├── settings.rs     # settings CRUD 操作
├── skills.rs       # skills CRUD 操作
```

修改现有文件：
- `commands/config.rs` - 调用 db 模块而非直接读写 JSON
- `commands/repo.rs` - 同步时更新 skills 表

## 依赖

Cargo.toml 添加：

```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```

## API 设计

### 前端 API（保持不变）

```typescript
// src/api/tauri.ts
export async function readConfig(): Promise<Config>
export async function saveConfig(config: Config): Promise<void>
```

### 后端 Commands

**config.rs（调整实现）**
- `read_config()` - 从 SQLite 读取所有表数据，组装 Config 结构体
- `save_config()` - 拆分数据写入各表

**新增 skills commands**
```rust
pub fn get_skills(repo_id: String) -> Result<Vec<SkillMeta>, String>
pub fn update_skill_selection(skill_id: String, is_selected: bool) -> Result<(), String>
pub fn clear_repo_skills(repo_id: String) -> Result<(), String>
```

**repo.rs 变更**

`sync_repository()` 同步完成后：
1. 清空该 repo 的旧 skills
2. 插入新扫描到的 skills
3. 保留用户选择状态（通过 selected_skills 参数恢复）

## 数据库初始化

### 启动流程

1. 获取可执行文件所在目录
2. 创建 `skills-manager.db` 文件
3. 执行 schema 创建表
4. 初始化默认数据

### 路径获取

```rust
fn get_db_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get exe directory")?;
    Ok(exe_dir.join("skills-manager.db"))
}
```

### 默认数据

- **settings**: 插入默认配置值
- **agents**: 插入默认 7 个 Agent（Claude Code, Codex CLI, Gemini CLI, Trae, OpenCode, Trae CN, Central）

## 前端变更

### config.ts Pinia Store

保持现有方法签名，新增：

```typescript
async loadSkills(repoId: string): Promise<void>
async toggleSkillSelection(skillId: string, isSelected: boolean): Promise<void>
```

### App.vue 启动流程

```typescript
onMounted(async () => {
  await configStore.loadConfig()

  if (configStore.config?.repositories?.length) {
    for (const repo of configStore.config.repositories) {
      const skills = await getSkills(repo.id)
      skillsStore.addSkills(skills)
    }
  }
  // ...其余不变
})
```

### skills.ts Pinia Store

- `addSkills()` 方法需处理 `is_selected` 字段
- 已选中的技能需要标记或过滤

## 实现步骤

1. 添加 rusqlite 依赖
2. 创建 db 模块（connection, schema, CRUD）
3. 修改 config.rs 使用 db 模块
4. 修改 repo.rs 更新 skills 表
5. 注册新的 Tauri commands
6. 调整前端调用逻辑
7. 测试完整流程