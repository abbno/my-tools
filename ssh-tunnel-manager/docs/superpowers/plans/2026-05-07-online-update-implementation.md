# SSH Tunnel Manager 在线升级功能实现计划

## 概述

基于设计文档 `docs/superpowers/specs/2026-05-07-online-update-design.md`，实现在线升级功能。

**实现方式：** 使用 Tauri 内置 Updater 插件 + 自建版本服务器

---

## 任务清单

### 第一阶段：基础配置

#### 1. 修改打包配置为便携版
- 文件：`src-tauri/tauri.conf.json`
- 修改 `bundle.targets` 为 `["nsis"]`
- 添加 `bundle.windows.nsis.installMode: "portable"`
- 添加 `plugins.updater` 配置（endpoints 和 pubkey）

#### 2. 添加 Rust 依赖
- 文件：`src-tauri/Cargo.toml`
- 添加 `tauri-plugin-updater = "2"`

#### 3. 生成签名密钥对
- 运行 `npm run tauri signer generate` 生成密钥
- 将公钥配置到 `tauri.conf.json`
- 私钥保存到安全位置（用于 CI/CD 构建）

---

### 第二阶段：Rust 后端实现

#### 4. 创建 updater 模块结构
- 新建 `src-tauri/src/updater/mod.rs`
- 新建 `src-tauri/src/updater/models.rs`（数据结构定义）

#### 5. 实现 Tauri Commands
- 文件：`src-tauri/src/updater/mod.rs`
- 实现 `check_update` 命令
- 实现 `download_update` 命令（带进度事件）
- 实现 `install_update` 命令
- 实现 `get_last_check_time` 命令

#### 6. 注册 Commands 到 lib.rs
- 文件：`src-tauri/src/lib.rs`
- 添加 `mod updater`
- 在 `invoke_handler` 中注册 updater commands
- 在 `setup` 中初始化 updater plugin

---

### 第三阶段：前端类型和 API

#### 7. 添加前端类型定义
- 文件：`src/types/index.ts`
- 添加 `UpdateInfo`、`ChangelogItem`、`DownloadProgress` 类型

#### 8. 添加前端 API 封装
- 文件：`src/api/tauri.ts`
- 添加 `checkUpdate`、`downloadUpdate`、`installUpdate`、`getLastCheckTime` 函数
- 添加 DTO 转换函数

---

### 第四阶段：前端状态管理

#### 9. 创建 update store
- 新建 `src/stores/update.ts`
- 实现 `useUpdateStore`
- 状态：checking、downloading、downloadProgress、updateInfo、lastCheckTime
- Actions：checkUpdate、downloadUpdate、installUpdate

#### 10. 更新 stores 入口
- 文件：`src/stores/index.ts`
- 导出 `useUpdateStore`

---

### 第五阶段：前端 UI 组件

#### 11. 创建升级弹窗组件
- 新建 `src/components/UpdateDialog.vue`
- 实现弹窗布局（版本信息、更新日志、下载进度、按钮）
- 使用 TDesign Dialog 组件
- 监听下载进度事件

#### 12. 创建设置页面（或添加到现有页面）
- 如果没有设置页面，新建 `src/views/Settings.vue`
- 添加版本信息区域
- 添加"检查更新"按钮
- 添加上次检查时间显示

#### 13. 添加路由（如需要）
- 文件：`src/router/index.ts`
- 添加 Settings 页面路由

---

### 第六阶段：集成和定时检查

#### 14. 在 App.vue 中集成
- 文件：`src/App.vue`
- 在 `onMounted` 中启动时检查更新
- 设置定时检查（每4小时）
- 监听下载进度事件
- 条件显示 UpdateDialog

---

### 第七阶段：测试和完善

#### 15. 本地测试
- 构建便携版安装包
- 测试版本检查流程
- 测试下载和安装流程
- 测试定时检查
- 测试手动检查

#### 16. 完善错误处理
- 添加网络超时重试
- 添加手动下载链接显示
- 添加下载失败提示

---

## 文件变更清单

