# 多项功能优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现两个项目的多项功能优化：按钮间距、下载地址拼接、ZIP编译、端口配置。

**Architecture:** 
- ssh-tunnel-manager: Vue前端按钮间距、Rust后端下载地址拼接、Tauri编译配置
- tool-update-server: Rust命令行参数解析、HTTP服务绑定配置

**Tech Stack:** Vue 3 + TDesign, Rust + Tauri 2.x, Rust + actix-web

---

## File Structure

**ssh-tunnel-manager 项目修改：**
- `src/views/Settings.vue` - 添加按钮间距
- `src-tauri/src/updater/mod.rs` - 下载地址拼接完整URL
- `src-tauri/tauri.conf.json` - 编译目标改为ZIP

**tool-update-server 项目修改：**
- `src/main.rs` - 端口参数解析和绑定地址

---

### Task 1: Settings.vue 按钮间距

**Files:**
- Modify: `D:\Projects\OtherProjects\ssh-tunnel-manager\src\views\Settings.vue:4-19`

- [ ] **Step 1: 在返回主页按钮添加 margin-right**

修改 Settings.vue 的 template #actions 部分，在返回主页按钮添加 style：

```vue
<template #actions>
  <t-button
    variant="outline"
    style="margin-right: 8px"
    @click="handleBack"
  >
    返回主页
  </t-button>
  <t-button
    theme="primary"
    :disabled="!hasChanges"
    :loading="saving"
    @click="handleSave"
  >
    保存
  </t-button>
</template>
```

- [ ] **Step 2: 验证前端编译**

Run: `cd D:\Projects\OtherProjects\ssh-tunnel-manager && npm run build`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/views/Settings.vue
git commit -m "fix: add spacing between back and save buttons in Settings"
```

---

### Task 2: 客户端下载地址拼接完整URL

**Files:**
- Modify: `D:\Projects\OtherProjects\ssh-tunnel-manager\src-tauri\src\updater\mod.rs`

- [ ] **Step 1: 修改 download_and_install_update 函数**

在 `download_and_install_update` 函数中，获取服务器地址并拼接完整下载URL。

找到使用 `info.download_url` 的位置（约第153行），修改为：

```rust
/// 下载并安装更新
/// 注意：调用此方法后应用会自动退出并重启
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    // 首先获取服务器地址
    let server_url = crate::db::get_app_setting("update_server_url")
        .map_err(|e| format!("读取更新服务器配置失败: {}", e))?
        .unwrap_or_default();

    // 检查更新以获取下载 URL
    let update_info = check_update(app.clone()).await?;

    match update_info {
        None => Err("没有可用的更新".to_string()),
        Some(info) => {
            let app_clone = app.clone();
            
            // 拼接完整下载 URL：服务器地址 + 相对路径
            let full_download_url = format!(
                "{}{}",
                server_url.trim_end_matches('/'),
                info.download_url
            );

            // 创建 HTTP 客户端
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(300)) // 5分钟超时
                .build()
                .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

            // 发送下载请求
            let response = client
                .get(&full_download_url)
                .send()
                .await
                .map_err(|e| format!("下载请求失败: {}", e))?;

            // ... 后续代码保持不变
```

- [ ] **Step 2: 验证 Rust 编译**

Run: `cd D:\Projects\OtherProjects\ssh-tunnel-manager\src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/updater/mod.rs
git commit -m "fix: concatenate server URL with download path for updates"
```

---

### Task 3: 编译成果物改为 ZIP 格式

**Files:**
- Modify: `D:\Projects\OtherProjects\ssh-tunnel-manager\src-tauri\tauri.conf.json`

- [ ] **Step 1: 修改 bundle targets 为 zip**

修改 tauri.conf.json 的 bundle 部分：

```json
"bundle": {
  "active": true,
  "targets": ["zip"],
  "icon": [
    "icons/icon.ico",
    "icons/32x32.png"
  ],
  "windows": {
    "nsis": {
      "installMode": "currentUser"
    }
  }
}
```

注意：保留 `windows.nsis` 配置，即使 targets 改为 zip，Tauri 可能仍需要这些配置。

- [ ] **Step 2: 验证配置文件**

Run: `cd D:\Projects\OtherProjects\ssh-tunnel-manager\src-tauri && cargo check`
Expected: 配置被正确解析

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: change build target to ZIP format with version number"
```

---

### Task 4: tool-update-server 端口配置

**Files:**
- Modify: `D:\Projects\OtherProjects\tool-update-server\src\main.rs`

- [ ] **Step 1: 添加端口参数解析函数**

在 main.rs 顶部添加端口解析函数：

```rust
mod models;
mod storage;
mod handlers;

use actix_web::{App, HttpServer, web};
use actix_files::Files;
use crate::handlers::AppState;

/// 解析命令行端口参数
/// 支持 -port 39000 格式
fn parse_port_arg(args: &[String]) -> Option<u16> {
    for i in 0..args.len() - 1 {
        if args[i] == "-port" {
            return args[i + 1].parse().ok();
        }
    }
    None
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let port = parse_port_arg(&args).unwrap_or(39000);

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
    println!("Server running at http://0.0.0.0:{}", port);

    // Create app state
    let app_state = web::Data::new(AppState { storage });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Static files
            .service(Files::new("/static", exe_dir.join("static")).show_files_listing())
            // Page routes (HTML)
            .route("/", web::get().to(handlers::index_page))
            .route("/app/new", web::get().to(handlers::new_app_page))
            .route("/app/{app_id}", web::get().to(handlers::app_detail_page))
            // Form routes
            .route("/api/apps/form", web::post().to(handlers::create_app_form))
            .route("/api/apps/{app_id}/versions/form", web::post().to(handlers::upload_version_form))
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
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
```

- [ ] **Step 2: 验证 Rust 编译**

Run: `cd D:\Projects\OtherProjects\tool-update-server && cargo check`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: add configurable port argument (-port) and bind to 0.0.0.0"
```

---

### Task 5: 集成测试

- [ ] **Step 1: 测试 ssh-tunnel-manager Settings 页面**

Run: `cd D:\Projects\OtherProjects\ssh-tunnel-manager && npm run tauri dev`

手动测试：
1. 打开设置页面
2. 验证返回主页按钮和保存按钮有间距

- [ ] **Step 2: 测试 tool-update-server 端口配置**

Run: `cd D:\Projects\OtherProjects\tool-update-server && cargo run -- -port 38000`

验证：
1. 服务启动在端口 38000
2. 绑定到 0.0.0.0（可从其他机器访问）

- [ ] **Step 3: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete multiple improvements implementation"
```

---

## Self-Review Checklist

1. **Spec coverage:**
   - ✓ Settings.vue 按钮间距 (Task 1)
   - ✓ 下载地址拼接完整URL (Task 2)
   - ✓ ZIP编译格式 (Task 3)
   - ✓ 端口配置 -port 参数 (Task 4)
   - ✓ 绑定到 0.0.0.0 (Task 4)

2. **Placeholder scan:** 无 TBD、TODO 或模糊描述。

3. **Type consistency:** 所有代码块使用正确的类型和函数名。