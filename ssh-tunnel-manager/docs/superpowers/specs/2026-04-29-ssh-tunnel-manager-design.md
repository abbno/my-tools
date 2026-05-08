# SSH Tunnel Manager 设计文档

## 1. 项目概述

**目标：** 开发一个跨平台桌面应用，可视化管理 SSH 端口转发隧道

**技术栈：**
- 前端：Vue 3 + TypeScript + TDesign 组件库
- 后端：Tauri 2.x + Rust
- 数据存储：SQLite（保存在程序运行目录，便于迁移）
- SSH 管理：混合架构（Sidecar + Rust 辅助库）

**核心功能：**
- SSH 隧道配置的增删改查
- 支持本地转发(-L)、远程转发(-R)、动态转发(-D)
- 支持密码和密钥两种认证方式
- 隧道启动/停止/重启
- 自动重连机制
- 连接状态监控和日志
- 配置分组和搜索
- 配置导入/导出
- 系统托盘图标

---

## 2. 数据模型设计

### 配置表 (configs)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT | 主键，UUID |
| name | TEXT | 配置名称 |
| group_id | TEXT | 所属分组 ID（可为空） |
| host | TEXT | SSH 服务器地址 |
| port | INTEGER | SSH 服务器端口 |
| username | TEXT | 登录用户名 |
| auth_type | TEXT | 认证类型：password / key |
| password | TEXT | 密码（加密存储） |
| key_path | TEXT | 私钥文件路径 |
| key_passphrase | TEXT | 私钥密码（加密存储） |
| tunnel_type | TEXT | 隧道类型：local / remote / dynamic |
| local_host | TEXT | 本地绑定地址 |
| local_port | INTEGER | 本地端口 |
| remote_host | TEXT | 远程目标地址 |
| remote_port | INTEGER | 远程目标端口 |
| auto_reconnect | BOOLEAN | 是否自动重连 |
| reconnect_interval | INTEGER | 重连间隔（秒） |
| created_at | DATETIME | 创建时间 |
| updated_at | DATETIME | 更新时间 |

### 分组表 (groups)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT | 主键，UUID |
| name | TEXT | 分组名称 |
| sort_order | INTEGER | 排序序号 |
| created_at | DATETIME | 创建时间 |

### 连接日志表 (connection_logs)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT | 主键，UUID |
| config_id | TEXT | 关联配置 ID |
| action | TEXT | 操作：connect / disconnect / reconnect / error |
| message | TEXT | 日志详情 |
| created_at | DATETIME | 记录时间 |

---

## 3. 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    Vue 3 前端                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  配置管理   │  │  隧道控制   │  │  日志查看   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  分组管理   │  │  搜索过滤   │  │  导入导出   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
                          │
                    Tauri IPC
                          │
┌─────────────────────────────────────────────────────────┐
│                   Rust 后端                             │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Command 处理层                       │   │
│  │  get_configs / save_config / delete_config ...  │   │
│  └─────────────────────────────────────────────────┘   │
│                          │                             │
│  ┌───────────────────┐  │  ┌───────────────────┐     │
│  │   SQLite 存储层    │  │  │   SSH 管理层      │     │
│  │   (rusqlite)       │  │  │                   │     │
│  └───────────────────┘  │  │  ┌─────────────┐  │     │
│                          │  │  │  Sidecar    │  │     │
│                          │  │  │  (系统SSH)  │  │     │
│                          │  │  └─────────────┘  │     │
│                          │  │  ┌─────────────┐  │     │
│                          │  │  │  状态监控   │  │     │
│                          │  │  │  (russh辅助)│  │     │
│                          │  │  └─────────────┘  │     │
│                          │  └───────────────────┘     │
└─────────────────────────────────────────────────────────┘
                          │
                    系统托盘
