# Tool Update Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a version management web service in Rust that allows uploading and distributing software installation packages.

**Architecture:** Actix-web handles HTTP routing, Askama renders HTML templates, local file directory stores packages and JSON metadata.

**Tech Stack:** Rust, Actix-web, Askama, serde, chrono, tokio

---

## File Structure

```
tool-update-server/
├── Cargo.toml                    # Project config and dependencies
├── src/
│   ├── main.rs                   # Entry point and route setup
│   ├── models.rs                 # Data structures (App, Version, ChangelogItem)
│   ├── storage.rs                # File I/O and JSON metadata operations
│   └── handlers.rs               # HTTP request handlers
├── templates/
│   ├── base.html                 # Base template with common layout
│   ├── index.html                # Home page - app list
│   ├── app_detail.html           # App detail page - version list
│   └── new_app.html              # New app creation page
│   └── upload.html               # Upload modal/partial
│   └── error.html                # Error page
├── static/
│   └── style.css                 # CSS styles (dark theme, industrial)
└── data/                         # Runtime data directory (auto-created)
    ├── apps.json
    └── {app_id}/
        ├── versions.json
        └── files/
            └── {filename}
```

---

## Task 1: Project Initialization

**Files:**
- Create: `Cargo.toml`

- [ ] **Step 1: Create Cargo.toml with dependencies**

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

- [ ] **Step 2: Create src directory structure**

Run: `mkdir -p src templates static`

- [ ] **Step 3: Create placeholder main.rs**

```rust
fn main() {
    println!("Tool Update Server starting...");
}
```

- [ ] **Step 4: Verify project compiles**

Run: `cargo check`
Expected: Compiles successfully with no errors

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: project initialization with dependencies"
```

---

## Task 2: Data Models

**Files:**
- Create: `src/models.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create models.rs with data structures**

```rust
use serde::{Deserialize, Serialize};

/// Software/App information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub app_id: String,
    pub version: String,
    pub release_date: String,
    pub changelog: Vec<ChangelogItem>,
    pub download_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub created_at: String,
}

/// Changelog item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub description: String,
}

/// Create app request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppRequest {
    pub id: String,
    pub name: String,
}

/// Version info for API response (client-compatible format)
#[derive(Debug, Clone, Serialize)]
pub struct VersionInfoResponse {
    pub version: String,
    pub release_date: String,
    pub download_url: String,
    pub changelog: Vec<ChangelogItem>,
}

impl From<Version> for VersionInfoResponse {
    fn from(v: Version) -> Self {
        Self {
            version: v.version,
            release_date: v.release_date,
            download_url: v.download_url,
            changelog: v.changelog,
        }
    }
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

```rust
mod models;