### 新增文件
```
src-tauri/src/updater/mod.rs       # Updater 模块入口和 Commands
src-tauri/src/updater/models.rs    # 数据结构定义（可选，可合并到 mod.rs）
src/stores/update.ts               # 前端状态管理
src/components/UpdateDialog.vue    # 升级弹窗组件
src/views/Settings.vue             # 设置页面（如不存在）
```

### 修改文件
```
src-tauri/Cargo.toml               # 添加 updater 依赖
src-tauri/tauri.conf.json          # 打包配置和 updater 配置
src-tauri/src/lib.rs               # 注册 updater 模块和 commands
src-tauri/src/commands/mod.rs      # 导出 updater 模块（可选）
src/types/index.ts                 # 添加 updater 类型
src/api/tauri.ts                   # 添加 updater API
src/stores/index.ts                # 导出 update store
src/App.vue                        # 集成升级检查逻辑
src/router/index.ts                # 添加设置页面路由（如需要）
```

---

## 实现顺序建议

1. **先完成基础配置**（任务 1-3）- 确保打包配置正确
2. **实现 Rust 后端**（任务 4-6）- 核心功能
3. **实现前端类型和 API**（任务 7-8）- 类型安全的基础
4. **实现状态管理**（任务 9-10）- 数据流基础
5. **实现 UI 组件**（任务 11-13）- 用户界面
6. **集成到 App**（任务 14）- 整合所有功能
7. **测试和完善**（任务 15-16）- 确保功能正常

---

## 关键实现细节

### Rust Command 实现示例

```rust
// src-tauri/src/updater/mod.rs
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateInfo {
    version: String,
    release_date: String,
    changelog: Vec<ChangelogItem>,
    download_url: String,
    force_update: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChangelogItem {
    #[serde(rename = "type")]
    item_type: String,
    description: String,
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    
    // 检查更新
    let update = updater.check().await.map_err(|e| e.to_string())?;
    
    if let Some(update) = update {
        // 返回更新信息
        Ok(Some(UpdateInfo {
            version: update.version,
            release_date: update.date.unwrap_or_default(),
            changelog: vec![], // 需要从接口获取
            download_url: "".to_string(), // updater 内部处理
            force_update: false,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn download_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    
    // 下载并发送进度事件
    // ... 实现下载逻辑
    
    Ok(())
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    // 执行安装
    // ... 实现安装逻辑
    
    Ok(())
}

#[tauri::command]
pub fn get_last_check_time() -> Option<String> {
    // 从持久化存储获取上次检查时间
    None
}
```

### 前端 Store 实现示例

```typescript
// src/stores/update.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '@/api/tauri'
import { listen } from '@tauri-apps/api/event'
import type { UpdateInfo, DownloadProgress } from '@/types'

export const useUpdateStore = defineStore('update', () => {
  const checking = ref(false)
  const downloading = ref(false)
  const downloadProgress = ref(0)
  const updateInfo = ref<UpdateInfo | null>(null)
  const lastCheckTime = ref<string | null>(null)

  // 监听下载进度事件
  listen<DownloadProgress>('update-download-progress', (event) => {
    downloadProgress.value = event.payload.percentage
  })

  async function checkUpdate() {
    checking.value = true
    try {
      const info = await api.checkUpdate()
      updateInfo.value = info
      lastCheckTime.value = new Date().toISOString()
    } finally {
      checking.value = false
    }
  }

  async function downloadUpdate() {
    downloading.value = true
    downloadProgress.value = 0
    try {
      await api.downloadUpdate()
    } finally {
      downloading.value = false
    }
  }

  async function installUpdate() {
    await api.installUpdate()
  }

  return {
    checking,
    downloading,
    downloadProgress,
    updateInfo,
    lastCheckTime,
    checkUpdate,
    downloadUpdate,
    installUpdate,
  }
})
```

---

## 注意事项

1. **签名密钥安全**：私钥不要提交到代码仓库，使用环境变量在 CI/CD 中配置
2. **版本服务器接口**：需要单独部署，返回格式需符合 Tauri Updater 要求
3. **便携版特性**：便携版升级时直接替换 exe 文件，无需安装程序
4. **Windows 进程替换**：需要先启动新版本进程，再退出当前进程
5. **定时检查间隔**：建议 4 小时，避免频繁请求服务器