```

### 模块职责

| 模块 | 职责 |
|------|------|
| 前端 UI | 配置表单、隧道列表、状态展示、用户交互 |
| Command 层 | 接收前端调用，协调存储和 SSH 模块 |
| SQLite 层 | 配置持久化、日志存储、数据加密 |
| Sidecar 层 | 启动 SSH 进程、传递参数、管理生命周期 |
| 状态监控层 | 检测连接状态、触发自动重连、记录日志 |

---

## 4. 前端页面设计

### 主界面布局

```
┌──────────────────────────────────────────────────────────────┐
│  SSH Tunnel Manager                    [─] [□] [×]          │
├────────────┬─────────────────────────────────────────────────┤
│            │  工具栏                                        │
│   分组列表  │  [新建配置] [导入] [导出] [搜索框...]          │
│            ├─────────────────────────────────────────────────┤
│  □ 默认分组 │                                                 │
│  □ 生产环境 │   隧道列表                                     │
│  □ 测试环境 │   ┌─────────────────────────────────────────┐ │
│  □ 开发环境 │   │ ● 运行中  本地数据库隧道                  │ │
│            │   │   ssh -L 3306:localhost:3306 root@prod    │ │
│  [+ 新分组] │   │   [停止] [编辑] [日志]                    │ │
│            │   ├─────────────────────────────────────────┤ │
│            │   │ ○ 已停止  Redis调试隧道                   │ │
│            │   │   ssh -R 6379:localhost:6379 root@test   │ │
│            │   │   [启动] [编辑] [删除]                    │ │
│            │   └─────────────────────────────────────────┘ │
├────────────┴─────────────────────────────────────────────────┤
│  状态栏: 3 个隧道运行中 | 最后更新: 10:30:45                │
└──────────────────────────────────────────────────────────────┘
```

### 核心页面

| 页面 | 功能 |
|------|------|
| 主页面 | 分组侧边栏 + 隧道列表 + 状态栏 |
| 配置表单 | 新建/编辑 SSH 隧道配置（弹窗或侧边抽屉） |
| 日志查看 | 连接历史日志（弹窗或独立页面） |
| 系统托盘 | 快速启停常用隧道、查看状态 |

### 配置表单字段

- 基本信息：配置名称、所属分组
- 连接信息：主机地址、端口、用户名
- 认证方式：密码 / 密钥文件（带密码）
- 隧道配置：类型（本地/远程/动态）、本地地址、本地端口、远程地址、远程端口
- 高级选项：自动重连、重连间隔

---

## 5. 核心流程设计

### 启动隧道流程

```
用户点击"启动"
    │
    ▼
前端调用 start_tunnel(config_id)
    │
    ▼
后端检查配置有效性
    │
    ├── 无效 → 返回错误提示
    │
    ▼ 有效
构建 SSH 命令参数
    │
    ├── 本地转发: ssh -L local_port:remote_host:remote_port
    ├── 远程转发: ssh -R remote_port:local_host:local_port
    └── 动态转发: ssh -D local_port
    │
    ▼
启动 Sidecar 进程
    │
    ├── 进程启动成功 → 记录 PID，状态设为"运行中"
    └── 进程启动失败 → 返回错误，记录日志
    │
    ▼
启动状态监控任务
    │
    ▼
返回成功状态给前端
```

### 自动重连流程

```
状态监控检测到进程退出
    │
    ▼
检查配置的 auto_reconnect 字段
    │
    ├── false → 更新状态为"已停止"，记录日志
    │
    ▼ true
等待 reconnect_interval 秒
    │
    ▼
重新启动 SSH 进程
    │
    ├── 成功 → 记录重连日志，状态恢复"运行中"
    └── 失败 → 继续等待下一轮重连
```

### 停止隧道流程

```
用户点击"停止"
    │
    ▼
前端调用 stop_tunnel(config_id)
    │
    ▼
后端查找对应 PID
    │
    ▼
发送终止信号给进程
    │
    ▼
清理进程资源，更新状态为"已停止"
    │
    ▼
