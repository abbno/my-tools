<template>
  <t-dialog
    :visible="visible"
    header="设置 SSH 密钥认证"
    width="420px"
    :close-on-overlay-click="false"
    :on-close="handleClose"
    @update:visible="emit('update:visible', $event)"
  >
    <!-- 结果显示 -->
    <div v-if="result" class="setup-result">
      <div v-if="result.success" class="success-message">
        <t-icon name="check-circle" style="color: #52c41a; font-size: 24px;" />
        <p>{{ result.message }}</p>
        <p class="key-path">密钥路径: {{ result.keyPath }}</p>
      </div>
      <div v-else class="error-message">
        <t-icon name="close-circle" style="color: #ff4d4f; font-size: 24px;" />
        <p>{{ result.message }}</p>
        <p class="retry-tip">请检查密码是否正确后重试</p>
      </div>
    </div>

    <!-- 进度和等待状态 -->
    <div v-else>
      <!-- 配置信息 -->
      <div class="config-info">
        <p>将为以下服务器生成密钥并部署：</p>
        <p class="config-detail">
          <strong>{{ currentFormData?.name }}</strong> - {{ currentFormData?.username }}@{{ currentFormData?.host }}:{{ currentFormData?.port }}
        </p>
      </div>

      <!-- 进度显示 -->
      <div v-if="inProgress" class="progress-area">
        <t-loading text="等待用户操作..." size="small" />
        <p class="progress-tip">请在弹出的命令窗口中输入密码部署密钥</p>
        <p class="progress-tip">完成后关闭命令窗口</p>
      </div>

      <!-- 开始按钮 -->
      <div v-else class="start-area">
        <p class="start-tip">点击"开始设置"将弹出命令窗口</p>
        <p class="start-tip">请在命令窗口中输入服务器密码完成密钥部署</p>
      </div>
    </div>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="dialog-footer">
        <t-button v-if="!result && !inProgress" variant="outline" @click="handleClose">
          取消
        </t-button>
        <t-button
          v-if="!result && !inProgress"
          theme="primary"
          @click="handleStartSetup"
        >
          开始设置
        </t-button>
        <t-button v-if="inProgress" loading disabled>
          等待中...
        </t-button>
        <!-- 成功时显示完成按钮 -->
        <t-button v-if="result?.success" theme="primary" @click="handleComplete">
          完成
        </t-button>
        <!-- 失败时显示重试按钮 -->
        <t-button v-if="result && !result.success" theme="primary" @click="handleRetry">
          重试
        </t-button>
        <t-button v-if="result && !result.success" variant="outline" @click="handleClose">
          取消
        </t-button>
      </div>
    </template>
  </t-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useKeySetupStore } from '@/stores/keySetup'
import type { CreateConfigRequest, UpdateConfigRequest } from '@/types'

interface Props {
  visible: boolean
}

const props = withDefaults(defineProps<Props>(), {})

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'complete': [data: { success: boolean; configData?: CreateConfigRequest | UpdateConfigRequest }]
}>()

const keySetupStore = useKeySetupStore()

const visible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val)
})

const currentFormData = computed(() => keySetupStore.currentFormData)
const inProgress = computed(() => keySetupStore.inProgress)
const result = computed(() => keySetupStore.result)

async function handleStartSetup() {
  await keySetupStore.startSetup()
}

function handleRetry() {
  keySetupStore.resetForRetry()
}

function handleClose() {
  keySetupStore.closeDialog()
  emit('update:visible', false)
}

function handleComplete() {
  const success = keySetupStore.result?.success ?? false
  const configData = keySetupStore.getUpdatedConfigData()

  emit('complete', {
    success,
    configData
  })

  // 通知上层的 ConfigForm 关闭
  keySetupStore.closeForm()

  emit('update:visible', false)
  keySetupStore.closeDialog()
}
</script>

<style scoped>
.config-info {
  margin-bottom: 16px;
}

.config-detail {
  color: var(--td-text-color-primary);
  margin-top: 8px;
}

.progress-area {
  padding: 16px 0;
  text-align: center;
}

.progress-tip {
  color: var(--td-text-color-placeholder);
  font-size: 13px;
  margin-top: 8px;
}

.start-area {
  padding: 16px 0;
  text-align: center;
}

.start-tip {
  color: var(--td-text-color-placeholder);
  font-size: 13px;
  margin-top: 4px;
}

.setup-result {
  padding: 16px;
  text-align: center;
}

.success-message {
  color: var(--td-success-color);
}

.error-message {
  color: var(--td-error-color);
}

.key-path {
  color: var(--td-text-color-placeholder);
  font-size: 12px;
  margin-top: 8px;
}

.retry-tip {
  color: var(--td-text-color-placeholder);
  font-size: 12px;
  margin-top: 8px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>