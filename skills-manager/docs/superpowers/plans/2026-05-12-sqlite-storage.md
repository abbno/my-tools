# SQLite Storage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace JSON config storage with SQLite database, storing all data in executable directory with skills persistence.

**Architecture:** Create db module with connection management, schema, and CRUD operations for each table. Modify existing commands to use db module instead of JSON. Add skills commands for persistence.

**Tech Stack:** rusqlite (bundled feature), Tauri commands, Pinia stores

---

## File Structure

**Create:**
- `src-tauri/src/db/mod.rs` - Module exports and init
- `src-tauri/src/db/connection.rs` - Database connection singleton
- `src-tauri/src/db/schema.rs` - Table creation and migrations
- `src-tauri/src/db/repositories.rs` - Repository CRUD
- `src-tauri/src/db/agents.rs` - Agent CRUD
- `src-tauri/src/db/settings.rs` - Settings CRUD
- `src-tauri/src/db/skills.rs` - Skill CRUD

**Modify:**
- `src-tauri/Cargo.toml` - Add rusqlite dependency
- `src-tauri/src/lib.rs` - Register new commands, init db
- `src-tauri/src/commands/config.rs` - Use db module
- `src-tauri/src/commands/repo.rs` - Update skills table on sync
- `src-tauri/src/models/config.rs` - Add SkillMeta id and is_selected
- `src/api/tauri.ts` - Add getSkills, updateSkillSelection APIs
- `src/stores/skills.ts` - Handle is_selected field

---

### Task 1: Add rusqlite Dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add rusqlite to dependencies**

```toml
# Add after line 18 (after log = "0.4")
rusqlite = { version = "0.32", features = ["bundled"] }
```

- [ ] **Step 2: Verify dependency resolves**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully, rusqlite downloaded

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add rusqlite dependency for SQLite storage"
```

---

### Task 2: Create db Module Structure

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/connection.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create db module directory**

Run: `mkdir -p src-tauri/src/db`
Expected: Directory created

- [ ] **Step 2: Create connection.rs - Database path and connection**

```rust
// src-tauri/src/db/connection.rs
use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;
use once_cell::sync::Lazy;

static DB_CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let db_path = get_db_path().expect("Failed to get db path");
    let conn = Connection::open(&db_path).expect("Failed to open database");
    Mutex::new(conn)
});

pub fn get_db_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get exe directory")?;
    Ok(exe_dir.join("skills-manager.db"))
}

pub fn get_connection() -> Result<'static, MutexGuard<'static, Connection>, String> {
    DB_CONNECTION.lock().map_err(|e| format!("Failed to get db lock: {}", e))
}
```

- [ ] **Step 3: Create mod.rs - Module exports**

```rust
// src-tauri/src/db/mod.rs
pub mod connection;
pub mod schema;
pub mod repositories;
pub mod agents;
pub mod settings;
pub mod skills;

use connection::get_connection;
use schema::init_schema;

pub fn init_database() -> Result<(), String> {
    let conn = get_connection()?;
    init_schema(&conn)?;
    init_default_data(&conn)?;
    Ok(())
}

fn init_default_data(conn: &Connection) -> Result<(), String> {
    // Will be implemented in schema.rs
    schema::init_default_settings(conn)?;
    schema::init_default_agents(conn)?;
    Ok(())
}
```

- [ ] **Step 4: Add db module to lib.rs**

```rust
// src-tauri/src/lib.rs - Add after line 7 (after pub mod logger;)
pub mod db;
```

- [ ] **Step 5: Verify module compiles**

Run: `cd src-tauri && cargo check`
Expected: May fail due to missing files, that's expected

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/db/connection.rs src-tauri/src/lib.rs
git commit -m "feat: create db module structure"
```

---

### Task 3: Create schema.rs - Table Definitions

**Files:**
- Create: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: Create schema.rs with all table definitions**

