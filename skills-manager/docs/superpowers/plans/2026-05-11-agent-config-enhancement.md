# Agent Config Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-step. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add ability to create, edit, and delete Agents with path auto-resolution for ~/ to home directory.

**Architecture:** Add new methods to config store, create AgentDialog component, and refactor Settings.vue Agent section with edit/delete buttons.

**Tech Stack:** Vue 3, TDesign Vue Next, Pinia, uuid, Tauri API

---

## File Structure

| File | Change | Purpose |
|------|--------|---------|
| `src/stores/config.ts` | Modify | Add addAgent, updateAgentInfo, removeAgent methods |
| `src/components/AgentDialog.vue` | Create | Dialog for adding/editing Agent |
| `src/views/Settings.vue` | Modify | Add edit/delete buttons, integrate AgentDialog |

---

### Task 1: Add Agent CRUD Methods to Config Store

**Files:**
- Modify: `src/stores/config.ts`

- [ ] **Step 1: Add addAgent method**

Add after `batchUpdateAgents` function (around line 84):

```typescript
function addAgent(agent: Agent) {
  if (!config.value) return
  config.value.agents.push(agent)
  saveConfigData()
}
```

- [ ] **Step 2: Add updateAgentInfo method**

Add after `addAgent`:

```typescript
function updateAgentInfo(agentId: string, updates: { name?: string; path?: string }) {
  if (!config.value) return
  const agent = config.value.agents.find(a => a.id === agentId)
  if (agent) {
    Object.assign(agent, updates)
    saveConfigData()
  }
}
```

- [ ] **Step 3: Add removeAgent method**

Add after `updateAgentInfo`:

```typescript
function removeAgent(agentId: string) {
  if (!config.value) return
  config.value.agents = config.value.agents.filter(a => a.id !== agentId)
  saveConfigData()
}
```

- [ ] **Step 4: Update return statement**

Update the return statement to include new methods:

```typescript
return {
  config,
  loading,
  error,
  loadConfig,
  saveConfigData,
  addRepository,
  updateRepository,
  removeRepository,
  updateAgent,
  updateSettings,
  batchUpdateSettings,
  batchUpdateAgents,
  addAgent,
  updateAgentInfo,
  removeAgent,
}
```

- [ ] **Step 5: Import Agent type**

Ensure `Agent` type is imported from api/tauri (should already be imported).

- [ ] **Step 6: Commit**

```bash
git add src/stores/config.ts
git commit -m "feat: add Agent CRUD methods to config store"
```

---

### Task 2: Create AgentDialog Component

**Files:**
- Create: `src/components/AgentDialog.vue`

- [ ] **Step 1: Create AgentDialog.vue with template**

```vue
<template>
  <t-dialog
    v-model:visible="visible"
    :header="isEditMode ? '编辑 Agent' : '添加 Agent'"
    width="480px"
    :footer="true"
    @close="onClose"
  >
    <t-form :data="formData" :rules="formRules" ref="formRef" label-align="top">
      <t-form-item label="Agent 名称" name="name">
        <t-input v-model="formData.name" placeholder="输入 Agent 名称" />
      </t-form-item>

      <t-form-item label="Agent 路径" name="path">
        <t-input v-model="formData.path" placeholder="~/xxx 或完整路径" />
        <div class="path-hint">
          提示：支持 ~/ 开头表示家目录，如 ~/.claude<br />
          或输入完整路径，如 /home/user/.claude
        </div>
      </t-form-item>

      <t-form-item label="启用状态">
        <t-switch v-model="formData.enabled" />
      </t-form-item>
    </t-form>

    <template #footer>
      <t-button variant="outline" @click="onClose">取消</t-button>
      <t-button theme="primary" :loading="saving" @click="onSave">
        {{ isEditMode ? '保存' : '添加' }}
      </t-button>
    </template>
  </t-dialog>
</template>
```

- [ ] **Step 2: Add script section**

```vue
<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import { getSystemInfo } from '@/api/tauri'
import type { Agent } from '@/api/tauri'
import { v4 as uuidv4 } from 'uuid'

const configStore = useConfigStore()

const visible = defineModel<boolean>('visible', { default: false })
const editAgent = defineModel<Agent | null>('editAgent', { default: null })

const formRef = ref()
const saving = ref(false)
const homeDir = ref('')

const formData = ref({
  name: '',
  path: '',
  enabled: true,
})

const formRules = {
  name: [{ required: true, message: 'Agent 名称必填' }],
  path: [{ required: true, message: 'Agent 路径必填' }],
}

const isEditMode = computed(() => !!editAgent.value)

// Reset form when dialog opens
watch(visible, async (val) => {
  if (val) {
    // Get home directory for path resolution
    try {
      const sysInfo = await getSystemInfo()
      homeDir.value = sysInfo.home_dir
    } catch {
      homeDir.value = ''
    }

    // Prefill form for edit mode
    if (editAgent.value) {
      formData.value = {
        name: editAgent.value.name,
        path: editAgent.value.path,
        enabled: editAgent.value.enabled,
      }
    } else {
      formData.value = {
        name: '',
        path: '',
        enabled: true,
      }
    }
  }
})

function resolvePath(inputPath: string): string {
  if (!homeDir.value) return inputPath
  // Support ~/ and ~\ (Windows)
  if (inputPath.startsWith('~/') || inputPath.startsWith('~\\')) {
    return homeDir.value + inputPath.slice(1)
  }
  return inputPath
}

async function onSave() {
  const valid = await formRef.value?.validate()
  if (valid !== true) return

  saving.value = true
  try {
    const resolvedPath = resolvePath(formData.value.path)

    if (isEditMode.value && editAgent.value) {
      // Update existing agent
      configStore.updateAgentInfo(editAgent.value.id, {
        name: formData.value.name,
        path: resolvedPath,
      })
      // Also update enabled state if changed
      if (formData.value.enabled !== editAgent.value.enabled) {
        configStore.updateAgent(editAgent.value.id, formData.value.enabled)
      }
      MessagePlugin.success('Agent 已更新')
    } else {
      // Create new agent
      const newAgent: Agent = {
        id: uuidv4(),
        name: formData.value.name,
        path: resolvedPath,
        enabled: formData.value.enabled,
      }
      configStore.addAgent(newAgent)
      MessagePlugin.success('Agent 已添加')
    }

    visible.value = false
    editAgent.value = null
  } finally {
    saving.value = false
  }
}

function onClose() {
  visible.value = false
  editAgent.value = null
}
</script>
```

