# 多项功能优化设计文档

## 1. 概述

本次优化涉及两个项目的多个改进：

**ssh-tunnel-manager 项目：**
1. Settings 页面按钮间距优化
2. 客户端下载地址拼接修复
3. 编译成果物改为 ZIP 格式（带版本号）

**tool-update-server 项目：**
1. 端口可配置（默认 39000，绑定 0.0.0.0）
2. 下载地址保持相对路径

---

## 2. ssh-tunnel-manager 项目修改

### 2.1 Settings.vue 按钮间距

**文件：** `src/views/Settings.vue`

**修改：** 在返回主页按钮添加 `style="margin-right: 8px"`，与保存按钮分隔。

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

### 2.2 客户端下载地址拼接

**文件：** `src-tauri/src/updater/mod.rs`

**问题：** 当前 `download_and_install_update` 直接使用服务端返回的相对路径 `/download/...`，没有拼接服务器地址。

**修改：** 在 `download_and_install_update` 函数中拼接完整 URL。

```rust
// 从 check_update 获取的 download_url 是相对路径
// 需要拼接服务器地址
let server_url = crate::db::get_app_setting("update_server_url")
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

let full_download_url = format!(
    "{}{}",
    server_url.trim_end_matches('/'),
    info.download_url
);
```

### 2.3 编译成果物改为 ZIP 格式

**文件：** `src-tauri/tauri.conf.json`

**修改：** bundle targets 改为 `["zip"]`

```json
"bundle": {
  "active": true,
  "targets": ["zip"],
  "icon": [...]
}
```

**输出格式：** `ssh-tunnel-manager_{version}_x64.zip`

**输出路径：** `src-tauri/target/release/bundle/`

---

## 3. tool-update-server 项目修改

### 3.1 端口可配置

**文件：** `src/main.rs`

**修改：** 
- 支持命令行参数 `-port 39000`
- 默认端口 39000
- 绑定地址改为 `0.0.0.0`

```rust
fn parse_port_arg(args: &[String]) -> Option<u16> {
    for i in 0..args.len() - 1 {
        if args[i] == "-port" {
            return args[i + 1].parse().ok();
        }
    }
    None
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port = parse_port_arg(&args).unwrap_or(39000);
    
    // ...
    
    HttpServer::new(move || { ... })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
```

**启动方式：**
- 默认：`tool-update-server`（端口 39000）
- 自定义：`tool-update-server -port 38000`

### 3.2 下载地址保持相对路径

**当前状态：** 服务端 `download_url` 已是相对路径 `/download/{app_id}/{filename}`

**无需修改：** 服务端保持现状，客户端负责拼接完整 URL。

---

## 4. 实现顺序

1. tool-update-server 端口配置（独立，无依赖）
2. ssh-tunnel-manager Settings.vue 按钮间距（独立）
3. ssh-tunnel-manager 下载地址拼接修复
4. ssh-tauri-manager ZIP 编译配置