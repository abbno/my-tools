# Settings Page Layout Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign Settings page with fixed header/footer, sidebar navigation, and deferred save functionality.

**Architecture:** Replace t-card layout with fixed-position header/footer + flex layout for sidebar/main content. Add pending settings state for batch updates before save.

**Tech Stack:** Vue 3, TDesign Vue Next, Tauri v2 API, Pinia

---

## File Structure

| File | Change | Purpose |
|------|--------|---------|
| `src/stores/config.ts` | Modify | Add `batchUpdateSettings` and `batchUpdateAgents` methods |
| `src/views/Settings.vue` | Rewrite | New layout with header/sidebar/main/footer, pending state, save logic |

---

### Task 1: Add Batch Update Methods to Config Store

**Files:**
- Modify: `src/stores/config.ts`

- [ ] **Step 1: Add batch update methods**

Add two new methods that update state without auto-saving:

```typescript
// Add after updateSettings function (around line 69)

function batchUpdateSettings(updates: Partial<Settings>) {
  if (!config.value) return
  Object.assign(config.value.settings, updates)
}

function batchUpdateAgents(updates: Array<{ id: string; enabled: boolean }>) {
  if (!config.value) return
  for (const update of updates) {
    const agent = config.value.agents.find(a => a.id === update.id)
    if (agent) {
      agent.enabled = update.enabled
    }
  }
}
```

- [ ] **Step 2: Export new methods**

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
}
```

- [ ] **Step 3: Commit**

```bash
git add src/stores/config.ts
git commit -m "feat: add batch update methods to config store for deferred save"
```

---

### Task 2: Restructure Settings.vue Template and Base Styles

**Files:**
- Modify: `src/views/Settings.vue`

- [ ] **Step 1: Rewrite template with new layout**

Replace the entire `<template>` section:

```vue
<template>
  <div class="settings-page">
    <!-- Header (fixed) -->
    <header class="settings-header">
      <div class="header-title">设置</div>
      <div class="header-actions">
        <t-button variant="outline" @click="handleBackClick">
          返回主页
        </t-button>
        <t-button
          :theme="pendingChanges ? 'primary' : 'default'"
          :loading="saving"
          @click="handleSave"
        >
          {{ pendingChanges ? '保存更改' : '保存' }}
        </t-button>
      </div>
    </header>

    <!-- Main Content -->
    <div class="settings-body">
      <!-- Sidebar -->
      <aside class="settings-sidebar">
        <t-menu :value="activeSection" @change="handleSectionChange">
          <t-menu-item value="agent-config">
            Agent 配置
          </t-menu-item>
          <t-menu-item value="sync-settings">
            同步设置
          </t-menu-item>
        </t-menu>
      </aside>

      <!-- Main Area -->
      <main ref="mainAreaRef" class="settings-main">
        <div class="settings-content">
          <!-- Agent 配置 -->
          <section id="agent-config" class="settings-section">
            <div class="section-title">Agent 配置</div>
            <div v-if="!configStore.config?.agents?.length" class="empty-tip">
              暂无已配置的 Agent
            </div>
            <div v-else class="agent-list">
              <div v-for="agent in configStore.config?.agents" :key="agent.id" class="agent-item">
                <div class="agent-info">
                  <span class="agent-name">{{ agent.name }}</span>
                  <code class="agent-path">{{ agent.path }}</code>
                </div>
                <t-switch
                  :value="getPendingAgentEnabled(agent.id)"
                  @change="(value: boolean) => handleAgentChange(agent.id, value)"
                />
              </div>
            </div>
          </section>

          <!-- 同步设置 -->
          <section id="sync-settings" class="settings-section">
            <div class="section-title">同步设置</div>
            <div class="settings-row">
              <div class="settings-info">
                <span class="settings-label">自动同步</span>
                <span class="settings-desc">自动同步仓库</span>
              </div>
              <t-switch
                :value="pendingSettings.auto_sync ?? configStore.config?.settings.auto_sync"
                @change="(value: boolean) => handleSettingChange('auto_sync', value)"
              />
            </div>
            <div class="settings-row">
              <div class="settings-info">
                <span class="settings-label">默认同步间隔</span>
                <span class="settings-desc">自动同步的频率</span>
              </div>
              <t-select
                :value="pendingSettings.default_sync_interval ?? configStore.config?.settings.default_sync_interval ?? 3600"
                @change="(value: number) => handleSettingChange('default_sync_interval', value)"
                :options="syncIntervalOptions"
                style="width: 140px"
              />
            </div>
          </section>
        </div>
      </main>
    </div>

    <!-- Footer (fixed) -->
    <footer class="settings-footer">
      <span class="footer-brand">◈</span>
      <span class="footer-name">Skills Manager</span>
      <span class="footer-version">v{{ appVersion }}</span>
    </footer>

    <!-- Unsaved Changes Dialog -->
    <t-dialog
      v-model:visible="showConfirmDialog"
      header="提示"
      body="设置未保存，是否放弃修改？"
      :confirm-btn="{ content: '放弃修改', theme: 'danger' }"
      :cancel-btn="{ content: '继续编辑' }"
      @confirm="handleDiscardChanges"
    />
  </div>
