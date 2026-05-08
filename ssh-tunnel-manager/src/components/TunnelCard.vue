<template>
  <div :class="['tunnel-card', `status-${statusClass}`]">
    <!-- 状态指示器 -->
    <div :class="['status-indicator', { pulsing: isPulsing }]">
      <span class="status-dot"></span>
    </div>

    <!-- 卡片内容 -->
    <div class="card-content">
      <!-- 头部：类型标签 + 名称 -->
      <div class="card-header">
        <t-tag :theme="tunnelTypeTheme" variant="light" size="small">
          {{ tunnelTypeLabel }}
        </t-tag>
        <span class="config-name">{{ config.name }}</span>

        <!-- 开机启动开关 -->
        <t-tooltip :content="config.autoStart ? '关闭开机启动' : '开启开机启动'" placement="top">
          <t-button
            :class="['autostart-btn', { 'is-autostart': config.autoStart }]"
            variant="text"
            shape="circle"
            size="medium"
            @click.stop="handleToggleAutoStart"
          >
            <template #icon>
              <RocketIcon />
            </template>
          </t-button>
        </t-tooltip>

        <!-- 收藏按钮 -->
        <t-tooltip :content="config.isFavorite ? '取消收藏' : '添加收藏'" placement="top">
          <t-button
            :class="['favorite-btn', { 'is-favorite': config.isFavorite }]"
            variant="text"
            shape="circle"
            size="medium"
            @click.stop="handleToggleFavorite"
          >
            <template #icon>
              <StarFilledIcon v-if="config.isFavorite" />
              <StarIcon v-else />
            </template>
          </t-button>
        </t-tooltip>
      </div>

      <!-- SSH 命令 -->
      <div class="ssh-command">
        <t-tooltip :content="copyTooltip" placement="top">
          <code class="command-text" @click="handleCopyCommand">
            {{ sshCommand }}
          </code>
        </t-tooltip>
      </div>

      <!-- 状态信息 -->
      <div class="status-info">
        <span :class="['status-text', `text-${statusClass}`]">
          {{ statusLabel }}
        </span>
        <span v-if="tunnelInfo?.pid" class="pid-info">PID: {{ tunnelInfo.pid }}</span>
        <span v-if="tunnelInfo?.errorMessage" class="error-message">
          {{ tunnelInfo.errorMessage }}
        </span>
      </div>

      <!-- 操作按钮 -->
      <div class="card-actions">
        <!-- 启动/停止按钮 -->
        <t-button
          v-if="canToggle"
          :theme="isRunning ? 'warning' : 'success'"
          :variant="'base'"
          :loading="isLoading"
          size="small"
          @click="handleToggle"
        >
          <template #icon>
            <StopCircleIcon v-if="isRunning" />
            <PlayCircleIcon v-else />
          </template>
          {{ isRunning ? '停止' : '启动' }}
        </t-button>

        <!-- 编辑按钮（仅非运行状态显示） -->
        <t-button
          v-if="!isRunning"
          theme="default"
          variant="base"
          size="small"
          @click="handleEdit"
        >
          <template #icon>
            <EditIcon />
          </template>
          编辑
        </t-button>

        <!-- 日志按钮 -->
        <t-button
          theme="default"
          variant="base"
          size="small"
          @click="handleShowLog"
        >
          <template #icon>
            <FileIcon />
          </template>
          日志
        </t-button>

        <!-- 删除按钮（仅非运行状态显示） -->
        <t-popconfirm
          v-if="!isRunning"
          content="确定要删除此隧道配置吗？"
          placement="top"
          @confirm="handleDelete"
        >
          <t-button
            theme="danger"
            variant="base"
            size="small"
          >
            <template #icon>
              <DeleteIcon />
            </template>
            删除
          </t-button>
        </t-popconfirm>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { PlayCircleIcon, StopCircleIcon, EditIcon, DeleteIcon, FileIcon, StarFilledIcon, StarIcon, RocketIcon } from 'tdesign-icons-vue-next'
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next'
import type { Config, TunnelInfo, TunnelStatus } from '@/types'
import { useTunnelStore } from '@/stores/tunnel'
import { useConfigStore } from '@/stores/config'
import {
  formatSshCommand,
  getTunnelTypeLabel,
  getTunnelTypeTheme,
  copyToClipboard
} from '@/utils/format'