```rust
// src-tauri/src/db/schema.rs
use rusqlite::Connection;
use crate::models::Agent;

pub fn init_schema(conn: &Connection) -> Result<(), String> {
    create_repositories_table(conn)?;
    create_agents_table(conn)?;
    create_settings_table(conn)?;
    create_skills_table(conn)?;
    Ok(())
}

fn create_repositories_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repositories (
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
        )",
        [],
    ).map_err(|e| format!("Failed to create repositories table: {}", e))?;
    Ok(())
}

fn create_agents_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1
        )",
        [],
    ).map_err(|e| format!("Failed to create agents table: {}", e))?;
    Ok(())
}

fn create_settings_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("Failed to create settings table: {}", e))?;
    Ok(())
}

fn create_skills_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            repo_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            path TEXT NOT NULL,
            local_path TEXT NOT NULL,
            is_selected INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| format!("Failed to create skills table: {}", e))?;
    Ok(())
}

pub fn init_default_settings(conn: &Connection) -> Result<(), String> {
    let defaults = [
        ("default_sync_interval", "3600"),
        ("auto_sync", "true"),
        ("check_interval", "300"),
    ];

    for (key, value) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        ).map_err(|e| format!("Failed to insert default setting: {}", e))?;
    }
    Ok(())
}

pub fn init_default_agents(conn: &Connection) -> Result<(), String> {
    let default_agents = Agent::default_agents();
    for agent in default_agents {
        conn.execute(
            "INSERT OR IGNORE INTO agents (id, name, path, enabled) VALUES (?1, ?2, ?3, ?4)",
            [agent.id, agent.name, agent.path, if agent.enabled { "1" } else { "0" }],
        ).map_err(|e| format!("Failed to insert default agent: {}", e))?;
    }
    Ok(())
}
```

- [ ] **Step 2: Add MutexGuard import to connection.rs**

```rust
// src-tauri/src/db/connection.rs - Add after use statements
use std::sync::MutexGuard;
```

- [ ] **Step 3: Verify schema compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/schema.rs src-tauri/src/db/connection.rs
git commit -m "feat: add database schema definitions"
```

---

### Task 4: Create repositories.rs CRUD

**Files:**
- Create: `src-tauri/src/db/repositories.rs`

- [ ] **Step 1: Create repositories.rs**

```rust
// src-tauri/src/db/repositories.rs
use rusqlite::{Connection, Row};
use crate::models::{Repository, AuthConfig};
use chrono::{DateTime, Utc};

fn row_to_repository(row: &Row) -> Result<Repository, rusqlite::Error> {
    let auth_type: String = row.get(4)?;
    let auth_token: Option<String> = row.get(5)?;
    let auth_username: Option<String> = row.get(6)?;
    let auth_password: Option<String> = row.get(7)?;
    let last_sync_str: Option<String> = row.get(9)?;
    let enabled_int: i32 = row.get(10)?;

    let last_sync = last_sync_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

    Ok(Repository {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        branch: row.get(3)?,
        auth: AuthConfig {
            auth_type,
            token: auth_token,
            username: auth_username,
            password: auth_password,
        },
        sync_interval: row.get::<_, i64>(8)? as u64,
        selected_skills: Vec::new(), // Will be populated from skills table
        last_sync,
        enabled: enabled_int != 0,
    })
}

pub fn get_all(conn: &Connection) -> Result<Vec<Repository>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, branch, auth_type, auth_token, auth_username, auth_password, sync_interval, last_sync, enabled FROM repositories"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let repos = stmt.query_map([], row_to_repository)
        .map_err(|e| format!("Failed to query repositories: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map repositories: {}", e))?;

    Ok(repos)
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Repository>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, branch, auth_type, auth_token, auth_username, auth_password, sync_interval, last_sync, enabled FROM repositories WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt.query_row([id], row_to_repository).ok();
    Ok(result)
}