fn main() {
    println!("Tool Update Server starting...");
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/models.rs src/main.rs
git commit -m "feat: add data models for App, Version, ChangelogItem"
```

---

## Task 3: Storage Layer

**Files:**
- Create: `src/storage.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create storage.rs with file operations**

```rust
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::models::{App, Version};

const APPS_FILE: &str = "apps.json";
const VERSIONS_FILE: &str = "versions.json";
const FILES_DIR: &str = "files";

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.data_dir)?;
        let apps_file = self.data_dir.join(APPS_FILE);
        if !apps_file.exists() {
            self.save_apps(&[])?;
        }
        Ok(())
    }

    // Apps operations
    pub fn load_apps(&self) -> std::io::Result<Vec<App>> {
        let path = self.data_dir.join(APPS_FILE);
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let apps: Vec<App> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(apps)
    }

    pub fn save_apps(&self, apps: &[App]) -> std::io::Result<()> {
        let path = self.data_dir.join(APPS_FILE);
        let content = serde_json::to_string_pretty(apps)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_app(&self, app_id: &str) -> std::io::Result<Option<App>> {
        let apps = self.load_apps()?;
        Ok(apps.iter().find(|a| a.id == app_id).cloned())
    }

    pub fn add_app(&self, app: App) -> std::io::Result<()> {
        let apps = self.load_apps()?;
        let mut apps = apps;
        apps.push(app);
        self.save_apps(&apps)?;
        // Create app directory structure
        let app_dir = self.data_dir.join(&app.id);
        fs::create_dir_all(&app_dir)?;
        fs::create_dir_all(app_dir.join(FILES_DIR))?;
        self.save_versions(&app.id, &[])?;
        Ok(())
    }

    // Versions operations
    pub fn load_versions(&self, app_id: &str) -> std::io::Result<Vec<Version>> {
        let path = self.data_dir.join(app_id).join(VERSIONS_FILE);
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let versions: Vec<Version> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(versions)
    }

    pub fn save_versions(&self, app_id: &str, versions: &[Version]) -> std::io::Result<()> {
        let path = self.data_dir.join(app_id).join(VERSIONS_FILE);
        let content = serde_json::to_string_pretty(versions)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_latest_version(&self, app_id: &str) -> std::io::Result<Option<Version>> {
        let versions = self.load_versions(app_id)?;
        Ok(versions.first().cloned())
    }

    pub fn get_version(&self, app_id: &str, version: &str) -> std::io::Result<Option<Version>> {
        let versions = self.load_versions(app_id)?;
        Ok(versions.iter().find(|v| v.version == version).cloned())
    }

    pub fn add_version(&self, version: Version) -> std::io::Result<()> {
        let versions = self.load_versions(&version.app_id)?;
        let mut versions = versions;
        // Insert at beginning (newest first)
        versions.insert(0, version);
        self.save_versions(&version.app_id, &versions)?;
        Ok(())
    }

    pub fn delete_version(&self, app_id: &str, version: &str) -> std::io::Result<bool> {
        let versions = self.load_versions(app_id)?;
        let mut versions = versions;
        let pos = versions.iter().position(|v| v.version == version);
        if let Some(pos) = pos {
            let deleted = versions.remove(pos);
            // Delete file
            let file_path = self.data_dir.join(app_id).join(FILES_DIR).join(&deleted.file_name);
            if file_path.exists() {
                fs::remove_file(&file_path)?;
            }
            self.save_versions(app_id, &versions)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_files_dir(&self, app_id: &str) -> PathBuf {
        self.data_dir.join(app_id).join(FILES_DIR)
    }
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

```rust
mod models;
mod storage;

fn main() {
    println!("Tool Update Server starting...");
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/storage.rs src/main.rs
git commit -m "feat: add storage layer for apps and versions JSON files"
```

---

## Task 4: HTTP Handlers

**Files:**
- Create: `src/handlers.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create handlers.rs with route handlers**

```rust
use actix_web::{web, HttpResponse, Error};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use std::path::PathBuf;
use crate::models::{App, Version, ChangelogItem, CreateAppRequest, VersionInfoResponse};
use crate::storage::Storage;
use chrono::Utc;

pub struct AppState {
    pub storage: Storage,
}

// API: Get latest version for client
pub async fn get_latest_version(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let app_id = path.into_inner();
    
    match state.storage.get_latest_version(&app_id) {
        Ok(Some(version)) => {
            let response: VersionInfoResponse = version.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("App or version not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Get specific version
pub async fn get_version(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (app_id, version) = path.into_inner();
    
    match state.storage.get_version(&app_id, &version) {
        Ok(Some(v)) => {
            let response: VersionInfoResponse = v.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("Version not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Download file
pub async fn download_file(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<NamedFile, Error> {
    let (app_id, filename) = path.into_inner();
    let file_path = state.storage.get_files_dir(&app_id).join(&filename);
    NamedFile::open(file_path).map_err(|e| e.into())
}

// API: Get all apps (JSON)
pub async fn get_apps(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.load_apps() {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Create new app
pub async fn create_app(
    state: web::Data<AppState>,
    body: web::Json<CreateAppRequest>,
) -> HttpResponse {
    let req = body.into_inner();
    let app = App {
        id: req.id,
        name: req.name,
        created_at: Utc::now().to_rfc3339(),
    };
    
    match state.storage.add_app(app) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Get app versions
pub async fn get_app_versions(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let app_id = path.into_inner();
    
    match state.storage.load_versions(&app_id) {
        Ok(versions) => HttpResponse::Ok().json(versions),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// API: Upload new version with file
pub async fn upload_version(
    state: web::Data<AppState>,
    path: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let app_id = path.into_inner();
    
    // Collect form data
    let mut version_num: Option<String> = None;
    let mut changelog: Vec<ChangelogItem> = vec![];
    let mut file_name: Option<String> = None;
    let mut file_size: u64 = 0;
    
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let name = field.name().to_string();
        
        if name == "version" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk?);
            }
            version_num = Some(String::from_utf8_lossy(&data).to_string());
        } else if name == "changelog" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                data.extend_from_slice(&chunk?);
            }
            let log_str = String::from_utf8_lossy(&data).to_string();
            // Parse changelog JSON
            if let Ok(items) = serde_json::from_str::<Vec<ChangelogItem>>(&log_str) {
                changelog = items;
            }
        } else if name == "file" {
            // Save file
            file_name = Some(field.content_filename().unwrap_or("unknown").to_string());
            let files_dir = state.storage.get_files_dir(&app_id);
            let file_path = files_dir.join(field.content_filename().unwrap_or("unknown"));
            let mut f = std::fs::File::create(&file_path)?;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file_size += data.len() as u64;
                f.write_all(&data)?;
            }
        }
    }
    
    if let (Some(version_num), Some(file_name)) = (version_num, file_name) {
        let version = Version {
            app_id: app_id.clone(),
            version: version_num,
            release_date: Utc::now().format("%Y-%m-%d").to_string(),
            changelog,
            download_url: format!("/download/{}/{}", app_id, file_name),
            file_name,
            file_size,
            created_at: Utc::now().to_rfc3339(),
        };
        
        state.storage.add_version(version)?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().body("Missing version or file"))
    }
}

// API: Delete version
pub async fn delete_version(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (app_id, version) = path.into_inner();
    
    match state.storage.delete_version(&app_id, &version) {
        Ok(true) => HttpResponse::Ok().finish(),
        Ok(false) => HttpResponse::NotFound().body("Version not found"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

```rust
mod models;
mod storage;
mod handlers;

fn main() {
    println!("Tool Update Server starting...");
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/handlers.rs src/main.rs
git commit -m "feat: add HTTP handlers for API routes"
```

---

## Task 5: Main Entry and Route Setup

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace main.rs with full implementation**

```rust
mod models;
mod storage;
mod handlers;

use actix_web::{App, HttpServer, web};
use actix_files::Files;
use std::path::PathBuf;
use crate::handlers::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get data directory (default: ./data relative to executable)
    let exe_dir = std::env::current_exe()
        .expect("Cannot get executable path")
        .parent()
        .expect("Cannot get executable directory")
        .to_path_buf();
    
    let data_dir = exe_dir.join("data");
    
    // Initialize storage
    let storage = storage::Storage::new(data_dir.clone());
    storage.init()?;
    
    println!("Tool Update Server starting...");
    println!("Data directory: {}", data_dir.display());
    println!("Server running at http://localhost:8080");
    
    // Create app state
    let app_state = web::Data::new(AppState { storage });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Static files
            .service(Files::new("/static", exe_dir.join("static")).show_files_listing())
            // API routes for client
            .route("/api/version/{app_id}", web::get().to(handlers::get_latest_version))
            .route("/api/version/{app_id}/{version}", web::get().to(handlers::get_version))
            .route("/download/{app_id}/{filename}", web::get().to(handlers::download_file))
            // API routes for management
            .route("/api/apps", web::get().to(handlers::get_apps))
            .route("/api/apps", web::post().to(handlers::create_app))
            .route("/api/apps/{app_id}/versions", web::get().to(handlers::get_app_versions))
            .route("/api/apps/{app_id}/versions", web::post().to(handlers::upload_version))
            .route("/api/apps/{app_id}/versions/{version}", web::delete().to(handlers::delete_version))
            // Health check
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: complete main.rs with route setup and server startup"
```

---

## Task 6: CSS Styles

**Files:**
- Create: `static/style.css`

- [ ] **Step 1: Create style.css with dark industrial theme**

```css
/* Base styles */
:root {
    --bg-primary: #0a0a0f;
    --bg-card: #15151f;
    --bg-hover: #1a1a2a;
    --accent: #00ff88;
    --accent-hover: #00cc6a;
    --secondary: #3d3d5c;
    --border: #2a2a3a;
    --text-primary: #ffffff;
    --text-secondary: #8888aa;
    --text-muted: #5a5a7a;
    --danger: #ff4444;
}

* {
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    margin: 0;
    padding: 0;
    min-height: 100vh;
}

/* Container */
.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 24px;
}

/* Header */
.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 32px;
}

.header-title {
    font-size: 24px;
    font-weight: 700;
    letter-spacing: 2px;
}

.header-title::before {
    content: "▓▓ ";
    color: var(--accent);
}

/* Buttons */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    border: 1px solid var(--border);
    background: var(--bg-card);
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    border-radius: 4px;
}