// Props
const props = defineProps<{
  config: Config
}>()

// Emits
const emit = defineEmits<{
  edit: [config: Config]
  delete: [configId: string]
  showLog: [configId: string]
}>()

// Store
const tunnelStore = useTunnelStore()
const configStore = useConfigStore()

// 复制提示状态
const copyTooltip = ref('点击复制')

// 计算属性
const tunnelInfo = computed<TunnelInfo | undefined>(() => {
  return tunnelStore.getStatus(props.config.id)
})

const status = computed<TunnelStatus>(() => {
  return tunnelInfo.value?.status ?? 'stopped'
})

const isLoading = computed(() => {
  return tunnelStore.isLoading(props.config.id) || tunnelStore.isPrechecking(props.config.id)
})

const isPrechecking = computed(() => {
  return tunnelStore.isPrechecking(props.config.id)
})

const isRunning = computed(() => {
  return status.value === 'running'
})

const canToggle = computed(() => {
  return !['starting', 'stopping'].includes(status.value)
})

const isPulsing = computed(() => {
  return status.value === 'running' || status.value === 'reconnecting'
})

const sshCommand = computed(() => {
  return formatSshCommand(props.config)
})

const tunnelTypeLabel = computed(() => {
  return getTunnelTypeLabel(props.config.tunnelType)
})

const tunnelTypeTheme = computed(() => {
  return getTunnelTypeTheme(props.config.tunnelType)
})

// 状态样式类
const statusClass = computed(() => {
  switch (status.value) {
    case 'running':
      return 'running'
    case 'starting':
    case 'stopping':
      return 'loading'
    case 'error':
      return 'error'
    case 'reconnecting':
      return 'reconnecting'
    default:
      return 'stopped'
  }
})

// 状态显示文本
const statusLabel = computed(() => {
  // 如果正在预检查，显示检查中
  if (isPrechecking.value) {
    return '检查中...'
  }

  const labels: Record<TunnelStatus, string> = {
    stopped: '已停止',
    starting: '启动中...',
    running: '运行中',
    stopping: '停止中...',
    error: '错误',
    reconnecting: '重连中...'
  }
  return labels[status.value]
})

// 方法
async function handleCopyCommand() {
  const success = await copyToClipboard(sshCommand.value)
  if (success) {
    copyTooltip.value = '已复制!'
    setTimeout(() => {
      copyTooltip.value = '点击复制'
    }, 2000)
  }
}

async function handleToggle() {
  if (isRunning.value) {
    await tunnelStore.stopTunnel(props.config.id)
  } else {
    // 先进行预检查
    try {
      const precheckResult = await tunnelStore.precheckTunnel(props.config.id)

      // 检查远程连通性
      if (!precheckResult.remoteOk) {
        MessagePlugin.error({
          content: precheckResult.remoteError || '远程主机连接失败',
          duration: 3000
        })
        return
      }

      // 检查本地端口
      if (!precheckResult.localPortOk) {
        // 如果有进程信息，显示详细弹框
        if (precheckResult.portProcessInfo) {
          const dialog = DialogPlugin.alert({
            header: '端口已被占用',
            body: `本地端口 ${props.config.localHost}:${props.config.localPort} 已被占用\n\n占用进程：${precheckResult.portProcessInfo.name}\n进程 PID：${precheckResult.portProcessInfo.pid}`,
            theme: 'warning',
            confirmBtn: '知道了',
            onConfirm: () => {
              dialog.hide()
            }
          })
        } else {
          MessagePlugin.error({
            content: precheckResult.localPortError || '本地端口已被占用',
            duration: 3000
          })
        }
        return
      }

      // 预检查通过，启动隧道
      await tunnelStore.startTunnel(props.config.id)
    } catch (error) {
      MessagePlugin.error({
        content: String(error),
        duration: 3000
      })
    }
  }
}

