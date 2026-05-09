<template>
  <t-dialog
    v-model:visible="visible"
    header="Settings"
    :width="600"
    :footer="false"
    destroy-on-close
  >
    <div class="settings-content">
      <!-- Agent Configuration Section -->
      <section class="settings-section">
        <h3 class="section-title">Agent Configuration</h3>
        <t-divider />

        <div v-if="!configStore.config?.agents?.length" class="empty-state">
          <p>No agents configured</p>
        </div>

        <div v-else class="agent-list">
          <div
            v-for="agent in configStore.config?.agents"
            :key="agent.id"
            class="agent-item"
          >
            <div class="agent-info">
              <span class="agent-name">{{ agent.name }}</span>
              <code class="agent-path">{{ agent.path }}</code>
            </div>
            <t-switch
              :value="agent.enabled"
              @change="(value: boolean) => configStore.updateAgent(agent.id, value)"
            />
          </div>
        </div>
      </section>

      <t-divider />

      <!-- Sync Settings Section -->
      <section class="settings-section">
        <h3 class="section-title">Sync Settings</h3>
        <t-divider />

        <div class="settings-form">
          <!-- Auto Sync -->
          <div class="form-row">
            <div class="form-info">
              <span class="form-label">Auto Sync</span>
              <span class="form-desc">Automatically synchronize repositories</span>
            </div>
            <t-switch
              :value="configStore.config?.settings.auto_sync"
              @change="(value: boolean) => configStore.updateSettings({ auto_sync: value })"
            />
          </div>

          <!-- Sync Interval -->
          <div class="form-row">
            <div class="form-info">
              <span class="form-label">Default Sync Interval</span>
              <span class="form-desc">Frequency for automatic synchronization</span>
            </div>
            <t-select
              :value="configStore.config?.settings.default_sync_interval || 3600"
              @change="(value: number) => configStore.updateSettings({ default_sync_interval: value })"
              :options="syncIntervalOptions"
              style="width: 140px"
            />
          </div>
        </div>
      </section>

      <t-divider />

      <!-- About Section -->
      <section class="settings-section about-section">
        <div class="about-brand">
          <img src="/app-icon.svg" alt="Skills Manager" class="about-icon" />
          <span class="about-name">Skills Manager</span>
        </div>
        <p class="about-version">Version 0.1.0</p>
        <p class="about-desc">A curated knowledge management system for AI agents</p>
      </section>

      <!-- Footer -->
      <div class="dialog-footer">
        <t-button theme="primary" @click="visible = false">Close</t-button>
      </div>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()
const visible = defineModel<boolean>('visible', { default: false })

const syncIntervalOptions = [
  { label: '5 minutes', value: 300 },
  { label: '15 minutes', value: 900 },
  { label: '30 minutes', value: 1800 },
  { label: '1 hour', value: 3600 },
  { label: '2 hours', value: 7200 },
  { label: '6 hours', value: 21600 },
  { label: '12 hours', value: 43200 },
  { label: 'Daily', value: 86400 },
]
</script>

<style scoped>
.settings-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.settings-section {
  padding: var(--space-md) 0;
}

.section-title {
  font-family: var(--font-body);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.empty-state {
  padding: var(--space-md);
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.agent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  display: block;
}

.agent-path {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg-elevated);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  display: inline-block;
  margin-top: 4px;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.form-info {
  flex: 1;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  display: block;
}

.form-desc {
  font-size: 12px;
  color: var(--text-muted);
  display: block;
  margin-top: 2px;
}

.about-section {
  text-align: center;
  padding: var(--space-lg) 0;
}

.about-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  margin-bottom: var(--space-sm);
}

.about-icon {
  width: 32px;
  height: 32px;
}

.about-name {
  font-family: var(--font-display);
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}

.about-version {
  font-size: 11px;
  color: var(--text-muted);
  margin: 0 0 var(--space-sm) 0;
}

.about-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding-top: var(--space-md);
}
</style>