</template>
```

- [ ] **Step 2: Rewrite script with new state and logic**

Replace the entire `<script setup>` section:

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getVersion } from '@tauri-apps/api/app'
import { useConfigStore } from '@/stores/config'
import type { Settings } from '@/api/tauri'

const router = useRouter()
const configStore = useConfigStore()

// Refs
const mainAreaRef = ref<HTMLElement | null>(null)
const appVersion = ref('0.1.0')
const activeSection = ref('agent-config')
const saving = ref(false)
const showConfirmDialog = ref(false)

// Pending state
const pendingSettings = ref<Partial<Settings>>({})
const pendingAgentChanges = ref<Map<string, boolean>>(new Map())
const pendingChanges = ref(false)

// Sync interval options
const syncIntervalOptions = [
  { label: '5 分钟', value: 300 },
  { label: '15 分钟', value: 900 },
  { label: '30 分钟', value: 1800 },
  { label: '1 小时', value: 3600 },
  { label: '2 小时', value: 7200 },
  { label: '6 小时', value: 21600 },
  { label: '12 小时', value: 43200 },
  { label: '每天', value: 86400 },
]

// Get pending agent enabled state
function getPendingAgentEnabled(agentId: string): boolean {
  const pending = pendingAgentChanges.value.get(agentId)
  if (pending !== undefined) return pending
  const agent = configStore.config?.agents.find(a => a.id === agentId)
  return agent?.enabled ?? false
}

// Handle agent change
function handleAgentChange(agentId: string, enabled: boolean) {
  const original = configStore.config?.agents.find(a => a.id === agentId)?.enabled ?? false
  pendingAgentChanges.value.set(agentId, enabled)
  updatePendingChangesState()
}

// Handle setting change
function handleSettingChange(key: keyof Settings, value: boolean | number) {
  pendingSettings.value[key] = value as any
  updatePendingChangesState()
}

// Check if there are actual pending changes
function updatePendingChangesState() {
  // Check settings changes
  const originalSettings = configStore.config?.settings
  let hasSettingsChanges = false
  for (const [key, value] of Object.entries(pendingSettings.value)) {
    if (originalSettings && originalSettings[key as keyof Settings] !== value) {
      hasSettingsChanges = true
      break
    }
  }

  // Check agent changes
  let hasAgentChanges = false
  for (const [agentId, enabled] of pendingAgentChanges.value) {
    const original = configStore.config?.agents.find(a => a.id === agentId)?.enabled ?? false
    if (original !== enabled) {
      hasAgentChanges = true
      break
    }
  }

  pendingChanges.value = hasSettingsChanges || hasAgentChanges
}

// Handle save
async function handleSave() {
  if (!pendingChanges.value) return
  
  saving.value = true
  try {
    // Apply pending settings
    if (Object.keys(pendingSettings.value).length > 0) {
      configStore.batchUpdateSettings(pendingSettings.value)
    }
    
    // Apply pending agent changes
    if (pendingAgentChanges.value.size > 0) {
      const agentUpdates = Array.from(pendingAgentChanges.value.entries()).map(([id, enabled]) => ({ id, enabled }))
      configStore.batchUpdateAgents(agentUpdates)
    }
    
    // Save to file
    await configStore.saveConfigData()
    
    // Clear pending state
    pendingSettings.value = {}
    pendingAgentChanges.value.clear()
    pendingChanges.value = false
  } catch (error) {
    console.error('Failed to save settings:', error)
  } finally {
    saving.value = false
  }
}

// Handle back click
function handleBackClick() {
  if (pendingChanges.value) {
    showConfirmDialog.value = true
  } else {
    router.push('/')
  }
}

// Handle discard changes
function handleDiscardChanges() {
  pendingSettings.value = {}
  pendingAgentChanges.value.clear()
  pendingChanges.value = false
  router.push('/')
}

// Handle section change (scroll to anchor)
function handleSectionChange(value: string) {
  activeSection.value = value
  if (mainAreaRef.value) {
    const target = mainAreaRef.value.querySelector(`#${value}`)
    if (target) {
      target.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }
}