pub fn insert(conn: &Connection, repo: &Repository) -> Result<(), String> {
    let last_sync_str = repo.last_sync.map(|dt| dt.to_rfc3339());
    conn.execute(
        "INSERT INTO repositories (id, name, url, branch, auth_type, auth_token, auth_username, auth_password, sync_interval, last_sync, enabled) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        [
            &repo.id,
            &repo.name,
            &repo.url,
            &repo.branch,
            &repo.auth.auth_type,
            repo.auth.token.as_ref().unwrap_or(&"".to_string()),
            repo.auth.username.as_ref().unwrap_or(&"".to_string()),
            repo.auth.password.as_ref().unwrap_or(&"".to_string()),
            &repo.sync_interval.to_string(),
            last_sync_str.as_ref().unwrap_or(&"".to_string()),
            if repo.enabled { "1" } else { "0" },
        ],
    ).map_err(|e| format!("Failed to insert repository: {}", e))?;
    Ok(())
}

pub fn update(conn: &Connection, repo: &Repository) -> Result<(), String> {
    let last_sync_str = repo.last_sync.map(|dt| dt.to_rfc3339());
    conn.execute(
        "UPDATE repositories SET name = ?2, url = ?3, branch = ?4, auth_type = ?5, auth_token = ?6, auth_username = ?7, auth_password = ?8, sync_interval = ?9, last_sync = ?10, enabled = ?11 WHERE id = ?1",
        [
            &repo.id,
            &repo.name,
            &repo.url,
            &repo.branch,
            &repo.auth.auth_type,
            repo.auth.token.as_ref().unwrap_or(&"".to_string()),
            repo.auth.username.as_ref().unwrap_or(&"".to_string()),
            repo.auth.password.as_ref().unwrap_or(&"".to_string()),
            &repo.sync_interval.to_string(),
            last_sync_str.as_ref().unwrap_or(&"".to_string()),
            if repo.enabled { "1" } else { "0" },
        ],
    ).map_err(|e| format!("Failed to update repository: {}", e))?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM repositories WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Failed to delete repository: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/repositories.rs
git commit -m "feat: add repositories CRUD operations"
```

---

### Task 5: Create agents.rs CRUD

**Files:**
- Create: `src-tauri/src/db/agents.rs`

- [ ] **Step 1: Create agents.rs**

```rust
// src-tauri/src/db/agents.rs
use rusqlite::{Connection, Row};
use crate::models::Agent;

fn row_to_agent(row: &Row) -> Result<Agent, rusqlite::Error> {
    let enabled_int: i32 = row.get(3)?;
    Ok(Agent {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        enabled: enabled_int != 0,
    })
}

pub fn get_all(conn: &Connection) -> Result<Vec<Agent>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, enabled FROM agents"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let agents = stmt.query_map([], row_to_agent)
        .map_err(|e| format!("Failed to query agents: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map agents: {}", e))?;

    Ok(agents)
}

pub fn insert(conn: &Connection, agent: &Agent) -> Result<(), String> {
    conn.execute(
        "INSERT INTO agents (id, name, path, enabled) VALUES (?1, ?2, ?3, ?4)",
        [&agent.id, &agent.name, &agent.path, if agent.enabled { "1" } else { "0" }],
    ).map_err(|e| format!("Failed to insert agent: {}", e))?;
    Ok(())
}

pub fn update(conn: &Connection, agent: &Agent) -> Result<(), String> {
    conn.execute(
        "UPDATE agents SET name = ?2, path = ?3, enabled = ?4 WHERE id = ?1",
        [&agent.id, &agent.name, &agent.path, if agent.enabled { "1" } else { "0" }],
    ).map_err(|e| format!("Failed to update agent: {}", e))?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM agents WHERE id = ?1",
        [id],
    ).map_err(|e| format!("Failed to delete agent: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/agents.rs
git commit -m "feat: add agents CRUD operations"
```

---

### Task 6: Create settings.rs CRUD

**Files:**
- Create: `src-tauri/src/db/settings.rs`

- [ ] **Step 1: Create settings.rs**

```rust
// src-tauri/src/db/settings.rs
use rusqlite::Connection;
use crate::models::Settings;

pub fn get_all(conn: &Connection) -> Result<Settings, String> {
    let default_sync_interval = get_value(conn, "default_sync_interval")?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(3600);

    let auto_sync = get_value(conn, "auto_sync")?
        .map(|v| v == "true")
        .unwrap_or(true);

    let check_interval = get_value(conn, "check_interval")?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300);

    Ok(Settings {
        default_sync_interval,
        auto_sync,
        check_interval,
    })
}

fn get_value(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get setting: {}", e)),
    }
}

