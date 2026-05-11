<template>
  <div class="settings-container">
    <t-card title="设置" :bordered="false">
      <template #actions>
        <t-button variant="outline" @click="handleBack">
          返回主页
        </t-button>
      </template>

      <!-- Agent 配置区域 -->
      <div class="settings-section">
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
              :value="agent.enabled"
              @change="(value: boolean) => configStore.updateAgent(agent.id, value)"
            />
          </div>
        </div>
      </div>

      <!-- 同步设置区域 -->
      <div class="settings-section">
        <div class="section-title">同步设置</div>
        <div class="settings-row">
          <div class="settings-info">
            <span class="settings-label">自动同步</span>
            <span class="settings-desc">自动同步仓库</span>
          </div>
          <t-switch
            :value="configStore.config?.settings.auto_sync"
            @change="(value: boolean) => configStore.updateSettings({ auto_sync: value })"
          />
        </div>
        <div class="settings-row">
          <div class="settings-info">
            <span class="settings-label">默认同步间隔</span>
            <span class="settings-desc">自动同步的频率</span>
          </div>
          <t-select
            :value="configStore.config?.settings.default_sync_interval || 3600"
            @change="(value: number) => configStore.updateSettings({ default_sync_interval: value })"
            :options="syncIntervalOptions"
            style="width: 140px"
          />
        </div>
      </div>

      <!-- 版本信息区域 -->
      <div class="settings-section">
        <div class="section-title">版本信息</div>
        <div class="version-info">
          <div class="info-row">
            <span class="label">当前版本：</span>
            <span class="value">0.1.0</span>
          </div>
        </div>
      </div>

      <!-- 关于区域 -->
      <div class="settings-section about-section">
        <div class="about-brand">
          <span class="about-icon">◈</span>
          <span class="about-name">Skills Manager</span>
        </div>
        <p class="about-desc">AI Agent 的知识管理系统</p>
      </div>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'

const router = useRouter()
const configStore = useConfigStore()

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

function handleBack() {
  router.push('/')
}
</script>

<style scoped>
.settings-container {
  height: 100vh;
  padding: 24px;
  background: var(--td-bg-color-container);
  overflow-y: auto;
}

.settings-container :deep(.t-card) {
  max-width: 800px;
  margin: 0 auto;
  height: auto;
  overflow: visible;
}

.settings-container :deep(.t-card__body) {
  padding: 24px;
  overflow: visible;
}

.section-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--td-component-border);
}

.settings-section {
  margin-bottom: 32px;
}

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
  background: var(--td-bg-color-secondarycontainer);
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
  background: var(--td-bg-color-specialcomponent);
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-block;
  margin-top: 4px;
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-secondarycontainer);
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

.version-info {
  margin-bottom: 16px;
}

.info-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.info-row .label {
  color: var(--td-text-color-secondary);
  min-width: 80px;
}

.info-row .value {
  color: var(--td-text-color-primary);
}

.empty-tip {
  color: var(--td-text-color-placeholder);
  font-size: 14px;
  padding: 16px 0;
}

.about-section {
  text-align: center;
  padding: 32px 0;
}

.about-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 8px;
}

.about-icon {
  color: var(--td-brand-color);
  font-size: 24px;
}

.about-name {
  font-size: 18px;
  font-weight: 700;
  color: var(--td-text-color-primary);
}

.about-desc {
  font-size: 14px;
  color: var(--td-text-color-secondary);
  margin: 0;
}
</style>