// Fetch version on mount
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.1.0'
  }
})
</script>
```

- [ ] **Step 3: Rewrite styles**

Replace the entire `<style scoped>` section:

```vue
<style scoped>
.settings-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--td-bg-color-page);
}

/* Header */
.settings-header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 64px;
  background: #fff;
  border-bottom: 1px solid var(--td-component-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 24px;
  z-index: 100;
}

.header-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--td-text-color-primary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

/* Body */
.settings-body {
  flex: 1;
  margin-top: 64px;
  margin-bottom: 48px;
  display: flex;
  overflow: hidden;
}

/* Sidebar */
.settings-sidebar {
  width: 200px;
  background: #fff;
  border-right: 1px solid var(--td-component-border);
  padding: 16px 0;
}

.settings-sidebar :deep(.t-menu) {
  background: transparent;
}

/* Main */
.settings-main {
  flex: 1;
  overflow-y: auto;
  background: var(--td-bg-color-page);
}

.settings-content {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

/* Sections */
.settings-section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--td-component-border);
}

/* Agent list */
.agent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-container);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  display: block;
}

.agent-path {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background: var(--td-bg-color-secondarycontainer);
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-block;
  margin-top: 4px;
}

/* Settings row */
.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-container);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
  margin-bottom: 8px;
}

.settings-info {
  flex: 1;
}

.settings-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  display: block;
}

.settings-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  display: block;
  margin-top: 2px;
}

.empty-tip {
  color: var(--td-text-color-placeholder);
  font-size: 14px;
  padding: 16px 0;
}

/* Footer */
.settings-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 48px;
  background: #fff;
  border-top: 1px solid var(--td-component-border);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  z-index: 100;
}

.footer-brand {
  color: var(--td-brand-color);
  font-size: 18px;
}

.footer-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.footer-version {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
}
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/views/Settings.vue
git commit -m "feat: redesign Settings page with fixed header/footer, sidebar, and deferred save"
```

---

### Task 3: Test in Browser

**Files:**
- None (verification only)

- [ ] **Step 1: Verify layout**

Navigate to `http://localhost:1420/settings` and verify:
- Header is fixed at top with title and buttons
- Footer is fixed at bottom with logo, name, version
- Sidebar shows menu items
- Main area scrolls independently
- No double scrollbar

- [ ] **Step 2: Verify pending state**

Test the save functionality:
- Toggle a switch → "保存" button changes to "保存更改" with primary theme
- Click save → button returns to normal state
- Toggle a switch → click "返回主页" → dialog appears asking to discard
- Click "继续编辑" → stays on page
- Click "放弃修改" → returns to home, changes discarded

- [ ] **Step 3: Verify sidebar navigation**

Test anchor scrolling:
- Click "Agent 配置" → scrolls to agent section
- Click "同步设置" → scrolls to sync section
- Scroll is smooth

---

## Spec Coverage Check

| Spec Requirement | Task |
|------------------|------|
| 固定顶部标题栏和返回主页按钮，增加保存按钮 | Task 2 |
| 固定底部 logo、名称、版本信息 | Task 2 |
| 增加左侧设置目录索引 | Task 2 |
| 设置变更需要保存后才生效 | Task 1, Task 2 |
| 去除 Card，改用更简洁的布局 | Task 2 |
| 版本号通过 Tauri API 获取 | Task 2 |
| 点击返回时未保存提示 | Task 2 |
| 锚点滚动 | Task 2 |