- [ ] **Step 3: Add style section**

```vue
<style scoped>
.path-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  line-height: 1.5;
}
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/components/AgentDialog.vue
git commit -m "feat: create AgentDialog component for add/edit Agent"
```

---

### Task 3: Refactor Settings.vue Agent Section

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: Import AgentDialog and DialogPlugin**

Update imports in script section:

```typescript
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getVersion } from '@tauri-apps/api/app'
import { DialogPlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import type { Settings, Agent } from '@/api/tauri'
import AgentDialog from '@/components/AgentDialog.vue'
```

- [ ] **Step 2: Add AgentDialog state refs**

Add after `showConfirmDialog` ref:

```typescript
const showAgentDialog = ref(false)
const editingAgent = ref<Agent | null>(null)
```

- [ ] **Step 3: Add Agent CRUD handlers**

Add before `onMounted`:

```typescript
function handleAddAgent() {
  editingAgent.value = null
  showAgentDialog.value = true
}

function handleEditAgent(agent: Agent) {
  editingAgent.value = agent
  showAgentDialog.value = true
}

function handleDeleteAgent(agent: Agent) {
  const confirmDialog = DialogPlugin.confirm({
    header: '确认删除',
    body: `确定删除 Agent "${agent.name}"？`,
    confirmBtn: { content: '删除', theme: 'danger' },
    cancelBtn: '取消',
    onConfirm: () => {
      configStore.removeAgent(agent.id)
      confirmDialog.hide()
    },
  })
}
```

- [ ] **Step 4: Update template - Agent section header with add button**

Replace the Agent section header:

```vue
<!-- Agent 配置 -->
<section id="agent-config" class="settings-section">
  <div class="section-header">
    <div class="section-title">Agent 配置</div>
    <t-button size="small" @click="handleAddAgent">
      添加 Agent
    </t-button>
  </div>
```

- [ ] **Step 5: Update template - Agent item with edit/delete buttons**

Replace the agent-item template:

```vue
<div v-if="!configStore.config?.agents?.length" class="empty-tip">
  暂无已配置的 Agent
</div>
<div v-else class="agent-list">
  <div v-for="agent in configStore.config?.agents" :key="agent.id" class="agent-item">
    <div class="agent-info">
      <span class="agent-name">{{ agent.name }}</span>
      <code class="agent-path">{{ agent.path }}</code>
    </div>
    <div class="agent-actions">
      <t-button size="small" variant="text" @click="handleEditAgent(agent)">
        编辑
      </t-button>
      <t-button size="small" variant="text" theme="danger" @click="handleDeleteAgent(agent)">
        删除
      </t-button>
      <t-switch
        :value="getPendingAgentEnabled(agent.id)"
        @change="(value: boolean) => handleAgentChange(agent.id, value)"
      />
    </div>
  </div>
</div>
```

- [ ] **Step 6: Update template - Add AgentDialog component**

Add after the unsaved changes dialog:

```vue
<!-- Agent Dialog -->
<AgentDialog
  v-model:visible="showAgentDialog"
  v-model:editAgent="editingAgent"
/>
```

- [ ] **Step 7: Add styles for section header and agent actions**

Add after `.section-title` style:

```css
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--td-component-border);
}

.section-header .section-title {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.agent-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
```

- [ ] **Step 8: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: add Agent edit/delete functionality to Settings page"
```

---

### Task 4: Test in Browser

**Files:**
- None (verification only)

- [ ] **Step 1: Verify layout**

Navigate to `http://localhost:1420/settings` and verify:
- Agent section header has "添加 Agent" button on right side
- Each Agent item has edit/delete buttons + switch
- Empty state shows "暂无已配置的 Agent"

- [ ] **Step 2: Test add Agent**

Click "添加 Agent" → Dialog opens
Enter name and path (e.g., `~/test`) → Click "添加"
→ Agent appears in list with resolved path

- [ ] **Step 3: Test edit Agent**

Click edit button → Dialog opens with prefilled data
Modify name or path → Click "保存"
→ Agent updated in list

- [ ] **Step 4: Test delete Agent**

Click delete button → Confirm dialog appears
Click "删除" → Agent removed from list
Click "取消" → Agent remains

- [ ] **Step 5: Test path resolution**

Add Agent with path `~/xxx` → Verify stored path is resolved to homeDir
Add Agent with path `/absolute/path` → Verify stored path unchanged

---

## Spec Coverage Check

| Spec Requirement | Task |
|------------------|------|
| 支持添加自定义 Agent（名称 + 路径） | Task 1, Task 2, Task 3 |
| 支持修改现有 Agent 的名称和路径 | Task 1, Task 2, Task 3 |
| 支持删除 Agent | Task 1, Task 3 |
| Agent 路径支持 ~/ 自动解析 | Task 2 |
| "添加 Agent" 按钮位于标题右侧 | Task 3 |
| 路径输入提示说明 | Task 2 |
| 删除确认对话框 | Task 3 |