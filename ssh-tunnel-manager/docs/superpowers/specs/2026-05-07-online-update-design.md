# SSH Tunnel Manager 在线升级功能设计文档

## 1. 功能概述

**目标：** 实现应用内置自动升级功能，支持启动时自动检查、定时检查、手动检查三种触发方式。

**技术方案：** 使用 Tauri 内置 Updater 插件 + 自建版本服务器

**核心功能：**
- 启动时自动检查新版本
- 定时检查新版本（每4小时）
- 手动检查新版本（设置页面按钮）
- 弹窗提示用户升级，展示完整更新日志
- 自动下载并安装便携版 exe
- 下载失败时提供手动下载链接

---

## 2. 版本检查接口设计

### 自建服务器接口规范

**接口地址：** `GET https://your-server.com/api/version`

**返回格式：** JSON

```json
{
  "version": "0.2.0",
  "release_date": "2026-05-10",
  "download_url": "https://your-server.com/downloads/ssh-tunnel-manager_0.2.0_x64.exe",
  "signature": "dW50cnVzdGVkL29...",
  "changelog": [
    {
      "type": "feature",
      "description": "新增在线升级功能"
    },
    {
      "type": "fix",
      "description": "修复隧道连接状态显示异常"
    },
    {
      "type": "improve",
      "description": "优化托盘菜单响应速度"
    }
  ],
  "min_version": "0.1.0",
  "force_update": false
}
```

### 字段说明

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| version | string | 是 | 最新版本号 |
| release_date | string | 是 | 发布日期 |
| download_url | string | 是 | 便携版 exe 下载地址 |
| signature | string | 是 | 安装包签名（用于验证完整性） |
| changelog | array | 是 | 更新日志列表 |
| changelog[].type | string | 是 | 类型：feature/fix/improve |
| changelog[].description | string | 是 | 更新内容描述 |
| min_version | string | 否 | 最低可升级版本 |
| force_update | boolean | 否 | 是否强制更新 |

---

## 3. 打包配置修改

### tauri.conf.json 配置

```json
{
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": ["icons/icon.ico", "icons/32x32.png"],
    "windows": {
      "nsis": {
        "installMode": "portable"
      }
    }
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://your-server.com/api/version"],
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

### Cargo.toml 添加依赖

```toml
[dependencies]
tauri-plugin-updater = "2"
```

---

## 4. Rust 后端实现

### 模块结构

```
src-tauri/src/
├── updater/
│   ├── mod.rs          # 模块入口
│   ├── checker.rs      # 版本检查逻辑
│   ├── downloader.rs   # 下载管理
│   └── installer.rs    # 安装执行
```

### Tauri Commands 接口

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| check_update | - | Option<UpdateInfo> | 检查是否有新版本 |
| download_update | - | - | 下载更新包，通过事件发送进度 |
| install_update | - | - | 执行安装（应用将退出） |
| get_last_check_time | - | Option<String> | 获取上次检查时间 |

### 数据结构

```rust
pub struct UpdateInfo {
    version: String,
    release_date: String,
    changelog: Vec<ChangelogItem>,
    download_url: String,
    force_update: bool,
}

pub struct ChangelogItem {
    type: String,      // feature/fix/improve
    description: String,
}

pub struct DownloadProgress {
    downloaded: u64,
    total: u64,
    percentage: u8,
}
```

### 下载进度事件

```rust
// 发送下载进度事件
app.emit("update-download-progress", DownloadProgress {
    downloaded: 1024000,
    total: 5000000,
    percentage: 20,
});
```

---

## 5. 前端 UI 实现

### 升级弹窗组件

新增 `src/components/UpdateDialog.vue`

**弹窗布局：**

```
┌────────────────────────────────────────────┐
│  发现新版本                         [×]    │
├────────────────────────────────────────────┤
│                                            │
│  当前版本: 0.1.0                           │
│  最新版本: 0.2.0                           │
│  发布日期: 2026-05-10                      │
│                                            │
│  更新内容:                                 │
│  ┌──────────────────────────────────────┐ │
│  │ ✨ 新增在线升级功能                   │ │
│  │ 🐛 修复隧道连接状态显示异常           │ │
│  │ ⚡ 优化托盘菜单响应速度               │ │
│  └──────────────────────────────────────┘ │
│                                            │
│  下载进度: ████████░░░░░░░░ 45%            │
│                                            │
│         [稍后提醒]  [立即更新]             │
│                                            │
│  如下载失败，可手动下载:                   │
│  https://your-server.com/downloads/...     │
└────────────────────────────────────────────┘
```

### 设置页面入口

在设置页面添加"检查更新"区域：

```
┌────────────────────────────────────┐
│  设置                              │
├────────────────────────────────────┤
│  ...                               │
│                                    │
│  版本信息                          │
│  当前版本: 0.1.0                   │
│  上次检查: 2026-05-07 10:30        │
│                                    │
│  [检查更新]                        │
│                                    │
└────────────────────────────────────┘
```

### 状态管理

新增 `src/stores/update.ts`：

```typescript
interface UpdateState {
  checking: boolean;              // 是否正在检查
  downloading: boolean;           // 是否正在下载
  downloadProgress: number;       // 下载进度百分比
  updateInfo: UpdateInfo | null;  // 新版本信息
  lastCheckTime: string | null;   // 上次检查时间
}
```

### 定时检查逻辑

在 App.vue 中设置定时检查：

```typescript
onMounted(() => {
  // 启动时检查一次
  checkUpdate();

  // 设置定时检查（每4小时）
  setInterval(() => {
    checkUpdate();
  }, 4 * 60 * 60 * 1000);
});
```

---

## 6. 升级流程设计

### 启动时自动检查流程

```
应用启动
    │
    ▼