pub fn set_value(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, value],
    ).map_err(|e| format!("Failed to set setting: {}", e))?;
    Ok(())
}

pub fn update_settings(conn: &Connection, settings: &Settings) -> Result<(), String> {
    set_value(conn, "default_sync_interval", &settings.default_sync_interval.to_string())?;
    set_value(conn, "auto_sync", if settings.auto_sync { "true" } else { "false" })?;
    set_value(conn, "check_interval", &settings.check_interval.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/settings.rs
git commit -m "feat: add settings CRUD operations"
```

---

### Task 7: Create skills.rs CRUD

**Files:**
- Create: `src-tauri/src/db/skills.rs`
- Modify: `src-tauri/src/models/config.rs`

- [ ] **Step 1: Update SkillMeta model to include id and is_selected**

```rust
// src-tauri/src/models/config.rs - Add after SkillMeta struct definition (or create if not exists)
// First, check if SkillMeta is defined elsewhere. It's likely in models module.

// Add to src-tauri/src/models/mod.rs or create new file:
// src-tauri/src/models/skill.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub local_path: String,
    pub is_selected: bool,
}
```

- [ ] **Step 2: Add skill model to models/mod.rs**

```rust
// src-tauri/src/models/mod.rs - Add export
pub mod skill;
pub use skill::SkillMeta;
```

- [ ] **Step 3: Create skills.rs CRUD**

```rust
// src-tauri/src/db/skills.rs
use rusqlite::{Connection, Row};
use crate::models::SkillMeta;
use uuid::Uuid;

fn row_to_skill(row: &Row) -> Result<SkillMeta, rusqlite::Error> {
    let is_selected_int: i32 = row.get(6)?;
    Ok(SkillMeta {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        path: row.get(4)?,
        local_path: row.get(5)?,
        is_selected: is_selected_int != 0,
    })
}

pub fn get_by_repo(conn: &Connection, repo_id: &str) -> Result<Vec<SkillMeta>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, repo_id, name, description, path, local_path, is_selected FROM skills WHERE repo_id = ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let skills = stmt.query_map([repo_id], row_to_skill)
        .map_err(|e| format!("Failed to query skills: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map skills: {}", e))?;

    Ok(skills)
}

pub fn get_selected_by_repo(conn: &Connection, repo_id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare(
        "SELECT path FROM skills WHERE repo_id = ?1 AND is_selected = 1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let paths = stmt.query_map([repo_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query selected skills: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to map paths: {}", e))?;

    Ok(paths)
}

pub fn insert(conn: &Connection, skill: &SkillMeta) -> Result<(), String> {
    conn.execute(
        "INSERT INTO skills (id, repo_id, name, description, path, local_path, is_selected) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [
            &skill.id,
            &skill.repo_id,
            &skill.name,
            &skill.description,
            &skill.path,
            &skill.local_path,
            if skill.is_selected { "1" } else { "0" },
        ],
    ).map_err(|e| format!("Failed to insert skill: {}", e))?;
    Ok(())
}

pub fn update_selection(conn: &Connection, skill_id: &str, is_selected: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE skills SET is_selected = ?2 WHERE id = ?1",
        [skill_id, if is_selected { "1" } else { "0" }],
    ).map_err(|e| format!("Failed to update skill selection: {}", e))?;
    Ok(())
}

pub fn clear_by_repo(conn: &Connection, repo_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM skills WHERE repo_id = ?1",
        [repo_id],
    ).map_err(|e| format!("Failed to clear repo skills: {}", e))?;
    Ok(())
}