记录日志，返回前端
```

---

## 6. Tauri Command 接口设计

### 配置管理接口

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| get_configs | group_id?: string | Config[] | 获取配置列表，可按分组筛选 |
| get_config | id: string | Config | 获取单个配置详情 |
| save_config | config: Config | Config | 新建或更新配置 |
| delete_config | id: string | - | 删除配置 |
| search_configs | keyword: string | Config[] | 按名称/主机搜索配置 |

### 分组管理接口

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| get_groups | - | Group[] | 获取所有分组 |
| save_group | group: Group | Group | 新建或更新分组 |
| delete_group | id: string | - | 删除分组 |

### 隧道控制接口

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| start_tunnel | config_id: string | - | 启动隧道 |
| stop_tunnel | config_id: string | - | 停止隧道 |
| restart_tunnel | config_id: string | - | 重启隧道 |
| get_tunnel_status | config_id: string | TunnelStatus | 获取隧道状态 |
| get_running_tunnels | - | TunnelStatus[] | 获取所有运行中的隧道 |

### 日志接口

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| get_logs | config_id: string, limit?: number | Log[] | 获取连接日志 |
| clear_logs | config_id: string | - | 清除日志 |

### 导入导出接口

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| export_configs | ids?: string[] | string | 导出配置为 JSON |
| import_configs | json: string | number | 导入配置，返回导入数量 |

---

## 7. 项目目录结构

```
ssh-proxy/
├── src/                          # Vue 前端源码
│   ├── views/
│   │   ├── Home.vue              # 主页面
│   │   └── LogViewer.vue         # 日志查看页面
│   ├── components/
│   │   ├── Sidebar.vue           # 分组侧边栏
│   │   ├── TunnelList.vue        # 隧道列表
│   │   ├── TunnelCard.vue        # 隧道卡片
│   │   ├── ConfigForm.vue        # 配置表单弹窗
│   │   ├── GroupForm.vue         # 分组表单弹窗
│   │   └── LogPanel.vue          # 日志面板
│   ├── stores/
│   │   ├── config.ts             # 配置状态管理
│   │   ├── tunnel.ts             # 隧道状态管理
│   │   └── group.ts              # 分组状态管理
│   ├── api/
│   │   └── tauri.ts              # Tauri Command 封装
│   ├── types/
│   │   └── index.ts              # TypeScript 类型定义
│   ├── utils/
│   │   └── format.ts             # 工具函数
│   ├── App.vue
│   └── main.ts
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── main.rs               # 入口文件
│   │   ├── lib.rs                # 库入口
│   │   ├── commands/             # Command 处理
│   │   │   ├── mod.rs
│   │   │   ├── config.rs         # 配置相关命令
│   │   │   ├── group.rs          # 分组相关命令
│   │   │   ├── tunnel.rs         # 隧道控制命令
│   │   │   └── log.rs            # 日志相关命令
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── config.rs
│   │   │   ├── group.rs
│   │   │   └── log.rs
│   │   ├── db/                   # 数据库操作
│   │   │   ├── mod.rs
│   │   │   └── sqlite.rs
│   │   ├── ssh/                  # SSH 管理
│   │   │   ├── mod.rs
│   │   │   ├── sidecar.rs        # 进程管理
│   │   │   └── monitor.rs        # 状态监控
│   │   └── utils/                # 工具函数
│   │       ├── mod.rs
│   │       └── crypto.rs         # 加密工具
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## 8. 技术依赖清单

### 前端依赖

| 依赖 | 用途 |
|------|------|
| vue | Vue 3 框架 |
| pinia | 状态管理 |
| tdesign-vue-next | TDesign 组件库 |
| @tauri-apps/api | Tauri 前端 API |
| vue-router | 路由管理 |
| typescript | TypeScript 支持 |

### Rust 依赖

| 依赖 | 用途 |
|------|------|
| tauri | Tauri 框架 |
| tauri-plugin-shell | Sidecar 进程管理 |
| rusqlite | SQLite 数据库 |
| uuid | UUID 生成 |
| serde | 序列化/反序列化 |
| serde_json | JSON 处理 |
| chrono | 时间处理 |
| aes-gcm | 敏感数据加密 |

---

## 9. 错误处理与安全设计

### 错误处理策略

| 场景 | 处理方式 |
|------|----------|
| SSH 进程启动失败 | 返回错误信息，记录日志，前端显示错误提示 |
| 网络连接中断 | 触发自动重连机制（如已启用） |
| 数据库操作失败 | 返回错误信息，前端提示用户检查数据文件 |
| 配置文件损坏 | 提示用户恢复或重建配置 |
| 端口被占用 | 启动前检测端口，提示用户更换端口 |

### 安全设计

| 项目 | 措施 |
|------|------|
| 密码存储 | 使用 AES-GCM 加密，密钥由用户主密码派生或存储在系统密钥链 |
| 私钥文件 | 不复制文件，仅存储路径引用 |
| 日志脱敏 | 日志中不记录密码、密钥等敏感信息 |
| 进程隔离 | SSH 进程以当前用户权限运行，不提权 |

---

## 10. 系统托盘功能

### 托盘图标状态

| 状态 | 图标表现 |
|------|----------|
| 无隧道运行 | 灰色图标 |
| 有隧道运行 | 彩色图标 |
| 有隧道异常 | 图标带警告标识 |

### 托盘菜单

```
┌─────────────────────┐
│ ▶ 启动常用隧道      │
│   ─ 本地数据库隧道  │
│   ─ Redis调试隧道   │
│ ─────────────────── │
│ ■ 停止所有隧道      │
│ ─────────────────── │
│ ⚙ 打开主窗口        │
│ ✕ 退出             │
└─────────────────────┘
```

### 托盘功能

- 单击托盘图标：显示/隐藏主窗口
- 右键菜单：快速启停常用隧道、打开主窗口、退出程序
- 关闭窗口时最小化到托盘，不退出程序