前端 App.vue mounted
    │
    ▼
调用 check_update()
    │
    ▼
后端检查版本接口
    │
    ├── 无新版本 → 记录检查时间，静默结束
    │
    ▼ 有新版本
返回 UpdateInfo 给前端
    │
    ▼
前端显示升级弹窗
    │
    ├── 用户点击"稍后提醒" → 关闭弹窗，记录状态
    └── 用户点击"立即更新" → 进入下载流程
```

### 下载与安装流程

```
用户点击"立即更新"
    │
    ▼
前端调用 download_update()
    │
    ▼
后端开始下载，发送进度事件
    │
    ├── 下载失败 → 显示手动下载链接
    │
    ▼ 下载成功
前端显示"下载完成"
    │
    ▼
前端调用 install_update()
    │
    ▼
后端启动便携版 exe 进程
    │
    ▼
当前应用退出
    │
    ▼
新版本便携版 exe 启动运行
```

### 定时检查流程

```
定时器触发（每4小时）
    │
    ▼
检查上次检查时间
    │
    ├── 间隔不足4小时 → 跳过本次检查
    │
    ▼ 间隔超过4小时
调用 check_update()
    │
    ▼
同启动检查流程
```

### 手动检查流程

```
用户点击"检查更新"
    │
    ▼
显示检查中状态
    │
    ▼
调用 check_update()
    │
    ├── 无新版本 → 提示"当前已是最新版本"
    │
    ▼ 有新版本
显示升级弹窗
```

---

## 7. 签名密钥管理

### 密钥生成

使用 Tauri CLI 生成签名密钥对：

```bash
npm run tauri signer generate
```

生成的文件：
- `tauri-signer.key` - 私钥（用于签名安装包，需保密）
- `tauri-signer.key.pub` - 公钥（用于配置 tauri.conf.json）

### 签名流程

1. 构建时自动签名安装包（配置环境变量）
2. 签名值写入版本接口的 `signature` 字段
3. 客户端下载后验证签名，确保安装包完整性

### 环境变量配置

```bash
# CI/CD 构建时设置
TAURI_SIGNING_PRIVATE_KEY=<私钥内容>
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=<私钥密码>
```

---

## 8. 错误处理

| 场景 | 处理方式 |
|------|----------|
| 版本接口请求失败 | 记录日志，下次检查时重试 |
| 下载失败 | 显示手动下载链接，用户可点击跳转 |
| 签名验证失败 | 拒绝安装，显示手动下载链接 |
| 安装进程启动失败 | 显示错误提示，保留手动下载链接 |
| 网络超时 | 重试3次后提示手动下载 |

---

## 9. 项目目录结构变更

```
src-tauri/src/
├── updater/           # 新增：升级模块
│   ├── mod.rs
│   ├── checker.rs
│   ├── downloader.rs
│   └── installer.rs

src/
├── components/
│   ├── UpdateDialog.vue  # 新增：升级弹窗组件
├── stores/
│   ├── update.ts         # 新增：升级状态管理
├── views/
│   ├── Settings.vue      # 修改：添加检查更新入口
```

---

## 10. 实现优先级

1. **第一阶段：基础框架**
   - 配置打包为便携版
   - 添加 tauri-plugin-updater 依赖
   - 实现版本检查接口

2. **第二阶段：核心功能**
   - 实现启动时自动检查
   - 实现升级弹窗 UI
   - 实现下载和安装流程

3. **第三阶段：完善功能**
   - 实现定时检查
   - 实现手动检查入口
   - 完善错误处理和手动下载链接