// SSH 密钥设置状态管理
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { setupSshKey } from '@/api'
import type { TunnelType, SetupKeyResult, CreateConfigRequest, UpdateConfigRequest } from '@/types'

// 表单数据类型（与 ConfigForm 中的 formData 一致）
export interface FormData {
  name: string
  groupId: string | null
  host: string
  port: number
  username: string
  tunnelType: TunnelType
  localHost: string
  localPort: number
  remoteHost: string | null
  remotePort: number | null
  autoReconnect: boolean
  reconnectInterval: number
  isFavorite: boolean
  autoStart: boolean
}

export const useKeySetupStore = defineStore('keySetup', () => {
  // 状态
  const showDialog = ref(false)
  const currentFormData = ref<FormData | null>(null)
  const editMode = ref(false) // 是否为编辑模式
  const editConfigId = ref<string | null>(null) // 编辑模式时的配置 ID
  const inProgress = ref(false)
  const result = ref<SetupKeyResult | null>(null)

  // 显示密钥设置对话框
  function showKeySetupDialog(formData: FormData, configId?: string) {
    currentFormData.value = { ...formData }
    editMode.value = !!configId
    editConfigId.value = configId || null
    showDialog.value = true
    result.value = null
    inProgress.value = false
  }

  // 关闭对话框
  function closeDialog() {
    showDialog.value = false
    currentFormData.value = null
    editMode.value = false
    editConfigId.value = null
    result.value = null
    inProgress.value = false
  }

  // 重试密钥设置（清除结果，重新开始）
  function resetForRetry() {
    result.value = null
    inProgress.value = false
  }

  // 开始密钥设置（弹出 CMD 窗口）
  async function startSetup(): Promise<SetupKeyResult | null> {
    if (!currentFormData.value) {
      return null
    }

    inProgress.value = true

    try {
      const setupResult = await setupSshKey({
        host: currentFormData.value.host,
        port: currentFormData.value.port,
        username: currentFormData.value.username,
      })

      result.value = setupResult
      inProgress.value = false
      return setupResult
    } catch (err) {
      result.value = {
        success: false,
        keyPath: null,
        message: String(err)
      }
      inProgress.value = false
      return null
    }
  }

  // 获取更新后的配置数据（密钥设置成功后）
  function getUpdatedConfigData(): (CreateConfigRequest | UpdateConfigRequest) | undefined {
    if (!currentFormData.value || !result.value?.success || !result.value.keyPath) {
      return undefined
    }

    const data: CreateConfigRequest = {
      name: currentFormData.value.name.trim(),
      groupId: currentFormData.value.groupId,
      host: currentFormData.value.host.trim(),
      port: currentFormData.value.port,
      username: currentFormData.value.username.trim(),
      authType: 'key', // 转换为密钥认证
      password: null,
      keyPath: result.value.keyPath,
      keyPassphrase: null,
      tunnelType: currentFormData.value.tunnelType,
      localHost: currentFormData.value.localHost.trim(),
      localPort: currentFormData.value.localPort,
      remoteHost: currentFormData.value.tunnelType !== 'dynamic' ? currentFormData.value.remoteHost?.trim() || null : null,
      remotePort: currentFormData.value.tunnelType !== 'dynamic' ? currentFormData.value.remotePort : null,
      autoReconnect: currentFormData.value.autoReconnect,
      reconnectInterval: currentFormData.value.reconnectInterval,
      isFavorite: currentFormData.value.isFavorite,
      autoStart: currentFormData.value.autoStart,
    }

    // 如果是编辑模式，添加 ID
    if (editMode.value && editConfigId.value) {
      return { ...data, id: editConfigId.value } as UpdateConfigRequest
    }

    return data
  }

  // 信号：通知上层的 ConfigForm 关闭
  const signalCloseForm = ref(false)

  // 在 handleComplete 中调用，通知 Home.vue 关闭配寘表单
  function closeForm() {
    signalCloseForm.value = true
  }

  return {
    showDialog,
    currentFormData,
    editMode,
    editConfigId,
    inProgress,
    result,
    signalCloseForm,
    showKeySetupDialog,
    closeDialog,
    resetForRetry,
    startSetup,
    getUpdatedConfigData,
    closeForm,
  }
})