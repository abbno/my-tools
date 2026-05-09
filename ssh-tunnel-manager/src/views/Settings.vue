<template>
  <div class="settings-container">
    <t-card title="设置" :bordered="false">
      <template #actions>
        <t-button
          variant="outline"
          style="margin-right: 8px"
          @click="handleBack"
        >
          返回主页
        </t-button>
        <t-button
          theme="primary"
          :disabled="!hasChanges"
          :loading="saving"
          @click="handleSave"
        >
          保存
        </t-button>
      </template>

      <!-- 版本信息区域 -->
      <div class="version-section">
        <div class="section-title">版本信息</div>
        <div class="version-info">
          <div class="info-row">
            <span class="label">当前版本：</span>
            <span class="value">{{ currentVersion }}</span>
          </div>
          <div class="info-row">
            <span class="label">上次检查：</span>
            <span class="value">{{ formatLastCheckTime }}</span>
          </div>
        </div>
        <div class="check-update">
          <t-button
            theme="primary"
            :loading="checking"
            @click="handleCheckUpdate"
          >
            {{ checking ? '检查中...' : '检查更新' }}
          </t-button>
        </div>
      </div>

      <!-- 更新服务器配置区域 -->
      <div class="update-server-section">
        <div class="section-title">更新服务器配置</div>
        <div class="config-row">
          <span class="label">服务器地址：</span>
          <div class="input-wrapper">
            <t-input
              v-model="updateServerUrl"
              placeholder="如: https://updates.example.com"
              :disabled="saving"
              clearable
            />
          </div>
        </div>
        <div class="config-tip">
          提示：配置后将自动检查更新
        </div>
      </div>

      <!-- 其他设置区域（预留） -->
      <div class="other-section">
        <div class="section-title">其他设置</div>
        <div class="empty-tip">
          更多设置功能正在开发中...
        </div>
      </div>
    </t-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { MessagePlugin } from 'tdesign-vue-next'
import { useUpdateStore } from '@/stores/update'
import { getAppSetting, saveAppSetting, deleteAppSetting, getVersion } from '@/api/tauri'

const router = useRouter()
const updateStore = useUpdateStore()

// 当前版本号（从后端获取）
const currentVersion = ref('')

// 更新服务器配置
const updateServerUrl = ref('')
const originalUpdateServerUrl = ref('')
const saving = ref(false)

// 检查中状态
const checking = computed(() => updateStore.checking)

// 是否有变更
const hasChanges = computed(() => updateServerUrl.value !== originalUpdateServerUrl.value)

// 格式化上次检查时间
const formatLastCheckTime = computed(() => {
  const time = updateStore.lastCheckTime
  if (!time) {
    return '从未检查'
  }
  const date = new Date(time)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
})

// 检查更新
async function handleCheckUpdate() {
  try {
    await updateStore.checkUpdate()
    if (!updateStore.updateInfo) {
      MessagePlugin.success('当前已是最新版本')
    }
  } catch (error) {
    const errorMsg = String(error)
    if (errorMsg.includes('请先配置更新服务器地址')) {
      MessagePlugin.warning('请先配置更新服务器地址')
    } else {
      MessagePlugin.error('检查更新失败：' + errorMsg)
    }
  }
}

// 保存设置
async function handleSave() {
  saving.value = true
  try {
    if (updateServerUrl.value.trim()) {
      await saveAppSetting('update_server_url', updateServerUrl.value.trim())
      originalUpdateServerUrl.value = updateServerUrl.value.trim()
      MessagePlugin.success('保存成功')
      // 触发更新检查
      try {
        await updateStore.checkUpdate()
        if (!updateStore.updateInfo) {
          MessagePlugin.success('当前已是最新版本')
        }
      } catch (checkError) {
        const errorMsg = String(checkError)
        if (!errorMsg.includes('请先配置更新服务器地址')) {
          MessagePlugin.warning('检查更新失败：' + errorMsg)
        }
      }
    } else {
      // 如果清空了 URL，则删除设置
      await deleteAppSetting('update_server_url')
      originalUpdateServerUrl.value = ''
      MessagePlugin.success('设置已清除')
    }
  } catch (error) {
    console.error('保存设置失败:', error)
    MessagePlugin.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}

// 返回主页
function handleBack() {
  router.push('/')
}

// 初始化
onMounted(async () => {
  // 从后端获取版本号
  try {
    currentVersion.value = await getVersion()
  } catch (error) {
    console.error('获取版本号失败:', error)
  }

  // 加载更新服务器配置
  try {
    const url = await getAppSetting('update_server_url')
    if (url) {
      updateServerUrl.value = url
      originalUpdateServerUrl.value = url
    }
  } catch (error) {
    console.error('加载更新服务器配置失败:', error)
  }
})
</script>

<style scoped>
.settings-container {
  height: 100vh;
  padding: 24px;
  background: var(--td-bg-color-container);
}

.settings-container :deep(.t-card) {
  height: 100%;
}

.settings-container :deep(.t-card__body) {
  padding: 24px;
  overflow-y: auto;
}

.section-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--td-text-color-primary);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--td-component-border);
}

.version-section {
  margin-bottom: 32px;
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

.check-update {
  margin-top: 16px;
}

.update-server-section {
  margin-bottom: 32px;
}

.config-row {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.config-row .label {
  color: var(--td-text-color-secondary);
  min-width: 80px;
  flex-shrink: 0;
}

.config-row .input-wrapper {
  flex: 1;
  max-width: 400px;
}

.config-tip {
  color: var(--td-text-color-placeholder);
  font-size: 12px;
}

.other-section {
  margin-bottom: 32px;
}

.empty-tip {
  color: var(--td-text-color-placeholder);
  font-size: 14px;
}
</style>