pub fn sync_skills(conn: &Connection, repo_id: &str, skills: &[SkillMeta], selected_paths: &[String]) -> Result<(), String> {
    // Clear existing skills for this repo
    clear_by_repo(conn, repo_id)?;

    // Insert new skills with selection status
    for skill in skills {
        let is_selected = selected_paths.contains(&skill.path);
        let skill_with_selection = SkillMeta {
            id: Uuid::new_v4().to_string(),
            is_selected,
            ..skill.clone()
        };
        insert(conn, &skill_with_selection)?;
    }

    Ok(())
}
```

- [ ] **Step 4: Verify compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/skills.rs src-tauri/src/models/skill.rs src-tauri/src/models/mod.rs
git commit -m "feat: add skills CRUD and SkillMeta model"
```

---

### Task 8: Modify commands/config.rs

**Files:**
- Modify: `src-tauri/src/commands/config.rs`

- [ ] **Step 1: Rewrite config.rs to use db module**

```rust
// src-tauri/src/commands/config.rs
use crate::models::{Config, Repository, Agent, Settings, SkillMeta};
use crate::db::{connection::get_connection, repositories, agents, settings, skills};

#[tauri::command]
pub fn read_config() -> Result<Config, String> {
    let conn = get_connection()?;

    let repos = repositories::get_all(&conn)?;
    let agents = agents::get_all(&conn)?;
    let settings = settings::get_all(&conn)?;

    // Populate selected_skills for each repository
    let repos_with_skills = repos.into_iter().map(|repo| {
        let selected_paths = skills::get_selected_by_repo(&conn, &repo.id).unwrap_or_default();
        Repository {
            selected_skills: selected_paths,
            ..repo
        }
    }).collect();

    Ok(Config {
        repositories: repos_with_skills,
        agents,
        settings,
    })
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    let conn = get_connection()?;

    // Clear and re-insert repositories
    for repo in &config.repositories {
        if repositories::get_by_id(&conn, &repo.id)?.is_some() {
            repositories::update(&conn, repo)?;
        } else {
            repositories::insert(&conn, repo)?;
        }
    }

    // Update agents
    for agent in &config.agents {
        if agents::get_by_id(&conn, &agent.id)?.is_some() {
            agents::update(&conn, agent)?;
        } else {
            agents::insert(&conn, agent)?;
        }
    }

    // Update settings
    settings::update_settings(&conn, &config.settings)?;

    Ok(())
}

#[tauri::command]
pub fn get_skills(repo_id: String) -> Result<Vec<SkillMeta>, String> {
    let conn = get_connection()?;
    skills::get_by_repo(&conn, &repo_id)
}

#[tauri::command]
pub fn update_skill_selection(skill_id: String, is_selected: bool) -> Result<(), String> {
    let conn = get_connection()?;
    skills::update_selection(&conn, &skill_id, is_selected)
}

#[tauri::command]
pub fn clear_repo_skills(repo_id: String) -> Result<(), String> {
    let conn = get_connection()?;
    skills::clear_by_repo(&conn, &repo_id)
}
```

- [ ] **Step 2: Add get_by_id to agents.rs**

```rust
// src-tauri/src/db/agents.rs - Add this function
pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Agent>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, enabled FROM agents WHERE id = ?1"
    ).map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = stmt.query_row([id], row_to_agent).ok();
    Ok(result)
}
```

- [ ] **Step 3: Verify compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/config.rs src-tauri/src/db/agents.rs
git commit -m "feat: modify config commands to use SQLite"
```

---

### Task 9: Modify commands/repo.rs

**Files:**
- Modify: `src-tauri/src/commands/repo.rs`

- [ ] **Step 1: Update sync_repository to save skills**

```rust
// src-tauri/src/commands/repo.rs - Modify sync_repository function
use crate::db::{connection::get_connection, skills};
use crate::models::SkillMeta;
use uuid::Uuid;

