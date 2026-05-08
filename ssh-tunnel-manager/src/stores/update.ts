import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as api from '@/api/tauri'
import type { UpdateInfo, DownloadProgress } from '@/types'

export const useUpdateStore = defineStore('update', () => {
  // 状态
  const checking = ref(false)
  const downloading = ref(false)
  const downloadProgress = ref(0)
  const updateInfo = ref<UpdateInfo | null>(null)
  const lastCheckTime = ref<string | null>(null)
  const showDialog = ref(false)

  // 事件监听器
  let unlistenProgress: UnlistenFn | null = null

  // 初始化：监听下载进度事件
  async function init() {
    unlistenProgress = await listen<DownloadProgress>('update-download-progress', (event) => {
      downloadProgress.value = event.payload.percentage
    })

    // 获取上次检查时间
    const time = await api.getLastCheckTime()
    lastCheckTime.value = time
  }

  // 清理
  function cleanup() {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
    }
  }

  // 检查更新
  async function checkUpdate() {
    checking.value = true
    try {
      const info = await api.checkUpdate()
      updateInfo.value = info
      lastCheckTime.value = new Date().toISOString()

      // 如果有新版本，显示弹窗
      if (info) {
        showDialog.value = true
      }
    } catch (error) {
      console.error('检查更新失败:', error)
      throw error // 重新抛出错误，让调用方处理
    } finally {
      checking.value = false
    }
  }

  // 下载并安装更新
  async function downloadAndInstall() {
    downloading.value = true
    downloadProgress.value = 0
    try {
      await api.downloadAndInstallUpdate()
      // 应用将退出，无需后续处理
    } catch (error) {
      console.error('下载安装更新失败:', error)
      throw error
    } finally {
      downloading.value = false
    }
  }

  // 关闭弹窗
  function closeDialog() {
    showDialog.value = false
  }

  // 稍后提醒
  function remindLater() {
    showDialog.value = false
    // 保持 updateInfo，下次检查时可能再次显示
  }

  return {
    // 状态
    checking,
    downloading,
    downloadProgress,
    updateInfo,
    lastCheckTime,
    showDialog,

    // 方法
    init,
    cleanup,
    checkUpdate,
    downloadAndInstall,
    closeDialog,
    remindLater,
  }
})