.btn:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
}

.btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
}

.btn-primary:hover {
    background: var(--accent-hover);
}

.btn-danger {
    color: var(--danger);
}

.btn-danger:hover {
    border-color: var(--danger);
}

.btn-sm {
    padding: 8px 16px;
    font-size: 12px;
}

/* Cards */
.card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 16px;
    transition: all 0.2s ease;
}

.card:hover {
    transform: translateY(-2px);
    border-color: var(--accent);
    box-shadow: 0 4px 20px rgba(0, 255, 136, 0.1);
}

.card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 16px;
}

.card-title {
    font-size: 18px;
    font-weight: 600;
}

.card-title::before {
    content: "████ ";
    color: var(--accent);
    opacity: 0.5;
}

.card-meta {
    display: flex;
    gap: 24px;
    color: var(--text-secondary);
    font-size: 13px;
}

.card-meta-item {
    display: flex;
    gap: 8px;
}

.card-meta-label {
    color: var(--text-muted);
}

.card-meta-value {
    color: var(--text-primary);
}

.card-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
}

/* Status indicator */
.status-active {
    display: inline-flex;
    align-items: center;
    gap: 6px;
}

.status-active::before {
    content: "●";
    color: var(--accent);
    animation: pulse 2s infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

/* Version list */
.version-list {
    margin-top: 24px;
}

.version-item {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 12px;
}

.version-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
}

