<template>
  <div class="settings-container">
    <t-card title="设置">
      <t-space direction="vertical" size="large">
        <t-divider>Agent 配置</t-divider>
        <t-list>
          <t-list-item v-for="agent in configStore.config?.agents" :key="agent.id">
            <t-list-item-meta :title="agent.name" :description="agent.path" />
            <template #action>
              <t-switch
                :value="agent.enabled"
                @change="(val: boolean) => configStore.updateAgent(agent.id, val)"
              />
            </template>
          </t-list-item>
        </t-list>

        <t-divider>同步设置</t-divider>
        <t-form>
          <t-form-item label="自动同步">
            <t-switch
              :value="configStore.config?.settings.auto_sync"
              @change="onAutoSyncChange"
            />
          </t-form-item>
          <t-form-item label="默认同步间隔">
            <t-select
              :value="String(configStore.config?.settings.default_sync_interval || 3600)"
              @change="onSyncIntervalChange"
              style="width: 200px"
            >
              <t-option value="300" label="5 分钟" />
              <t-option value="900" label="15 分钟" />
              <t-option value="1800" label="30 分钟" />
              <t-option value="3600" label="1 小时" />
              <t-option value="7200" label="2 小时" />
              <t-option value="21600" label="6 小时" />
              <t-option value="43200" label="12 小时" />
              <t-option value="86400" label="每天" />
            </t-select>
          </t-form-item>
        </t-form>

        <t-divider />
        <t-button @click="goBack">返回</t-button>
      </t-space>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useConfigStore } from '@/stores/config'

const router = useRouter()
const configStore = useConfigStore()

function onSyncIntervalChange(value: string) {
  configStore.updateSettings({ default_sync_interval: Number(value) })
}

function onAutoSyncChange(value: boolean) {
  configStore.updateSettings({ auto_sync: value })
}

function goBack() {
  router.push('/')
}

onMounted(() => {
  if (!configStore.config) {
    configStore.loadConfig()
  }
})
</script>

<style scoped>
.settings-container {
  padding: 24px;
  height: 100vh;
  overflow-y: auto;
}
</style>