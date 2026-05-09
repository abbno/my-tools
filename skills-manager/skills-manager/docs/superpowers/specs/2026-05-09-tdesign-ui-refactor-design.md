# Skills Manager TDesign UI 重构设计文档

## 概述

将 skills-manager 项目从自定义 CSS 样式重构为使用 TDesign Vue-next 组件库，采用 TDesign 默认风格（浅色主题），同时重新设计页面布局结构。

## 设计决策

| 决策项 | 选择 |
|--------|------|
| UI 风格 | TDesign 默认风格 |
| 页面布局 | TDesign Layout 组件重新设计 |
| 导航方式 | 侧边栏导航 |
| 技能展示 | 卡片网格 |
| 详情展示 | 右侧滑入面板 |

## 整体布局架构

```
┌─────────────────────────────────────────────────────────────┐
│                    顶部 Header (t-header)                    │
│  [品牌 Logo] [搜索框 t-input]                [设置按钮]     │
├──────────────┬──────────────────────────────────────────────┤
│              │                                               │
│   侧边栏     │              内容区                            │
│   (t-menu)   │                                               │
│              │  ┌─────────────────────────────────────────┐  │
│  [仓库列表]  │  │  技能卡片网格 (t-card grid)              │  │
│              │  │  [t-card] [t-card] [t-card]              │  │
│  [同步按钮]  │  │  [t-card] [t-card] [t-card]              │  │
│              │  └─────────────────────────────────────────┘  │
│              │                                               │
│              │         ← 右侧详情面板 (点击卡片时滑入)       │
│              │                                               │
└──────────────┴──────────────────────────────────────────────┤
```

### TDesign 组件使用

- `t-layout` + `t-header` + `t-aside` + `t-content` 构建整体框架
- `t-menu` 作为侧边栏导航（仓库列表 + 设置入口）
- `t-input` 作为搜索框（带搜索图标前缀）
- `t-card` 展示每个技能
- 设置功能使用 `t-dialog` 弹窗

## 侧边栏设计

```
┌─────────────────────┐
│  ◈ Skills Manager   │  ← 品牌区
├─────────────────────┤
│                     │
│  仓库列表区         │  ← t-menu-group
│  ┌─────────────────┐│
│  │ ○ Repo A        ││  ← t-menu-item (选中高亮)
│  │ ○ Repo B        ││
│  │ ○ Repo C        ││
│  └─────────────────┘│
│                     │
│  [+ Add Repo]       │  ← t-button
│  [↻ Sync All]       │  ← t-button
│                     │
├─────────────────────┤
│  ⚙ Settings         │  ← t-menu-item
└─────────────────────┘
```

### TDesign 组件

- `t-menu` 可折叠模式，`theme="light"`
- `t-menu-group` 分组仓库列表
- `t-menu-item` 每个仓库项，选中状态控制当前仓库
- `t-button` 添加仓库、同步按钮
- 同步状态使用 `t-tag` 或 `t-badge` 标识

### 交互行为

- 点击仓库项切换当前仓库，筛选显示该仓库的技能
- 点击 "Add Repo" 打开添加仓库对话框（多步骤表单）
- 点击 "Settings" 打开设置对话框

## 技能卡片网格设计

### 卡片内容结构

```
┌─────────────────────────────────────┐
│  ◈ Skill Name                       │  ← 标题区
│                                     │
│  Skill description text here,       │  ← 描述
│  showing brief info about...        │
│                                     │
│  ┌─────────────────────────────────┐│
│  │ Source Repo Name   │ 2024-01-15 ││  ← 元信息区
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### TDesign 组件

- `t-card` 作为卡片容器，`bordered` 带边框，`hover-shadow` 悬停效果
- 标题：`<template #header>` 或自定义标题区
- 描述：默认内容区
- 元信息：`<template #footer>` 底部展示仓库来源

### 网格布局

- CSS Grid：`grid-template-columns: repeat(auto-fill, minmax(280px, 1fr))`
- 或 TDesign `t-row` + `t-col` 响应式栅格

### 交互行为

- 悬停卡片：`hoverable` 属性，阴影加深效果
- 点击卡片：右侧滑入详情面板，卡片变为选中状态
- 空状态：使用 `t-empty` 组件显示 "No skills found"

## 右侧详情面板设计

### 面板结构

