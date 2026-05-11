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
            <div class="section-header">
              <div class="section-title">Agent 配置</div>
              <t-button size="small" @click="handleAddAgent">
                添加 Agent
              </t-button>
            </div>
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

    <!-- Agent Dialog -->
    <AgentDialog
      v-model:visible="showAgentDialog"
      v-model:editAgent="editingAgent"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { getVersion } from '@tauri-apps/api/app'
import { DialogPlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import type { Settings, Agent } from '@/api/tauri'
import AgentDialog from '@/components/AgentDialog.vue'

const router = useRouter()
const configStore = useConfigStore()

// Refs
const mainAreaRef = ref<HTMLElement | null>(null)
const appVersion = ref('0.1.0')
const activeSection = ref('agent-config')
const saving = ref(false)
const showConfirmDialog = ref(false)
const showAgentDialog = ref(false)
const editingAgent = ref<Agent | null>(null)

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

// Fetch version on mount
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = '0.1.0'
  }
})
</script>

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
  width: 248px;
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