.version-number {
    font-size: 16px;
    font-weight: 600;
    color: var(--accent);
}

.version-date {
    color: var(--text-secondary);
}

.changelog-list {
    margin: 12px 0;
}

.changelog-item {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 14px;
}

.changelog-icon {
    color: var(--accent);
}

.file-info {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 8px;
}

/* Forms */
.form-group {
    margin-bottom: 20px;
}

.form-label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 8px;
    color: var(--text-secondary);
}

.form-input {
    width: 100%;
    padding: 12px 16px;
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 14px;
    border-radius: 4px;
    transition: border-color 0.2s;
}

.form-input:focus {
    outline: none;
    border-color: var(--accent);
}

/* File upload */
.file-drop {
    border: 2px dashed var(--border);
    padding: 32px;
    text-align: center;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
}

.file-drop:hover {
    border-color: var(--accent);
    background: var(--bg-hover);
}

.file-drop-text {
    color: var(--text-secondary);
}

/* Links */
a {
    color: var(--accent);
    text-decoration: none;
}

a:hover {
    color: var(--accent-hover);
}

/* Utility */
.text-center { text-align: center; }
.mt-4 { margin-top: 16px; }
.mb-4 { margin-bottom: 16px; }
.flex { display: flex; }
.flex-between { justify-content: space-between; }
.gap-2 { gap: 8px; }
```

- [ ] **Step 2: Commit**

```bash
git add static/style.css
git commit -m "feat: add CSS styles with dark industrial theme"
```

---

## Task 7: HTML Templates

**Files:**
- Create: `templates/base.html`
- Create: `templates/index.html`
- Create: `templates/app_detail.html`
- Create: `templates/new_app.html`
- Modify: `src/main.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Create templates/base.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Version Server{% endblock %}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        {% block content %}{% endblock %}
    </div>
</body>
</html>
```

- [ ] **Step 2: Create templates/index.html**

```html
{% extends "base.html" %}

{% block title %}Version Server{% endblock %}

