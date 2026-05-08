<template>
  <t-dialog
    v-model:visible="dialogVisible"
    :header="dialogTitle"
    width="640px"
    :footer="false"
    :destroy-on-close="true"
    @close="handleClose"
  >
    <!-- 工具栏 -->
    <div class="log-toolbar">
      <t-button
        theme="danger"
        variant="outline"
        size="small"
        :disabled="logs.length === 0"
        @click="handleClearLogs"
      >
        <template #icon>
          <t-icon name="delete" />
        </template>
        清除日志
      </t-button>
    </div>

    <!-- 日志列表 -->
    <div class="log-container">
      <!-- 加载状态 -->
      <t-loading v-if="loading" text="加载中..." />

      <!-- 空状态 -->
      <t-empty
        v-else-if="logs.length === 0"
        description="暂无日志记录"
      />

      <!-- 日志项列表 -->
      <div v-else class="log-list">
        <div
          v-for="log in logs"
          :key="log.id"
          class="log-item"
        >
          <!-- 操作类型标签 -->
          <t-tag
            :theme="getActionTheme(log.action)"
            variant="light"
            size="small"
          >
            {{ getActionLabel(log.action) }}
          </t-tag>

          <!-- 时间戳 -->
          <span class="log-time">{{ formatDateTime(log.createdAt) }}</span>

          <!-- 日志消息 -->
          <span :class="['log-message', { 'error-message': log.action === 'error' }]">
            {{ log.message }}
          </span>
        </div>
      </div>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { ConnectionLog, LogAction } from '@/types'
import * as api from '@/api/tauri'
import { formatDateTime } from '@/utils/format'

// Props
const props = defineProps<{
  visible: boolean
  configId: string
  configName: string
}>()

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

// 状态
const logs = ref<ConnectionLog[]>([])
const loading = ref(false)

// 计算属性
const dialogVisible = computed({
  get: () => props.visible,
  set: (value) => emit('update:visible', value)
})

const dialogTitle = computed(() => `${props.configName} - 连接日志`)

// 监听对话框显示状态，自动加载日志
watch(() => props.visible, (newVisible) => {
  if (newVisible && props.configId) {
    loadLogs()
  }
})

// 方法

/**
 * 加载日志
 */
async function loadLogs() {
  loading.value = true
  try {
    logs.value = await api.getLogs(props.configId)
  } catch (error) {
    console.error('加载日志失败:', error)
    logs.value = []
  } finally {
    loading.value = false
  }
}

/**
 * 清除日志
 */
async function handleClearLogs() {
  try {
    await api.clearLogs(props.configId)
    logs.value = []
  } catch (error) {
    console.error('清除日志失败:', error)
  }
}

/**
 * 关闭对话框
 */
function handleClose() {
  emit('update:visible', false)
}

/**
 * 获取操作类型标签显示文本
 */
function getActionLabel(action: LogAction): string {
  const labels: Record<LogAction, string> = {
    connect: '连接',
    disconnect: '断开',
    reconnect: '重连',
    error: '错误'
  }
  return labels[action]
}

/**
 * 获取操作类型标签主题色
 */
function getActionTheme(action: LogAction): 'success' | 'warning' | 'danger' | 'primary' {
  const themes: Record<LogAction, 'success' | 'warning' | 'danger' | 'primary'> = {
    connect: 'success',
    disconnect: 'warning',
    reconnect: 'primary',
    error: 'danger'
  }
  return themes[action]
}
</script>

<style scoped>
.log-toolbar {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 16px;
}

.log-container {
  min-height: 200px;
  max-height: 400px;
  overflow-y: auto;
}

.log-list {
  display: flex;
  flex-direction: column;
}

.log-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid #eee;
}

.log-item:last-child {
  border-bottom: none;
}

.log-time {
  flex-shrink: 0;
  font-size: 12px;
  color: #999;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.log-message {
  flex: 1;
  font-size: 13px;
  color: #333;
  word-break: break-all;
}

.log-message.error-message {
  color: #e34d59;
}
</style>
