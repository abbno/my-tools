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
  const downloadComplete = ref(false)
  const updateInfo = ref<UpdateInfo | null>(null)
  const lastCheckTime = ref<string | null>(null)
  const showDialog = ref(false)

  // 事件监听器
  let unlistenProgress: UnlistenFn | null = null
  let unlistenComplete: UnlistenFn | null = null

  // 初始化：监听下载进度事件
  async function init() {
    console.log('[UpdateStore] 开始初始化事件监听器...')
    try {
      unlistenProgress = await listen<DownloadProgress>('update-download-progress', (event) => {
        console.log('[UpdateStore] 收到下载进度事件:', event.payload.percentage, '%')
        downloadProgress.value = event.payload.percentage
        // 强制触发响应式更新
        downloadProgress.value = Number(downloadProgress.value)
      })

      // 监听下载完成事件
      unlistenComplete = await listen('update-download-complete', () => {
        console.log('[UpdateStore] 收到下载完成事件')
        downloadComplete.value = true
        downloadProgress.value = 100
      })

      console.log('[UpdateStore] 事件监听器初始化完成')
    } catch (error) {
      console.error('[UpdateStore] 事件监听器初始化失败:', error)
    }

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
    if (unlistenComplete) {
      unlistenComplete()
      unlistenComplete = null
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
    console.log('[UpdateStore] 开始下载更新...')
    downloading.value = true
    downloadProgress.value = 0
    downloadComplete.value = false
    console.log('[UpdateStore] 状态已设置: downloading=true, progress=0')

    try {
      console.log('[UpdateStore] 调用 downloadAndInstallUpdate API...')
      await api.downloadAndInstallUpdate()
      console.log('[UpdateStore] downloadAndInstallUpdate API 调用完成')
      // 下载完成事件会通过监听器触发 downloadComplete
      // 等待一小段时间让安装程序启动，然后退出应用
      await new Promise(resolve => setTimeout(resolve, 1500))
      // 退出应用
      try {
        console.log('[UpdateStore] 退出应用...')
        await api.exitApp()
      } catch (e) {
        console.error('[UpdateStore] 退出应用失败:', e)
      }
    } catch (error) {
      console.error('[UpdateStore] 下载安装更新失败:', error)
      downloading.value = false
      throw error
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
    downloadComplete,
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