function handleEdit() {
  emit('edit', props.config)
}

function handleShowLog() {
  emit('showLog', props.config.id)
}

function handleDelete() {
  emit('delete', props.config.id)
}

async function handleToggleFavorite() {
  try {
    await configStore.setFavorite(props.config.id, !props.config.isFavorite)
  } catch (error) {
    console.error('切换收藏状态失败:', error)
  }
}

async function handleToggleAutoStart() {
  try {
    await configStore.setAutoStart(props.config.id, !props.config.autoStart)
  } catch (error) {
    console.error('切换开机启动状态失败:', error)
  }
}
</script>

<style scoped>
.tunnel-card {
  position: relative;
  background: #fff;
  border: 1px solid #e7e7e7;
  border-radius: 6px;
  padding: 16px;
  margin-bottom: 12px;
  transition: box-shadow 0.2s ease;
  overflow: hidden;
}

.tunnel-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

/* 状态左边框 */
.tunnel-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: #bbb;
  transition: background 0.3s ease;
}

.tunnel-card.status-running::before {
  background: #00a870;
}

.tunnel-card.status-stopped::before {
  background: #bbb;
}

.tunnel-card.status-error::before {
  background: #e34d59;
}

.tunnel-card.status-reconnecting::before {
  background: #ed7b2f;
}

.tunnel-card.status-loading::before {
  background: #0052d9;
}

/* 状态指示器 */
.status-indicator {
  position: absolute;
  left: 12px;
  top: 12px;
  width: 8px;
  height: 8px;
}

.status-dot {
  display: block;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: currentColor;
}

/* 脉冲动画 */
.status-indicator.pulsing .status-dot {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(1.2);
  }
}

.tunnel-card.status-running .status-indicator {
  color: #00a870;
}

.tunnel-card.status-reconnecting .status-indicator {
  color: #ed7b2f;
}

.tunnel-card.status-error .status-indicator {
  color: #e34d59;
}

.tunnel-card.status-stopped .status-indicator {
  color: #bbb;
}

.tunnel-card.status-loading .status-indicator {
  color: #0052d9;
}

/* 卡片内容 */
.card-content {
  margin-left: 8px;
}

/* 头部 */
.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.config-name {
  flex: 1;
  font-weight: 500;
  font-size: 14px;
  color: #333;
}

/* SSH 命令 */
.ssh-command {
  margin-bottom: 8px;
}

.command-text {
  display: block;
  padding: 8px 12px;
  background: #f5f5f5;
  border-radius: 4px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #666;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: background 0.2s ease;
}

.command-text:hover {
  background: #eee;
}

/* 状态信息 */
.status-info {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  font-size: 12px;
}

.status-text {
  font-weight: 500;
}

.status-text.text-running {
  color: #00a870;
}

.status-text.text-stopped {
  color: #bbb;
}

.status-text.text-error {
  color: #e34d59;
}

.status-text.text-reconnecting {
  color: #ed7b2f;
}

.status-text.text-loading {
  color: #0052d9;
}

.pid-info {
  color: #999;
}

.error-message {
  color: #e34d59;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 操作按钮 */
.card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-actions :deep(.t-button) {
  min-width: auto;
}

.favorite-btn {
  color: var(--td-text-color-placeholder);
  transition: color 0.2s;
  cursor: pointer;
}

.favorite-btn:hover {
  color: var(--td-warning-color);
}

/* 已收藏状态时按钮始终显示品牌色 */
.favorite-btn.is-favorite {
  color: var(--td-brand-color);
}

.autostart-btn {
  color: var(--td-text-color-placeholder);
  transition: color 0.2s;
  cursor: pointer;
}

.autostart-btn:hover {
  color: var(--td-brand-color);
}

.autostart-btn.is-autostart {
  color: var(--td-success-color);
}
</style>