```
┌─────────────────────────────────────┐
│  ◈ Skill Name              [× 关闭] │  ← 标题栏
├─────────────────────────────────────┤
│                                     │
│  DESCRIPTION                        │
│  Full description of the skill...   │
│                                     │
│  ─────────────────────────────────  │  ← t-divider
│                                     │
│  SOURCE REPOSITORY                  │
│  [Repo Name]                        │  ← t-tag
│                                     │
│  PATH                               │
│  /path/to/skill/file.md             │
│                                     │
│  ─────────────────────────────────  │
│                                     │
│  SKILL.MD PREVIEW                   │
│  Loading preview content...         │
│                                     │
├─────────────────────────────────────┤
│  [→ Open in Editor]  [⟳ Refresh]   │  ← t-button
└─────────────────────────────────────┘
```

### TDesign 组件

- 面板容器：自定义固定宽度容器 + CSS transition 动画滑入
- 分隔线：`t-divider`
- 标签：`t-tag` 展示仓库来源
- 操作按钮：`t-button` `variant="outline"` 和 `variant="base"`

### 交互行为

- 滑入动画：Vue `<Transition>` 或 CSS `transform: translateX()`
- 点击关闭按钮或点击其他卡片时关闭面板
- 面板宽度固定约 380px

## 添加仓库对话框 (AddRepoDialog)

保持现有三步骤流程，使用 TDesign 组件重构。

### TDesign 组件

- `t-dialog` 对话框容器，`width="600px"`，`destroy-on-close`
- `t-steps` 步骤指示器替代自定义进度条
- `t-form` + `t-form-item` 表单布局
- `t-input` 输入框，`t-select` 下拉选择
- `t-checkbox-group` + `t-checkbox` 技能选择
- `t-descriptions` 汇总信息展示

### 步骤流程

1. **步骤1：表单输入** - 填写仓库名称、URL、认证方式、同步间隔
2. **步骤2：技能预览** - 显示仓库中的技能列表，勾选需要同步的技能
3. **步骤3：确认摘要** - 使用 `t-descriptions` 展示汇总信息，确认添加

## 设置对话框

### 结构

```
┌─────────────────────────────────────────────┐
│  ⚙ Settings                         [×]     │
├─────────────────────────────────────────────┤
│                                             │
│  Agent Configuration                        │
│  ┌─────────────────────────────┬───────┐    │
│  │ Agent Name                  │ [开关]│    │  ← t-switch
│  │ /path/to/agent              │       │    │
│  └─────────────────────────────┴───────┘    │
│                                             │
│  Sync Settings                              │
│  Auto Sync: [t-switch]                      │
│  Interval:  [t-select]                      │
│                                             │
│  About                                      │
│  Skills Manager v0.1.0                      │
│                                             │
├─────────────────────────────────────────────┤
│                              [Close]        │
└─────────────────────────────────────────────┘
```

### TDesign 组件

- `t-dialog` 对话框容器
- `t-collapse` 或分组标题展示各设置区
- `t-switch` 开关控件（Agent 启用/禁用、Auto Sync）
- `t-select` 同步间隔选择
- `t-empty` 当无 Agent 时显示空状态

## Git 安装对话框

### TDesign 组件

- `t-dialog` 对话框容器
- `t-alert` `theme="warning"` 警告提示
- `t-link` 下载链接
- `t-button` 操作按钮

## 冲突对话框

### TDesign 组件

- `t-dialog` 对话框容器
- `t-alert` `theme="warning"` 警告提示
- `t-radio-group` + `t-radio` 选项选择

## 其他交互细节

- 搜索框：`t-input` 带 `prefix-icon` 搜索图标
- 同步进度：`t-progress` 线性进度条，或 `t-loading` 加载状态
- 成功/失败提示：`t-message` 全局提示
- 空状态：`t-empty` 组件

## 文件结构规划

```
src/
├── App.vue              # 重构为 Layout 框架
├── views/
│   └── Home.vue         # 主页面（合并 Settings 功能）
│   ├── AddRepoDialog.vue    # 添加仓库对话框
│   ├── SettingsDialog.vue   # 新建：设置对话框
│   └── SkillDetailPanel.vue # 新建：详情面板组件
├── components/
│   ├── GitInstallDialog.vue # Git 安装对话框
│   ├── ConflictDialog.vue   # 冲突对话框
│   └── SkillCard.vue        # 新建：技能卡片组件
├── stores/              # 保持现有状态管理
├── api/                 # 保持现有 API 调用
└── main.ts              # 引入 TDesign 样式
```

## 实现优先级

1. 整体 Layout 框架 + 侧边栏导航
2. 技能卡片网格 + 详情面板
3. 添加仓库对话框重构
4. 设置对话框新建
5. Git 安装对话框重构
6. 冲突对话框重构
7. 全局样式和交互优化