#[tauri::command]
pub fn sync_repository(repo_id: String, url: String, branch: String, auth: AuthConfig, selected_skills: Vec<String>) -> Result<Vec<SkillMeta>, String> {
    info!("Syncing repository {} branch {}", repo_id, branch);

    let repo_path = get_repo_path(&repo_id)?;

    if is_git_repo(&repo_path) {
        let checkout_result = checkout_branch(&repo_path, &branch);
        if !checkout_result.success {
            error!("Checkout failed: {}", checkout_result.message);
            return Err(checkout_result.message);
        }

        let result = pull_repo(&repo_path, &auth);
        if !result.success {
            error!("Pull failed: {}", result.message);
            return Err(result.message);
        }
        info!("Pull successful");
    } else {
        let result = clone_repo(&url, &branch, &repo_path, &auth);
        if !result.success {
            error!("Clone failed: {}", result.message);
            return Err(result.message);
        }
        info!("Clone successful");
    }

    // Scan for skills
    info!("Scanning skills in {}", repo_path.to_string_lossy());
    let scanned_skills = scan_skills(&repo_path, &repo_id);
    info!("Found {} skills", scanned_skills.len());

    // Convert to SkillMeta with full data
    let skills_with_paths: Vec<SkillMeta> = scanned_skills.into_iter().map(|s| {
        let local_path = repo_path.join(&s.path).to_string_lossy().to_string();
        SkillMeta {
            id: Uuid::new_v4().to_string(),
            repo_id: repo_id.clone(),
            name: s.name,
            description: s.description,
            path: s.path,
            local_path,
            is_selected: selected_skills.contains(&s.path),
        }
    }).collect();

    // Save to database
    let conn = get_connection()?;
    skills::sync_skills(&conn, &repo_id, &skills_with_paths, &selected_skills)?;

    Ok(skills_with_paths)
}
```

- [ ] **Step 2: Update skill_parser to return basic skill info**

```rust
// src-tauri/src/skill_parser.rs - Modify SkillMeta struct (if it exists there)
// Or ensure scan_skills returns compatible data
```

- [ ] **Step 3: Verify compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/repo.rs
git commit -m "feat: update sync_repository to persist skills"
```

---

### Task 10: Register Commands and Initialize DB

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add db initialization and register new commands**

```rust
// src-tauri/src/lib.rs - Modify setup and invoke_handler
use tauri::Manager;
use scheduler::start_scheduler;
use db::init_database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            // Initialize database
            init_database().expect("Failed to initialize database");

            // Initialize logger
            logger::init_logger(app.handle())?;

            // Start background scheduler
            start_scheduler(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_git_installed,
            commands::get_system_info,
            commands::read_config,
            commands::save_config,
            commands::get_skills,
            commands::update_skill_selection,
            commands::clear_repo_skills,
            commands::fetch_branches,
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Update models/mod.rs exports**

```rust
// src-tauri/src/models/mod.rs
mod config;
mod skill;

pub use config::*;
pub use skill::SkillMeta;
```

- [ ] **Step 3: Verify full build**

Run: `cd src-tauri && cargo build`
Expected: Builds successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/models/mod.rs
git commit -m "feat: register SQLite commands and initialize DB on startup"
```

---

### Task 11: Update Frontend API

**Files:**
- Modify: `src/api/tauri.ts`
- Modify: `src/stores/skills.ts`

- [ ] **Step 1: Add new API functions to tauri.ts**

```typescript
// src/api/tauri.ts - Add after existing Config interface

// Update SkillMeta interface
export interface SkillMeta {
  id: string
  repo_id: string
  name: string
  description: string
  path: string
  local_path: string
  is_selected: boolean
}

// Add new API functions after existing ones
export async function getSkills(repoId: string): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('get_skills', { repoId })
}

export async function updateSkillSelection(skillId: string, isSelected: boolean): Promise<void> {
  return invoke<void>('update_skill_selection', { skillId, isSelected })
}
```

- [ ] **Step 2: Update syncRepository to accept selected_skills**

```typescript
// src/api/tauri.ts - Modify existing function
export async function syncRepository(repoId: string, url: string, branch: string, auth: AuthConfig, selectedSkills: string[] = []): Promise<SkillMeta[]> {
  return invoke<SkillMeta[]>('sync_repository', { repoId, url, branch, auth, selectedSkills })
}
```

- [ ] **Step 3: Update skills.ts store to handle is_selected**

