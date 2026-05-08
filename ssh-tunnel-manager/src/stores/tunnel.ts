import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TunnelInfo, PreCheckResult } from '@/types'
import * as api from '@/api/tauri'

export const useTunnelStore = defineStore('tunnel', () => {
  // 阧道状态映射: configId -> TunnelInfo
  const tunnels = ref<Map<string, TunnelInfo>>(new Map())
  // 加载中的隧道 ID 集合
  const loading = ref<Set<string>>(new Set())
  // 预检查中的隧道 ID 集合
  const prechecking = ref<Set<string>>(new Set())
  // 事件监听是否已初始化
  const listenerInitialized = ref(false)

  // 运行中的隧道数量
  const runningCount = computed(() => {
    let count = 0
    for (const info of tunnels.value.values()) {
      if (info.status === 'running') {
        count++
      }
    }
    return count
  })

  // 监听后端发送的阧道状态变化事件
  async function setupEventListener() {
    if (listenerInitialized.value) return

    try {
      const { listen } = await import('@tauri-apps/api/event')
      await listen<{ configId: string; status: string; pid?: number; errorMessage?: string }>(
        'tunnel-status-changed',
        (event) => {
          console.log('Received tunnel-status-changed event:', event.payload)
          const { configId, status, pid, errorMessage } = event.payload
          const info: TunnelInfo = {
            configId,
            status: status as TunnelInfo['status'],
            pid: pid || null,
            errorMessage: errorMessage || null
          }
          tunnels.value.set(configId, info)
        }
      )
      listenerInitialized.value = true
      console.log('Tunnel status event listener initialized')
    } catch (err) {
      console.error('Failed to setup tunnel status listener:', err)
    }
  }

  // 预检查阧道启动条件
  async function precheckTunnel(configId: string): Promise<PreCheckResult> {
    prechecking.value.add(configId)
    try {
      const result = await api.precheckTunnel(configId)
      return result
    } finally {
      prechecking.value.delete(configId)
    }
  }

  // 启动阧道
  async function startTunnel(configId: string) {
    loading.value.add(configId)
    try {
      const info = await api.startTunnel(configId)
      tunnels.value.set(configId, info)
      return info
    } finally {
      loading.value.delete(configId)
    }
  }

  // 停止阧道
  async function stopTunnel(configId: string) {
    loading.value.add(configId)
    try {
      const info = await api.stopTunnel(configId)
      tunnels.value.set(configId, info)
      return info
    } finally {
      loading.value.delete(configId)
    }
  }

  // 重启阧道
  async function restartTunnel(configId: string) {
    loading.value.add(configId)
    try {
      const info = await api.restartTunnel(configId)
      tunnels.value.set(configId, info)
      return info
    } finally {
      loading.value.delete(configId)
    }
  }

  // 获取单个阧道状态
  async function fetchStatus(configId: string) {
    const info = await api.getTunnelStatus(configId)
    tunnels.value.set(configId, info)
    return info
  }

  // 获取所有运行中的阧道
  async function fetchAllRunning() {
    const infos = await api.getRunningTunnels()
    for (const info of infos) {
      tunnels.value.set(info.configId, info)
    }
    return infos
  }

  // 获取指定阧道状态（同步获取，不请求后端）
  function getStatus(configId: string): TunnelInfo | undefined {
    return tunnels.value.get(configId)
  }

  // 检查阧道是否正在加载中
  function isLoading(configId: string): boolean {
    return loading.value.has(configId)
  }

  // 检查阧道是否正在预检查中
  function isPrechecking(configId: string): boolean {
    return prechecking.value.has(configId)
  }

  return {
    tunnels,
    loading,
    prechecking,
    runningCount,
    listenerInitialized,
    setupEventListener,
    precheckTunnel,
    startTunnel,
    stopTunnel,
    restartTunnel,
    fetchStatus,
    fetchAllRunning,
    getStatus,
    isLoading,
    isPrechecking
  }
})