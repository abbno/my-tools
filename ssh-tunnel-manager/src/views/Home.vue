<template>
  <div class="home-container">
    <t-layout class="main-layout">
      <!-- 左侧侧边栏 -->
      <t-aside width="200px" class="sidebar-aside">
        <Sidebar />
      </t-aside>

      <!-- 右侧主内容区 -->
      <t-content class="main-content">
        <TunnelList
          @create="handleCreate"
          @edit="handleEdit"
          @import="handleImport"
          @export="handleExport"
          @delete="handleDelete"
          @showLog="handleShowLog"
        />
      </t-content>
    </t-layout>

    <!-- 配置表单抽屉 -->
    <ConfigForm
      v-model:visible="showConfigForm"
      :config="editingConfig"
      :default-group-id="currentGroupId"
      @saved="handleConfigSaved"
    />

    <!-- 日志面板对话框 -->
    <LogPanel
      v-model:visible="showLogPanel"
      :config-id="logConfigId"
      :config-name="logConfigName"
    />

    <!-- 导入配置对话框 -->
    <t-dialog
      v-model:visible="showImportDialog"
      header="导入配置"
      width="500px"
      :confirm-btn="{ content: '导入', loading: importing }"
      :on-confirm="handleImportConfirm"
      :on-close="handleImportClose"
    >
      <div class="import-content">
        <t-textarea
          v-model="importJson"
          placeholder="请粘贴导出的配置 JSON 数据..."
          :autosize="{ minRows: 8, maxRows: 16 }"
        />
        <p class="import-tip">
          提示：导入的配置将添加到现有配置列表中，不会覆盖已有配置。
        </p>
      </div>
    </t-dialog>

    <!-- 删除确认对话框 -->
    <t-dialog
      v-model:visible="showDeleteDialog"
      header="确认删除"
      width="400px"
      :confirm-btn="{ content: '删除', theme: 'danger', loading: deleting }"
      :on-confirm="handleDeleteConfirm"
      :on-close="handleDeleteClose"
    >
      <p class="delete-tip">
        确定要删除配置「{{ deletingConfigName }}」吗？此操作不可恢复。
      </p>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import type { Config } from '@/types'
import { useGroupStore } from '@/stores/group'
import { useConfigStore } from '@/stores/config'
import { useTunnelStore } from '@/stores/tunnel'
import { useKeySetupStore } from '@/stores/keySetup'
import * as api from '@/api/tauri'

// 组件
import Sidebar from '@/components/Sidebar.vue'
import TunnelList from '@/components/TunnelList.vue'
import ConfigForm from '@/components/ConfigForm.vue'
import LogPanel from '@/components/LogPanel.vue'

// Stores
const groupStore = useGroupStore()
const configStore = useConfigStore()
const tunnelStore = useTunnelStore()
const keySetupStore = useKeySetupStore()

// 配置表单状态
const showConfigForm = ref(false)
const editingConfig = ref<Config | null>(null)

// 密钥设置完成后关闭配置表单
watch(() => keySetupStore.signalCloseForm, (val) => {
  if (val) {
    showConfigForm.value = false
    keySetupStore.signalCloseForm = false
  }
})

// 当前选中的分组ID（用于新建配置时的默认分组）
const currentGroupId = computed(() => configStore.selectedGroupId)

// 日志面板状态
const showLogPanel = ref(false)
const logConfigId = ref('')
const logConfigName = ref('')

// 导入配置状态
const showImportDialog = ref(false)
const importJson = ref('')
const importing = ref(false)

// 删除确认状态
const showDeleteDialog = ref(false)
const deletingConfigId = ref('')
const deletingConfigName = ref('')
const deleting = ref(false)

// 组件挂载时加载分组列表
onMounted(async () => {
  try {
    await groupStore.fetchGroups()
  } catch (error) {
    console.error('加载分组失败:', error)
    MessagePlugin.error('加载分组失败')
  }
})

// 新建配置
function handleCreate() {
  editingConfig.value = null
  showConfigForm.value = true
}

// 编辑配置
function handleEdit(config: Config) {
  console.log('Home handleEdit config:', config)
  editingConfig.value = config
  console.log('Home editingConfig set to:', editingConfig.value)
  showConfigForm.value = true
}

// 配置保存完成
function handleConfigSaved(config: Config) {
  console.log('配置已保存:', config.id)
}

// 导入配置
function handleImport() {
  importJson.value = ''
  showImportDialog.value = true
}

// 确认导入
async function handleImportConfirm(): Promise<boolean> {
  if (!importJson.value.trim()) {
    MessagePlugin.warning('请输入配置数据')
    return false
  }

  importing.value = true
  try {
    const count = await api.importConfigs(importJson.value.trim())
    MessagePlugin.success(`成功导入 ${count} 个配置`)
    // 刷新配置列表
    await configStore.fetchConfigs()
    importJson.value = ''
    showImportDialog.value = false
    return true
  } catch (error) {
    console.error('导入配置失败:', error)
    MessagePlugin.error('导入配置失败，请检查数据格式')
    return false
  } finally {
    importing.value = false
  }
}

// 关闭导入对话框
function handleImportClose() {
  importJson.value = ''
}

// 导出配置
async function handleExport() {
  try {
    const json = await api.exportConfigs()
    // 复制到剪贴板
    await navigator.clipboard.writeText(json)
    MessagePlugin.success('配置已导出并复制到剪贴板')
  } catch (error) {
    console.error('导出配置失败:', error)
    MessagePlugin.error('导出配置失败')
  }
}

// 删除配置
function handleDelete(configId: string) {
  const config = configStore.configs.find(c => c.id === configId)
  if (config) {
    deletingConfigId.value = configId
    deletingConfigName.value = config.name
    showDeleteDialog.value = true
  }
}

// 确认删除
async function handleDeleteConfirm(): Promise<boolean> {
  deleting.value = true
  try {
    // 如果隧道正在运行，先停止
    const tunnelInfo = tunnelStore.getStatus(deletingConfigId.value)
    if (tunnelInfo && tunnelInfo.status === 'running') {
      await tunnelStore.stopTunnel(deletingConfigId.value)
    }
    // 删除配置
    await configStore.removeConfig(deletingConfigId.value)
    MessagePlugin.success('配置已删除')
    showDeleteDialog.value = false
    return true
  } catch (error) {
    console.error('删除配置失败:', error)
    MessagePlugin.error('删除配置失败')
    return false
  } finally {
    deleting.value = false
  }
}

// 关闭删除对话框
function handleDeleteClose() {
  deletingConfigId.value = ''
  deletingConfigName.value = ''
}

// 显示日志面板
function handleShowLog(configId: string) {
  const config = configStore.configs.find(c => c.id === configId)
  if (config) {
    logConfigId.value = configId
    logConfigName.value = config.name
    showLogPanel.value = true
  }
}
</script>

<style scoped>
.home-container {
  height: 100vh;
  overflow: hidden;
}

.main-layout {
  height: 100%;
}

.sidebar-aside {
  border-right: 1px solid var(--td-component-border);
}

.main-content {
  height: 100%;
  overflow: hidden;
}

/* 导入对话框样式 */
.import-content {
  padding: 8px 0;
}

.import-tip {
  margin-top: 12px;
  font-size: 12px;
  color: var(--td-text-color-placeholder);
}

/* 删除对话框样式 */
.delete-tip {
  font-size: 14px;
  color: var(--td-text-color-primary);
  line-height: 1.6;
}
</style>