```typescript
// src/stores/skills.ts - Modify SkillMeta interface and addSkills
export interface SkillMeta {
  id: string
  repo_id: string
  name: string
  description: string
  path: string
  local_path: string
  is_selected: boolean
}

export const useSkillsStore = defineStore('skills', () => {
  const skills = ref<SkillMeta[]>([])
  const currentRepoId = ref<string | null>(null)
  const selectedSkill = ref<SkillMeta | null>(null)
  const searchQuery = ref('')

  function setSkills(skillsList: SkillMeta[]) {
    skills.value = skillsList
  }

  function addSkills(skillsList: SkillMeta[]) {
    const existingRepoIds = skillsList.map(s => s.repo_id)
    skills.value = skills.value.filter(s => !existingRepoIds.includes(s.repo_id))
    skills.value = [...skills.value, ...skillsList]
  }

  function updateSkillIsSelected(skillId: string, isSelected: boolean) {
    const skill = skills.value.find(s => s.id === skillId)
    if (skill) {
      skill.is_selected = isSelected
    }
  }

  // ... rest unchanged
})
```

- [ ] **Step 4: Commit frontend changes**

```bash
git add src/api/tauri.ts src/stores/skills.ts
git commit -m "feat: update frontend API for SQLite skills"
```

---

### Task 12: Update App.vue Startup Flow

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Modify startup to use getSkills from database**

```typescript
// src/App.vue - Modify imports
import { checkGitInstalled, syncRepository, getSkills } from '@/api/tauri'

// Modify onMounted
onMounted(async () => {
  // Load config (from SQLite)
  await configStore.loadConfig()

  // Load skills from database (no need to sync again)
  if (configStore.config?.repositories?.length) {
    for (const repo of configStore.config.repositories) {
      try {
        const skills = await getSkills(repo.id)
        skillsStore.addSkills(skills)
      } catch (e) {
        console.error('Failed to load skills for repo:', repo.name, e)
      }
    }
    // Select first repo if none selected
    if (!skillsStore.currentRepoId && configStore.config.repositories.length > 0) {
      skillsStore.setCurrentRepo(configStore.config.repositories[0].id)
    }
  }

  // ... rest unchanged (git check, event listener)
})
```

- [ ] **Step 2: Verify frontend builds**

Run: `cd skills-manager && npm run build`
Expected: Builds successfully

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: update App.vue to load skills from SQLite"
```

---

### Task 13: Update AddRepoDialog.vue

**Files:**
- Modify: `src/views/AddRepoDialog.vue`

- [ ] **Step 1: Update onSave to pass selected_skills to syncRepository**

```typescript
// src/views/AddRepoDialog.vue - Modify onSave function
async function onSave() {
  const auth = getAuthConfig()
  const repoId = uuidv4()

  // Create repo entry
  const newRepo = {
    id: repoId,
    name: generatedName.value,
    url: formData.value.url,
    branch: formData.value.branch,
    auth,
    sync_interval: 3600,
    selected_skills: selectedSkills.value,
    last_sync: null,
    enabled: true,
  }

  configStore.addRepository(newRepo)

  // Sync with selected skills
  syncStore.startSync(repoId)
  try {
    const skills = await syncRepository(repoId, formData.value.url, formData.value.branch, auth, selectedSkills.value)
    skillsStore.addSkills(skills)
    syncStore.endSync(repoId)
    MessagePlugin.success('仓库添加成功')
  } catch (e) {
    syncStore.setError(repoId, String(e))
    MessagePlugin.warning('仓库已添加，但同步失败：' + String(e))
  }

  skillsStore.setCurrentRepo(repoId)
  visible.value = false
}
```

- [ ] **Step 2: Commit**

```bash
git add src/views/AddRepoDialog.vue
git commit -m "feat: update AddRepoDialog to persist skill selection"
```

---

### Task 14: Integration Test

- [ ] **Step 1: Build full application**

Run: `cd skills-manager && npm run tauri build`
Expected: Builds successfully, creates executable

- [ ] **Step 2: Run application and verify**

1. Launch the built executable
2. Verify `skills-manager.db` is created in executable directory
3. Add a repository
4. Verify skills are persisted after restart

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete SQLite storage implementation"
```