{% block content %}
<header class="header">
    <h1 class="header-title">VERSION SERVER</h1>
    <a href="/app/new" class="btn btn-primary">+ NEW APP</a>
</header>

<h2 style="color: var(--text-secondary); margin-bottom: 24px;">Apps</h2>

{% for app in apps %}
<div class="card">
    <div class="card-header">
        <div class="card-title">{{ app.name }}</div>
    </div>
    <div class="card-meta">
        <div class="card-meta-item">
            <span class="card-meta-label">ID:</span>
            <span class="card-meta-value">{{ app.id }}</span>
        </div>
        <div class="card-meta-item">
            <span class="card-meta-label">Created:</span>
            <span class="card-meta-value">{{ app.created_at }}</span>
        </div>
    </div>
    <div class="card-footer">
        <a href="/app/{{ app.id }}" class="btn btn-sm">DETAILS →</a>
    </div>
</div>
{% endfor %}

{% if apps.is_empty() %}
<div class="card text-center">
    <p style="color: var(--text-secondary);">No apps yet. Create one to get started.</p>
</div>
{% endif %}
{% endblock %}
```

- [ ] **Step 3: Create templates/app_detail.html**

```html
{% extends "base.html" %}

{% block title %}{{ app.name }}{% endblock %}

{% block content %}
<div style="margin-bottom: 24px;">
    <a href="/" class="btn btn-sm">← BACK TO HOME</a>
</div>

<header class="header">
    <h1 class="header-title">{{ app.name }}</h1>
</header>

<div style="margin-bottom: 32px;">
    <button class="btn btn-primary" onclick="showUploadModal()">⬆ UPLOAD NEW VERSION</button>
</div>

<h2 style="color: var(--text-secondary); margin-bottom: 24px;">Version History</h2>

{% for version in versions %}
<div class="version-item">
    <div class="version-header">
        <span class="version-number">v{{ version.version }}</span>
        <span class="version-date">{{ version.release_date }}</span>
    </div>
    {% for item in version.changelog %}
    <div class="changelog-item">
        <span class="changelog-icon">●</span>
        <span>{{ item.item_type }} {{ item.description }}</span>
    </div>
    {% endfor %}
    <div class="file-info">
        FILE: {{ version.file_name }} ({{ version.file_size | human_size }} bytes)
    </div>
    <div class="card-footer">
        <a href="{{ version.download_url }}" class="btn btn-sm">↓</a>
        <button class="btn btn-sm btn-danger" onclick="deleteVersion('{{ version.version }}')">✕ DELETE</button>
    </div>
</div>
{% endfor %}

{% if versions.is_empty() %}
<div class="card text-center">
    <p style="color: var(--text-secondary);">No versions yet. Upload one to get started.</p>
</div>
{% endif %}

<!-- Upload Modal -->
<div id="uploadModal" style="display: none; position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.8); z-index: 100;">
    <div style="max-width: 500px; margin: 100px auto; background: var(--bg-card); border: 1px solid var(--border); border-radius: 8px; padding: 24px;">
        <h3 style="margin-bottom: 24px;">Upload New Version</h3>
        <form id="uploadForm" action="/api/apps/{{ app.id }}/versions" method="post" enctype="multipart/form-data">
            <div class="form-group">
                <label class="form-label">VERSION</label>
                <input type="text" name="version" class="form-input" placeholder="0.1.0" required>
            </div>
            <div class="form-group">
                <label class="form-label">PACKAGE FILE</label>
                <input type="file" name="file" class="form-input" required>
            </div>
            <div class="form-group">
                <label class="form-label">CHANGELOG (JSON)</label>
                <textarea name="changelog" class="form-input" rows="4" placeholder='[{"type":"feature","description":"..."}]'></textarea>
            </div>
            <div class="flex flex-between" style="margin-top: 24px;">
                <button type="button" class="btn" onclick="hideUploadModal()">CANCEL</button>
                <button type="submit" class="btn btn-primary">UPLOAD →</button>
            </div>
        </form>
    </div>
</div>

