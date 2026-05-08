---
name: 软件退出时停止所有连接
description: 软件退出时自动停止所有运行中的 SSH 隧道连接
type: project
---

# 软件退出时停止所有连接

## 背景

当前软件退出时（通过托盘菜单"退出"），直接调用 `app.exit(0)`，不会清理正在运行的 SSH 隧道进程。这可能导致 SSH 进程残留，占用端口和资源。

## 目标

在软件退出前，自动停止所有运行中的 SSH 隧道连接，确保进程被正确清理。

## 设计

### 新增函数

在 `src-tauri/src/ssh/sidecar.rs` 中新增 `stop_all_tunnels()` 函数：

```rust
/// 停止所有运行中的 SSH 隧道
pub fn stop_all_tunnels() {
    let tunnels = TUNNELS.lock().unwrap();
    let running_ids: Vec<String> = tunnels
        .values()
        .filter(|info| info.status == TunnelStatus::Running)
        .map(|info| info.config_id.clone())
        .collect();

    for config_id in running_ids {
        // 停止监控任务
        stop_monitor(&config_id);

        // 终止进程
        let mut processes = SSH_PROCESSES.lock().unwrap();
        if let Some(mut child) = processes.remove(&config_id) {
            let _ = child.kill();
        }
    }

    // 清空隧道状态
    let mut tunnels = TUNNELS.lock().unwrap();
    tunnels.clear();

    // 清空进程记录
    let mut processes = SSH_PROCESSES.lock().unwrap();
    processes.clear();

    // 清空监控任务
    let mut handles = MONITOR_HANDLES.lock().unwrap();
    for handle in handles.values() {
        handle.abort();
    }
    handles.clear();
}
```

### 修改退出逻辑

在 `src-tauri/src/lib.rs` 的托盘菜单事件处理中，修改 "quit" 分支：

```rust
"quit" => {
    // 停止所有运行中的隧道
    crate::ssh::stop_all_tunnels();
    app.exit(0);
}
```

### 为什么: 防止进程残留
SSH 进程是外部子进程，软件退出后不会自动终止。如果不显式清理，会占用端口和系统资源。

### How to apply: 在所有退出入口点调用清理函数
目前只有托盘菜单 "quit" 是真正的退出入口点。未来如果有其他退出方式（如快捷键），也需要调用 `stop_all_tunnels()`。