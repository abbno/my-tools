# 托盘菜单快捷操作功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在系统托盘右键菜单中增加常用隧道和分组隧道的快捷启动/停止操作。

**Architecture:** 纯 Rust 后端实现，在 lib.rs 中动态构建托盘菜单，菜单打开时查询数据库和运行状态，构建分层子菜单结构。

**Tech Stack:** Tauri 2.x Menu/Submenu/MenuItem API，SQLite 数据库查询，SSH 隧道状态管理

---

## 文件结构

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/lib.rs` | 修改 | 新增托盘菜单构建函数和事件处理 |
| `src-tauri/src/ssh/mod.rs` | 修改 | 新增 `update_tray_menu` 公开函数 |
| `src-tauri/src/ssh/sidecar.rs` | 修改 | 在启动/停止隧道后调用菜单更新 |

---

### Task 1: 新增托盘菜单构建函数

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 lib.rs 顶部添加 Submenu import**

在现有 imports 中添加 `Submenu`：

```rust
use tauri::{
    menu::{Menu, MenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
```

- [ ] **Step 2: 在 setup 函数之前添加 build_tray_menu 函数**

```rust
/// 构建托盘菜单
fn build_tray_menu(app: &tauri::AppHandle) -> Result<Menu, Box<dyn std::error::Error>> {
    use crate::db;
    use crate::ssh::get_running_tunnels;

    // 获取运行中的隧道
    let running_tunnels = get_running_tunnels();
    let running_ids: std::collections::HashSet<String> = running_tunnels
        .iter()
        .map(|t| t.config_id.clone())
        .collect();

    // 获取常用隧道
    let favorites = db::get_favorites()?;
    let groups = db::get_groups()?;

    // 创建菜单
    let menu = Menu::new(app)?;

    // 常用隧道子菜单
    if !favorites.is_empty() {
        let fav_items: Vec<&MenuItem> = favorites
            .iter()
            .map(|config| {
                let is_running = running_ids.contains(&config.id);
                let status_icon = if is_running { "🟢" } else { "🔴" };
                let action = if is_running { "停止" } else { "启动" };
                let action_code = if is_running { "stop" } else { "start" };
                let text = format!("{} {} → {}", status_icon, config.name, action);
                let id = format!("fav:{}:{}", config.id, action_code);
                MenuItem::with_id(app, &id, &text, true, None::<&str>)
                    .expect("Failed to create menu item")
            })
            .collect();

        let fav_submenu = Submenu::with_items(app, "常用隧道", true, &fav_items)?;
        menu.append(&fav_submenu)?;
    }

    // 分组子菜单
    for group in &groups {
        let configs = db::get_configs_by_group(&group.id)?;
        if configs.is_empty() {
            continue;
        }

        // 检查分组整体状态
        let running_count = configs.iter().filter(|c| running_ids.contains(&c.id)).count();
        let group_icon = if running_count == configs.len() && running_count > 0 {
            "🟢"
        } else if running_count > 0 {
            "🟡"
        } else {
            "🔴"
        };

        let group_items: Vec<&MenuItem> = configs
            .iter()
            .map(|config| {
                let is_running = running_ids.contains(&config.id);
                let status_icon = if is_running { "🟢" } else { "🔴" };
                let action = if is_running { "停止" } else { "启动" };
                let action_code = if is_running { "stop" } else { "start" };
                let text = format!("{} {} → {}", status_icon, config.name, action);
                let id = format!("grp:{}:{}:{}", group.id, config.id, action_code);
                MenuItem::with_id(app, &id, &text, true, None::<&str>)
                    .expect("Failed to create menu item")
            })
            .collect();

        let group_submenu = Submenu::with_items(app, &format!("{} {}", group_icon, group.name), true, &group_items)?;
        menu.append(&group_submenu)?;
    }

    // 分隔线和基础菜单项
    let show_item = MenuItem::with_id(app, "show", "打开主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    menu.append(&show_item)?;
    menu.append(&quit_item)?;

    Ok(menu)
}
```

- [ ] **Step 3: 验证编译通过**

运行: `cd src-tauri && cargo check`
预期: 编译通过，无错误

---

### Task 2: 修改 setup 函数使用动态菜单

**Files:**
- Modify: `src-tauri/src/lib.rs:53-100` (setup 函数中的托盘部分)

- [ ] **Step 1: 替换 setup 中的静态菜单构建代码**

找到 setup 函数中创建托盘菜单的代码（约第 53-56 行），将：

```rust
// 创建托盘菜单项
let show_item = MenuItem::with_id(app, "show", "打开主窗口", true, None::<&str>)?;
let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
```

替换为：

```rust
// 创建动态托盘菜单
let menu = build_tray_menu(app)?;
```

- [ ] **Step 2: 保存托盘实例以便后续更新**

在创建托盘后，需要保存 `TrayIcon` 以便后续更新菜单。将：

```rust
let _tray = TrayIconBuilder::new()
```

改为：

```rust
let tray = TrayIconBuilder::new()
```

然后在 setup 函数末尾，将 tray 存储到 AppHandle：

```rust
// 存储托盘引用以便后续更新
app.manage(TrayState(Mutex::new(Some(tray))));
```

但 Tauri 2.x 不支持直接 manage TrayIcon，需要使用其他方式。改为使用全局静态变量：

在 `lib.rs` 顶部添加：

```rust
use std::sync::{Mutex, LazyLock};

/// 托盘实例
pub static TRAY: LazyLock<Mutex<Option<tauri::tray::TrayIcon>>> = LazyLock::new(|| Mutex::new(None));
```

然后在 setup 中存储：

```rust
// 存储托盘引用以便后续更新
*TRAY.lock().unwrap() = Some(tray);
```

- [ ] **Step 3: 验证编译通过**

运行: `cd src-tauri && cargo check`
预期: 编译通过

---

### Task 3: 实现菜单事件处理逻辑

**Files:**
- Modify: `src-tauri/src/lib.rs:on_menu_event`

- [ ] **Step 1: 修改 on_menu_event 处理隧道操作**

将现有的 `on_menu_event` 处理逻辑：

```rust
.on_menu_event(|app, event| match event.id.as_ref() {
    "show" => {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    "quit" => {
        app.exit(0);
    }
    _ => {}
})
```

改为：

```rust
.on_menu_event(|app, event| {
    let id = event.id.as_ref();
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {
            // 处理隧道操作
            if id.starts_with("fav:") || id.starts_with("grp:") {
                handle_tunnel_menu_event(app, id);
            }
        }
    }
})
```

- [ ] **Step 2: 添加 handle_tunnel_menu_event 函数**

在 `build_tray_menu` 函数之后添加：

```rust
/// 处理隧道菜单事件
fn handle_tunnel_menu_event(app: &tauri::AppHandle, id: &str) {
    use crate::ssh::{start_ssh_tunnel, stop_ssh_tunnel, stop_monitor_task, start_monitor_with_defaults};
    use crate::db;

    // 解析菜单项 ID: fav:{config_id}:start/stop 或 grp:{group_id}:{config_id}:start/stop
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() < 3 {
        return;
    }

    let config_id = if parts[0] == "fav" {
        parts[1].to_string()
    } else {
        parts[2].to_string()
    };

    let action = parts.last().unwrap_or("");

    match action {
        "start" => {
            // 获取配置
            let config = db::get_config_by_id(&config_id);
            if let Ok(Some(cfg)) = config {
                // 启动隧道
                if let Ok(_info) = start_ssh_tunnel(&cfg) {
                    // 启动监控
                    start_monitor_with_defaults(cfg.id.clone(), cfg.auto_reconnect, cfg.reconnect_interval);
                    // 更新菜单
                    update_tray_menu(app);
                }
            }
        }
        "stop" => {
            // 停止监控
            stop_monitor_task(&config_id);
            // 停止隧道
            if let Ok(_info) = stop_ssh_tunnel(&config_id) {
                // 更新菜单
                update_tray_menu(app);
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 3: 添加 update_tray_menu 函数**

在 `handle_tunnel_menu_event` 之后添加：

```rust
/// 更新托盘菜单
pub fn update_tray_menu(app: &tauri::AppHandle) {
    if let Ok(menu) = build_tray_menu(app) {
        let tray = TRAY.lock().unwrap();
        if let Some(tray_icon) = tray.as_ref() {
            let _ = tray_icon.set_menu(Some(&menu));
        }
    }
}
```

- [ ] **Step 4: 验证编译通过**

运行: `cd src-tauri && cargo check`
预期: 编译通过

---

### Task 4: 在隧道状态变化后更新菜单

**Files:**
- Modify: `src-tauri/src/ssh/sidecar.rs:52-57` (start_ssh_tunnel 函数)
- Modify: `src-tauri/src/ssh/sidecar.rs:120-125` (stop_ssh_tunnel 函数)

- [ ] **Step 1: 在 sidecar.rs 中导入 update_tray_menu**

在 `sidecar.rs` 顶部添加：

```rust
use crate::update_tray_menu;
```

但这需要 `lib.rs` 中的函数是 pub 的。由于模块依赖关系（sidecar 是 ssh 的子模块，ssh 是 lib 的子模块），需要反向暴露。

改为在 `ssh/mod.rs` 中添加代理函数：

修改 `src-tauri/src/ssh/mod.rs`：

```rust
mod sidecar;
mod monitor;

use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use tauri::AppHandle;

pub use monitor::*;
pub use sidecar::*;

use crate::models::TunnelInfo;

/// 运行中的隧道信息
pub static TUNNELS: LazyLock<Mutex<HashMap<String, TunnelInfo>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 应用句柄
pub static APP_HANDLE: LazyLock<Mutex<Option<AppHandle>>> = LazyLock::new(|| Mutex::new(None));

/// 初始化 SSH 管理器
pub fn init(app_handle: AppHandle) {
    *APP_HANDLE.lock().unwrap() = Some(app_handle);
}

/// 更新托盘菜单（代理到 lib.rs）
pub fn update_tray_menu() {
    if let Some(app) = APP_HANDLE.lock().unwrap().as_ref() {
        crate::update_tray_menu(app);
    }
}
```

- [ ] **Step 2: 在 start_ssh_tunnel 成功后调用 update_tray_menu**

修改 `src-tauri/src/ssh/sidecar.rs` 的 `start_ssh_tunnel` 函数，在返回成功结果前添加：

```rust
// 在 let info = TunnelInfo { ... 之后，return Ok(info) 之前添加
// 更新托盘菜单
super::update_tray_menu();
```

找到 `start_ssh_tunnel` 函数末尾（约第 52-57 行），在 `Ok(info)` 返回前添加调用：

```rust
    // 更新托盘菜单
    super::update_tray_menu();

    Ok(info)
```

- [ ] **Step 3: 在 stop_ssh_tunnel 成功后调用 update_tray_menu**

同样在 `stop_ssh_tunnel` 函数末尾（约第 120-125 行），在返回前添加：

```rust
    // 更新托盘菜单
    super::update_tray_menu();

    Ok(info)
```

- [ ] **Step 4: 验证编译通过**

运行: `cd src-tauri && cargo check`
预期: 编译通过

---

### Task 5: 测试功能

**Files:**
- 无文件改动，手动测试

- [ ] **Step 1: 启动应用**

运行: `pnpm tauri dev`
预期: 应用正常启动

- [ ] **Step 2: 测试托盘菜单显示**

操作:
1. 右键点击托盘图标
2. 检查菜单是否显示常用隧道和分组子菜单
3. 检查状态图标是否正确显示（🟢 运行中，🔴 已停止）

预期: 菜单结构正确，状态图标显示

- [ ] **Step 3: 测试隧道启动/停止**

操作:
1. 在托盘菜单中点击一个已停止隧道的"启动"项
2. 检查隧道是否启动成功
3. 再次打开托盘菜单，检查状态是否变为 🟢 且操作变为"停止"
4. 点击"停止"项，检查隧道是否停止

预期: 隧道操作正常，菜单状态实时更新

- [ ] **Step 4: 测试无常用隧道场景**

操作:
1. 清空所有常用隧道
2. 打开托盘菜单

预期: 不显示"常用隧道"子菜单，仅显示分组和基础菜单项

---

### Task 6: 提交代码

**Files:**
- 无文件改动，Git 操作

- [ ] **Step 1: 检查改动**

运行: `git status && git diff --stat`
预期: 确认改动文件列表

- [ ] **Step 2: 提交代码**

```bash
git add src-tauri/src/lib.rs src-tauri/src/ssh/mod.rs src-tauri/src/ssh/sidecar.rs
git commit -m "feat(tray): 托盘菜单新增常用隧道和分组快捷操作

- 新增 build_tray_menu 函数动态构建托盘菜单
- 常用隧道显示在分组之前
- 每个隧道显示状态图标和启动/停止操作
- 隧道状态变化后自动更新菜单

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

预期: 提交成功

---

## Self-Review Checklist

**1. Spec Coverage:**
- ✅ 常用隧道子菜单 → Task 1
- ✅ 分组子菜单 → Task 1
- ✅ 状态图标（🟢/🔴） → Task 1
- ✅ 动态单按钮 → Task 1
- ✅ 菜单打开时刷新 → Task 1 (build_tray_menu 每次构建时查询)
- ✅ 常用优先排列 → Task 1
- ✅ 隧道操作事件处理 → Task 3
- ✅ 状态变化后更新菜单 → Task 4

**2. Placeholder Scan:**
- ✅ 无 TBD/TODO
- ✅ 所有代码步骤包含完整代码
- ✅ 所有命令步骤包含具体命令

**3. Type Consistency:**
- ✅ 函数签名一致：`build_tray_menu(app: &tauri::AppHandle)`
- ✅ 菜单项 ID 格式一致：`fav:{id}:start/stop`, `grp:{group}:{id}:start/stop`
- ✅ TRAY 全局变量类型一致：`LazyLock<Mutex<Option<tauri::tray::TrayIcon>>>`