<script>
function showUploadModal() {
    document.getElementById('uploadModal').style.display = 'block';
}
function hideUploadModal() {
    document.getElementById('uploadModal').style.display = 'none';
}
function deleteVersion(version) {
    if (confirm('Delete version ' + version + '?')) {
        fetch('/api/apps/{{ app.id }}/versions/' + version, { method: 'DELETE' })
            .then(r => { if (r.ok) location.reload(); });
    }
}
</script>
{% endblock %}
```

- [ ] **Step 4: Create templates/new_app.html**

```html
{% extends "base.html" %}

{% block title %}New App{% endblock %}

{% block content %}
<div style="margin-bottom: 24px;">
    <a href="/" class="btn btn-sm">← BACK TO HOME</a>
</div>

<header class="header">
    <h1 class="header-title">NEW APP</h1>
</header>

<div class="card">
    <form action="/api/apps" method="post">
        <div class="form-group">
            <label class="form-label">APP ID (unique identifier)</label>
            <input type="text" name="id" class="form-input" placeholder="ssh-tunnel-manager" required>
        </div>
        <div class="form-group">
            <label class="form-label">APP NAME (display name)</label>
            <input type="text" name="name" class="form-input" placeholder="SSH Tunnel Manager" required>
        </div>
        <div class="flex flex-between" style="margin-top: 24px;">
            <a href="/" class="btn">CANCEL</a>
            <button type="submit" class="btn btn-primary">CREATE →</button>
        </div>
    </form>
</div>
{% endblock %}
```

- [ ] **Step 5: Add Askama template structs and routes**

Add to `Cargo.toml`:
```toml
[dependencies]
askama = { version = "0.12", features = ["with-actix-web"] }
askama_actix = "0.14"
```

Add template handlers to `src/handlers.rs`:
```rust
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub apps: Vec<App>,
}

#[derive(Template)]
#[template(path = "app_detail.html")]
pub struct AppDetailTemplate {
    pub app: App,
    pub versions: Vec<Version>,
}

#[derive(Template)]
#[template(path = "new_app.html")]
pub struct NewAppTemplate;

pub async fn index_page(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.load_apps() {
        Ok(apps) => {
            let template = IndexTemplate { apps };
            HttpResponse::Ok().content_type("text/html").body(template.render().unwrap())
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn app_detail_page(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let app_id = path.into_inner();
    
    match (state.storage.get_app(&app_id), state.storage.load_versions(&app_id)) {
        (Ok(Some(app)), Ok(versions)) => {
            let template = AppDetailTemplate { app, versions };
            HttpResponse::Ok().content_type("text/html").body(template.render().unwrap())
        }
        _ => HttpResponse::NotFound().body("App not found"),
    }
}

pub async fn new_app_page() -> HttpResponse {
    let template = NewAppTemplate;
    HttpResponse::Ok().content_type("text/html").body(template.render().unwrap())
}
```

- [ ] **Step 6: Add page routes in main.rs**

```rust
// Page routes (HTML)
.route("/", web::get().to(handlers::index_page))
.route("/app/{app_id}", web::get().to(handlers::app_detail_page))
.route("/app/new", web::get().to(handlers::new_app_page))
```

- [ ] **Step 7: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully (may need Askama configuration adjustments)

- [ ] **Step 8: Commit**

```bash
git add templates/ Cargo.toml src/handlers.rs src/main.rs
git commit -m "feat: add HTML templates with Askama"
```

---

## Task 8: Test and Finalize

- [ ] **Step 1: Build release binary**

Run: `cargo build --release`
Expected: Creates `target/release/tool-update-server.exe`

- [ ] **Step 2: Run server and test**

Run: `./target/release/tool-update-server.exe`
Expected: Server starts at http://localhost:8080

- [ ] **Step 3: Test API endpoints manually**

- GET http://localhost:8080/api/apps → empty array []
- POST http://localhost:8080/api/apps with JSON body → creates app
- GET http://localhost:8080/ → shows HTML page

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: tool-update-server complete implementation"
```

---

## Summary

This plan creates a complete version management server in 8 tasks:

1. Project initialization
2. Data models
3. Storage layer
4. HTTP handlers
5. Main entry and routes
6. CSS styles
7. HTML templates
8. Test and finalize

Each task produces self-contained changes that compile and can be tested independently.