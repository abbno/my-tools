# 托盘菜单快捷操作功能设计

## 功能概述

在系统托盘图标右键菜单中增加对常用隧道和所有分组隧道的快捷开启和关闭功能，用户无需打开主窗口即可快速操作隧道。

## 菜单结构

```
┌─────────────────────────┐
│ 🟢 常用隧道            │ ← 子菜单（仅当有常用隧道时显示）
│    ├─ 🟢 隧道A → 停止  │ ← 运行中显示绿色●，操作为"停止"
│    ├─ 🔴 隧道B → 启动  │ ← 已停止显示红色●，操作为"启动"
│ 🔴 分组A               │ ← 子菜单（分组名前显示该分组运行状态摘要）
│    ├─ 🟢 隧道1 → 停止  │
│    ├─ 🔴 隧道2 → 启动  │
│ 🟢 分组B               │ ← 全部运行中显示绿色
│    └─ ...              │
│ ─────────────────────  │ ← 分隔线
│ 打开主窗口             │
│ 退出                   │
└─────────────────────────┘
```

**排列顺序**：常用隧道优先，分组按 `sort_order` 排序在其后。

## 状态图标

- **运行中**：绿色圆点 `🟢`
- **已停止**：红色圆点 `🔴`

**注意**：Tauri 菜单对 emoji 支持可能有限，备选方案使用 Unicode 字符：
- 绿色：使用带颜色的文本或前缀 `●` 配合状态文字
- 红色：同上

实际实现时测试 emoji 支持，若不支持则改用文字标注 `[运行]` / `[停止]`。

## 菜单项 ID 格式

用于在 `on_menu_event` 中识别点击的菜单项：

| 类型 | ID 格式 | 示例 |
|------|---------|------|
| 常用隧道启动 | `fav:{config_id}:start` | `fav:abc123:start` |
| 常用隧道停止 | `fav:{config_id}:stop` | `fav:abc123:stop` |
| 分组隧道启动 | `grp:{group_id}:{config_id}:start` | `grp:group1:abc123:start` |
| 分组隧道停止 | `grp:{group_id}:{config_id}:stop` | `grp:group1:abc123:stop` |

## 数据流程

每次菜单打开时：
1. 查询数据库获取常用隧道列表（`db::get_favorites()`）
2. 查询数据库获取所有分组（`db::get_groups()`）
3. 查询每个分组下的隧道（`db::get_configs_by_group()`）
4. 获取当前运行中的隧道状态（`ssh::get_running_tunnels()`）
5. 构建菜单树，设置到托盘

## 技术实现

### 实现方案：纯 Rust 后端

在 `lib.rs` 中实现完整的菜单构建和事件处理逻辑。

### 核心改动

**文件：`src-tauri/src/lib.rs`**

新增：
- `build_tray_menu(app: &AppHandle) -> Result<Menu, Box<dyn Error>>` 函数
- 动态构建常用隧道和分组子菜单
- 菜单事件处理逻辑：解析 ID，调用隧道启动/停止命令

修改：
- `setup` 函数：托盘菜单构建改为动态构建
- `on_menu_event`：增加对隧道操作菜单项的处理

**文件：`src-tauri/src/ssh/mod.rs` 或相关文件**

新增：
- `get_all_tunnel_statuses() -> HashMap<String, TunnelStatus>` 返回所有隧道状态 Map（可选，用于优化状态查询）

### 关键代码逻辑

```rust
fn build_tray_menu(app: &AppHandle) -> Result<Menu, Box<dyn Error>> {
    // 1. 获取常用隧道
    let favorites = db::get_favorites()?;
    let running = ssh::get_running_tunnels();

    // 2. 构建"常用隧道"子菜单
    let fav_submenu = if !favorites.is_empty() {
        let items: Vec<MenuItem> = favorites.iter().map(|config| {
            let is_running = running.contains_key(&config.id);
            let status_icon = if is_running { "🟢" } else { "🔴" };
            let action = if is_running { "停止" } else { "启动" };
            let action_code = if is_running { "stop" } else { "start" };
            let text = format!("{} {} → {}", status_icon, config.name, action);
            let id = format!("fav:{}:{}", config.id, action_code);
            MenuItem::with_id(app, &id, &text, true, None::<&str>)
        }).collect();
        Submenu::with_items(app, "常用隧道", true, &items)
    } else {
        None
    };

    // 3. 构建分组子菜单（类似逻辑）
    // ...

    // 4. 组装主菜单
    let menu = Menu::new(app)?;
    if let Some(sub) = fav_submenu {
        menu.append(&sub)?;
    }
    // 分组子菜单...
    menu.append(&MenuItem::with_id(app, "show", "打开主窗口", true, None::<&str>))?;
    menu.append(&MenuItem::with_id(app, "quit", "退出", true, None::<&str>))?;
    Ok(menu)
}
```

### 菜单动态更新

Tauri 的 `TrayIcon` 支持 `set_menu()` 方法，可在每次菜单打开前动态更新。实现方式：

1. 使用 `on_tray_icon_event` 监听菜单打开事件
2. 在事件触发时调用 `build_tray_menu()` 重新构建菜单
3. 使用 `tray.set_menu(&new_menu)` 更新

**注意**：`on_tray_icon_event` 的 `Click` 事件在 Windows 上是左键点击，右键菜单打开需要监听其他事件或使用 Tauri 的默认菜单显示行为（已配置 `show_menu_on_left_click(true)`）。

实际测试发现：Windows 上右键菜单由系统触发，无法精确拦截"菜单打开前"事件。替代方案：
- 使用定时器定期更新菜单（如每 5 秒）
- 或在每次隧道状态变化后更新菜单（启动/停止事件触发）

## 成功标准

1. 托盘右键菜单显示常用隧道子菜单和各分组子菜单
2. 每个隧道显示正确状态图标和对应的启动/停止操作
3. 点击菜单项能正确启动或停止隧道
4. 隧道状态变化后菜单能及时更新

## 测试要点

1. 无常用隧道时菜单不显示"常用隧道"区域
2. 无分组时菜单只显示隧道（如有）和基础菜单项
3. 长隧道名称的显示效果（是否截断）
4. emoji 图标在不同 Windows 版本的显示兼容性
5. 多个隧道同时运行时的菜单响应性能