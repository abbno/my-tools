---
title: Agent Configuration Enhancement
date: 2026-05-11
status: approved
---

# Agent 配置增强设计

## 目标

1. 支持添加自定义 Agent（名称 + 路径）
2. 支持修改现有 Agent 的名称和路径
3. 支持删除 Agent
4. Agent 路径支持 `~/` 自动解析为系统家目录

## 用户界面

### Agent 列表区域

```
┌─────────────────────────────────────────────────────────┐
│ Agent 配置                              [+ 添加 Agent]  │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Agent 名称: Claude Code                             │ │
│ │ 路径: /home/user/.claude                  [编辑][删除]│ │
│ │ 启用: [Switch ON]                                   │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Agent 名称: Cursor                                  │ │
│ │ 路径: ~/cursor                        [编辑][删除]    │ │
│ │ 启用: [Switch OFF]                                  │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                         │
│ 暂无已配置的 Agent（空状态时显示）                        │
└─────────────────────────────────────────────────────────┘
```

**布局**：
- "添加 Agent" 按钮位于 "Agent 配置" 标题右侧
- 每个 Agent 项包含：名称、路径、启用开关、编辑按钮、删除按钮
- 空状态显示提示文字 + 添加按钮

### AgentDialog 对话框

**添加模式**：
```
┌─────────────────────────────────────────────────┐
│  添加 Agent                                [X]  │
├─────────────────────────────────────────────────┤
│  Agent 名称 *                                   │
│  ┌───────────────────────────────────────────┐ │
│  │ [输入名称]                                 │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Agent 路径 *                                   │
│  ┌───────────────────────────────────────────┐ │
│  │ ~/xxx  (支持 ~ 自动解析为家目录)           │ │
│  └───────────────────────────────────────────┘ │
│  提示：支持 ~/ 开头表示家目录，如 ~/.claude     │
│       或输入完整路径，如 /home/user/.claude    │
│                                                 │
│  启用状态                                       │
│  [Switch] 默认启用                              │
│                                                 │
├─────────────────────────────────────────────────┤
│              [取消]    [添加]                   │
└─────────────────────────────────────────────────┘
```

**编辑模式**：
- 标题改为 "编辑 Agent"
- 按钮改为 "保存"
- 表单预填充现有 Agent 数据

## 交互流程

### 添加 Agent

```
点击"添加 Agent" → 打开 AgentDialog（添加模式）
输入名称和路径 → 点击"添加"
→ 解析路径（~/ 替换为家目录）
→ 生成 UUID 作为 id
→ 调用 addAgent(agent)
→ 关闭对话框
```

### 编辑 Agent

```
点击 Agent 项的"编辑"按钮 → 打开 AgentDialog（编辑模式）
修改名称或路径 → 点击"保存"
→ 解析路径（~/ 替换为家目录）
→ 调用 updateAgentInfo(agentId, updates)
→ 关闭对话框
```

### 删除 Agent

```
点击 Agent 项的"删除"按钮
→ 弹出确认对话框："确定删除 Agent xxx？"
→ 用户确认 → 调用 removeAgent(agentId)
→ 用户取消 → 关闭确认对话框
```

## 路径解析

### 解析逻辑

```typescript
function resolvePath(inputPath: string, homeDir: string): string {
  // 支持 ~/ 和 ~\ (Windows)
  if (inputPath.startsWith('~/') || inputPath.startsWith('~\\')) {
    return homeDir + inputPath.slice(1)
  }
  return inputPath
}
```

### 家目录获取

- 使用现有 `getSystemInfo()` API 获取 `home_dir`
- 在 Settings.vue mounted 时获取并缓存

### 解析时机

- **保存时解析**：用户输入原始值（如 ~/xxx），保存时转换为绝对路径存储
- **不实时转换**：输入框保持用户原始输入可见
- **编辑时回显**：显示存储的绝对路径（不反向转换为 ~）

### 示例

| 用户输入 | Linux 存储 | Windows 存储 |
|---------|-----------|-------------|
| `~/skills` | `/home/user/skills` | `C:\Users\zz\skills` |
| `~\.claude` | - | `C:\Users\zz\.claude` |
| `/absolute/path` | `/absolute/path` | - |
| `C:\absolute` | - | `C:\absolute` |

## 数据结构

### Agent 类型（现有，无需修改）

```typescript
export interface Agent {
  id: string        // UUID
  name: string      // Agent 名称
  path: string      // 解析后的绝对路径
  enabled: boolean  // 启用状态
}
```

### Config Store 新增方法

```typescript
// 添加新 Agent
function addAgent(agent: Agent): void {
  if (!config.value) return
  config.value.agents.push(agent)
  saveConfigData()
}

// 更新 Agent 信息（名称、路径）
function updateAgentInfo(agentId: string, updates: { name?: string; path?: string }): void {
  if (!config.value) return
  const agent = config.value.agents.find(a => a.id === agentId)
  if (agent) {
    Object.assign(agent, updates)
    saveConfigData()
  }
}

// 删除 Agent
function removeAgent(agentId: string): void {
  if (!config.value) return
  config.value.agents = config.value.agents.filter(a => a.id !== agentId)
  saveConfigData()
}
```

**注意**：现有 `updateAgent` 方法只处理 enabled 状态，新增 `updateAgentInfo` 处理名称和路径。

## 文件变更

| 文件 | 变更 | 说明 |
|------|------|------|
| `src/views/Settings.vue` | 修改 | 重构 Agent 配置区域，添加编辑/删除按钮 |
| `src/components/AgentDialog.vue` | 新建 | Agent 添加/编辑对话框 |
| `src/stores/config.ts` | 修改 | 新增 addAgent, updateAgentInfo, removeAgent 方法 |

## 组件依赖

- TDesign Dialog：AgentDialog 使用 t-dialog
- TDesign DialogPlugin：删除确认使用 DialogPlugin.confirm()
- TDesign Form：表单验证（名称和路径必填）
- TDesign Input：名称和路径输入
- TDesign Switch：启用状态开关
- TDesign Button：操作按钮
- uuid：生成 Agent ID