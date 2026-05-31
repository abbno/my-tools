<template>
  <t-config-provider :global-config="globalConfig">
    <router-view />
    <UpdateDialog :current-version="currentVersion" />
    <KeySetupDialog
      :visible="keySetupStore.showDialog"
      @update:visible="keySetupStore.showDialog = $event"
      @complete="handleKeySetupComplete"
    />
  </t-config-provider>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useUpdateStore } from '@/stores/update'
import { useKeySetupStore } from '@/stores/keySetup'
import { useConfigStore } from '@/stores/config'
import UpdateDialog from '@/components/UpdateDialog.vue'
import KeySetupDialog from '@/components/KeySetupDialog.vue'
import { getVersion } from '@/api/tauri'
import { MessagePlugin } from 'tdesign-vue-next'
import type { CreateConfigRequest, UpdateConfigRequest } from '@/types'

// TDesign 全局配置
const globalConfig = {
  classPrefix: 't'
}

// 当前版本号（从后端获取）
const currentVersion = ref('')

// Stores
const updateStore = useUpdateStore()
const keySetupStore = useKeySetupStore()
const configStore = useConfigStore()

// 定时检查间隔（4小时）
const CHECK_INTERVAL = 4 * 60 * 60 * 1000
let checkTimer: number | null = null

// 处理密钥设置完成
async function handleKeySetupComplete(data: { success: boolean; configData?: CreateConfigRequest | UpdateConfigRequest }) {
  if (!data.success || !data.configData) {
    return
  }

  try {
    if (keySetupStore.editMode && keySetupStore.editConfigId) {
      // 编辑模式：更新配置
      await configStore.updateConfig(data.configData as UpdateConfigRequest)
      MessagePlugin.success('密钥设置成功，配置已更新')
    } else {
      // 新建模式：创建配置
      await configStore.createConfig(data.configData as CreateConfigRequest)
      MessagePlugin.success('密钥设置成功，配置已创建')
    }

    // 刷新配置列表
    await configStore.fetchConfigs()
  } catch (error) {
    console.error('保存配置失败:', error)
    MessagePlugin.error('保存配置失败')
  }
}

// 组件挂载时初始化
onMounted(async () => {
  // 获取版本号
  try {
    currentVersion.value = await getVersion()
  } catch (error) {
    console.error('获取版本号失败:', error)
  }

  // 初始化更新 store
  await updateStore.init()

  // 启动时检查更新（延迟 3 秒，避免影响启动速度）
  setTimeout(() => {
    updateStore.checkUpdate()
  }, 3000)

  // 设置定时检查
  checkTimer = window.setInterval(() => {
    updateStore.checkUpdate()
  }, CHECK_INTERVAL)
})

// 组件卸载时清理
onUnmounted(() => {
  // 清理定时器
  if (checkTimer) {
    clearInterval(checkTimer)
    checkTimer = null
  }

  // 清理 store
  updateStore.cleanup()
})
</script>

<style>
/* 全局样式重置 */
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 禁止选中文本 */
body {
  user-select: none;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--td-scrollbar-color, #dcdcdc);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--td-scrollbar-hover-color, #bbb);
}
</style>