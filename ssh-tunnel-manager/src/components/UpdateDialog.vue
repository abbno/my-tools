<template>
  <t-dialog
    v-model:visible="visible"
    header="发现新版本"
    width="500px"
    :close-on-overlay-click="false"
    :on-close="handleClose"
  >
    <div class="update-dialog-content">
      <!-- 版本信息 -->
      <div class="version-info">
        <div class="version-row">
          <span class="label">当前版本：</span>
          <span class="value">{{ currentVersion }}</span>
        </div>
        <div class="version-row">
          <span class="label">最新版本：</span>
          <span class="value highlight">{{ updateInfo?.version }}</span>
        </div>
        <div class="version-row">
          <span class="label">发布日期：</span>
          <span class="value">{{ updateInfo?.releaseDate }}</span>
        </div>
      </div>

      <!-- 更新内容 -->
      <div class="changelog-section">
        <div class="changelog-title">更新内容：</div>
        <div class="changelog-list">
          <div
            v-for="(item, index) in updateInfo?.changelog"
            :key="index"
            class="changelog-item"
          >
            <span class="changelog-icon">{{ getChangelogIcon(item.type) }}</span>
            <span class="changelog-text">{{ item.description }}</span>
          </div>
        </div>
      </div>

      <!-- 下载进度 -->
      <div v-if="downloading" class="progress-section">
        <div class="progress-label">
          {{ downloadComplete ? '下载完成，正在安装...' : '下载进度：' }}
        </div>
        <t-progress
          :percentage="downloadProgress"
          :theme="downloadComplete ? 'success' : 'default'"
        />
        <div v-if="downloadComplete" class="install-tip">
          安装程序已启动，应用即将退出...
        </div>
      </div>

      <!-- 手动下载链接 -->
      <div v-if="showManualDownload" class="manual-download">
        <div class="manual-download-tip">
          如下载失败，可手动下载：
        </div>
        <a
          :href="updateInfo?.fullDownloadUrl"
          target="_blank"
          class="download-link"
        >
          {{ updateInfo?.fullDownloadUrl }}
        </a>
      </div>
    </div>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <t-button
          v-if="!downloading"
          variant="outline"
          @click="handleRemindLater"
        >
          稍后提醒
        </t-button>
        <t-button
          v-if="!downloading"
          theme="primary"
          @click="handleUpdateNow"
        >
          立即更新
        </t-button>
        <t-button
          v-if="downloading && !downloadComplete"
          theme="primary"
          loading
          disabled
        >
          下载中...
        </t-button>
        <t-button
          v-if="downloadComplete"
          theme="success"
          loading
          disabled
        >
          安装中...
        </t-button>
      </div>
    </template>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useUpdateStore } from '@/stores/update'
import type { ChangelogItem } from '@/types'

const updateStore = useUpdateStore()

// Props
interface Props {
  currentVersion?: string
}
withDefaults(defineProps<Props>(), {
  currentVersion: '0.1.0'
})

// 使用 store 的状态
const visible = computed({
  get: () => updateStore.showDialog,
  set: (val) => {
    if (!val) {
      updateStore.closeDialog()
    }
  }
})

const updateInfo = computed(() => updateStore.updateInfo)
const downloading = computed(() => updateStore.downloading)
const downloadProgress = computed(() => updateStore.downloadProgress)
const downloadComplete = computed(() => updateStore.downloadComplete)

// 是否显示手动下载链接（下载失败后显示）
const showManualDownload = ref(false)

// 监听下载进度，完成后重置手动下载提示
watch(downloadProgress, (progress) => {
  if (progress >= 100) {
    showManualDownload.value = false
  }
})

// 获取更新日志图标
function getChangelogIcon(type: ChangelogItem['type']): string {
  switch (type) {
    case 'feature':
      return '✨'
    case 'fix':
      return '🐛'
    case 'improve':
      return '⚡'
    default:
      return '•'
  }
}

// 关闭弹窗
function handleClose() {
  updateStore.closeDialog()
  showManualDownload.value = false
}

// 稍后提醒
function handleRemindLater() {
  updateStore.remindLater()
  showManualDownload.value = false
}

// 立即更新（下载并安装）
async function handleUpdateNow() {
  showManualDownload.value = false
  try {
    await updateStore.downloadAndInstall()
    // 应用将退出并重启
  } catch (error) {
    console.error('下载安装更新失败:', error)
    MessagePlugin.error('更新失败，请尝试手动下载')
    showManualDownload.value = true
  }
}
</script>

<style scoped>
.update-dialog-content {
  padding: 8px 0;
}

.version-info {
  margin-bottom: 16px;
}

.version-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.version-row .label {
  color: var(--td-text-color-secondary);
  min-width: 80px;
}

.version-row .value {
  color: var(--td-text-color-primary);
}

.version-row .value.highlight {
  color: var(--td-brand-color);
  font-weight: 500;
}

.changelog-section {
  margin-bottom: 16px;
}

.changelog-title {
  color: var(--td-text-color-secondary);
  margin-bottom: 8px;
}

.changelog-list {
  background: var(--td-bg-color-container);
  border-radius: 4px;
  padding: 12px;
  max-height: 150px;
  overflow-y: auto;
}

.changelog-item {
  display: flex;
  align-items: flex-start;
  margin-bottom: 6px;
}

.changelog-item:last-child {
  margin-bottom: 0;
}

.changelog-icon {
  margin-right: 8px;
  font-size: 14px;
}

.changelog-text {
  color: var(--td-text-color-primary);
  font-size: 14px;
  line-height: 1.5;
}

.progress-section {
  margin-bottom: 16px;
}

.progress-label {
  color: var(--td-text-color-secondary);
  margin-bottom: 8px;
}

.install-tip {
  margin-top: 8px;
  color: var(--td-success-color);
  font-size: 12px;
}

.manual-download {
  margin-top: 16px;
  padding: 12px;
  background: var(--td-bg-color-container-hover);
  border-radius: 4px;
}

.manual-download-tip {
  color: var(--td-text-color-secondary);
  margin-bottom: 8px;
  font-size: 12px;
}

.download-link {
  color: var(--td-brand-color);
  font-size: 12px;
  word-break: break-all;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>