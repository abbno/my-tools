---
title: Settings Page Layout Redesign
date: 2026-05-11
status: approved
---

# Settings 页面布局重构设计

## 目标

1. 固定顶部标题栏和返回主页按钮，增加保存按钮
2. 固定底部 logo、名称、版本信息
3. 增加左侧设置目录索引
4. 设置变更需要保存后才生效
5. 去除 Card，改用更简洁的布局

## 页面结构

```
┌─────────────────────────────────────────────────────────────┐
│  HEADER (fixed, 64px)                                       │
│  [设置]                          [返回主页]  [保存]          │
├─────────────────────────────────────────────────────────────┤
│  CONTENT (calc(100vh - 64px - 48px))                        │
│  ┌─────────────┬───────────────────────────────────────────┤│
│  │ SIDEBAR     │  MAIN AREA (scrollable)                   ││
│  │ (~200px)    │                                           ││
│  │             │  ┌───────────────────────────────────────┐││
│  │ [Agent配置] │  │ Agent 配置 (id="agent-config")        │││
│  │ [同步设置]  │  │ - Agent 列表                          │││
│  │             │  └───────────────────────────────────────┘││
│  │             │  ┌───────────────────────────────────────┐││
│  │             │  │ 同步设置 (id="sync-settings")         │││
│  │             │  │ - 自动同步                            │││
│  │             │  │ - 同步间隔                            │││
│  │             │  └───────────────────────────────────────┘││
│  └─────────────┴───────────────────────────────────────────┤│
├─────────────────────────────────────────────────────────────┤
│  FOOTER (fixed, ~48px)                                      │
│  ◈ Skills Manager · v{{ appVersion }}                       │
└─────────────────────────────────────────────────────────────┘
```

## 组件与样式

### Header
- 全屏宽度，固定在顶部 (`position: fixed; top: 0`)
- 高度 64px，背景白色，底部边框
- 左侧：标题"设置"
- 右侧：按钮组（返回主页 + 保存）
- 保存按钮状态：
  - 默认：普通样式
  - 有未保存变更：primary 主题

### Sidebar
- 固定宽度 ~200px
- 使用 TDesign Menu 组件
- 菜单项：Agent配置、同步设置
- 点击跳转到锚点

### Main Area
- 去除 t-card，直接显示区块
- 最大宽度 800px，居中，padding 24px
- `overflow-y: auto`，独立滚动
- 各区块设置 id 作为锚点

### Footer
- 全屏宽度，固定在底部 (`position: fixed; bottom: 0`)
- 高度 ~48px
- 内容：logo (◈) + 名称 + 版本号，居中或左对齐
- 版本号通过 Tauri API 动态获取

## 保存功能逻辑

### 状态管理
- `pendingSettings` ref：暂存用户的修改值
- `pendingChanges` ref boolean：追踪是否有未保存变更

### 交互流程
```
用户修改设置 → 暂存到 pendingSettings → pendingChanges = true → 保存按钮高亮
点击保存 → 写入 configStore → 调用保存 API → pendingChanges = false → 显示成功提示
点击返回 → 若 pendingChanges = true → 弹出确认框："设置未保存，是否放弃修改？"
```

### 确认对话框
- 使用 TDesign Dialog 组件
- 内容："设置未保存，是否放弃修改？"
- 选项：放弃修改 / 继续编辑

## 版本号获取

- 使用 `@tauri-apps/api/app` 的 `getVersion()` API
- 组件 mounted 时获取，存储到 ref `appVersion`
- 失败时 fallback 为 "0.1.0"

```typescript
import { getVersion } from '@tauri-apps/api/app'

const appVersion = ref('')

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.1.0'
  }
})
```

## 锚点滚动

- Main Area 设置区块 id：`agent-config`、`sync-settings`
- 点击 Sidebar 菜单项 → `scrollIntoView({ behavior: 'smooth' })`
- 滚动容器为 Main Area（不是整个页面）

## 文件变更

- `src/views/Settings.vue`：重构模板和样式，添加保存逻辑
- 可能需要修改 `src/stores/config.ts`：添加批量保存方法