import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Config, CreateConfigRequest, UpdateConfigRequest } from '@/types'
import * as api from '@/api/tauri'

export const useConfigStore = defineStore('config', () => {
  const configs = ref<Config[]>([])
  const loading = ref(false)
  const selectedGroupId = ref<string | null>(null)
  const selectedFavoriteId = ref<string | null>(null)

  // 常用配置列表（排序后）
  const favorites = computed(() => {
    return configs.value
      .filter(c => c.isFavorite)
      .sort((a, b) => a.favoriteOrder - b.favoriteOrder)
  })

  // 过滤后的配置列表
  const filteredConfigs = computed(() => {
    // 如果选中了常用项（单个）
    if (selectedFavoriteId.value && selectedFavoriteId.value !== 'all') {
      return configs.value.filter(c => c.id === selectedFavoriteId.value)
    }

    // 如果选中了"常用"标题（显示全部常用）
    if (selectedFavoriteId.value === 'all') {
      return configs.value
        .filter(c => c.isFavorite)
        .sort((a, b) => a.favoriteOrder - b.favoriteOrder)
    }

    // 按分组筛选
    if (selectedGroupId.value) {
      return configs.value.filter(c => c.groupId === selectedGroupId.value)
    }

    return configs.value
  })

  // 获取配置列表
  async function fetchConfigs(groupId?: string) {
    loading.value = true
    try {
      configs.value = await api.getConfigs(groupId)
    } finally {
      loading.value = false
    }
  }

  // 创建配置
  async function createConfig(request: CreateConfigRequest) {
    const config = await api.saveConfig(request)
    configs.value.push(config)
    return config
  }

  // 更新配置
  async function updateConfig(request: UpdateConfigRequest) {
    const config = await api.updateConfig(request)
    const index = configs.value.findIndex(c => c.id === request.id)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }

  // 删除配置
  async function removeConfig(id: string) {
    await api.deleteConfig(id)
    configs.value = configs.value.filter(c => c.id !== id)
  }

  // 搜索配置
  async function search(keyword: string) {
    loading.value = true
    try {
      configs.value = await api.searchConfigs(keyword)
    } finally {
      loading.value = false
    }
  }

  // 设置选中分组
  function setSelectedGroup(groupId: string | null) {
    selectedGroupId.value = groupId
    selectedFavoriteId.value = null  // 清除常用筛选
  }

  // 设置选中常用项
  function setSelectedFavorite(favoriteId: string | null) {
    selectedFavoriteId.value = favoriteId
    selectedGroupId.value = null  // 清除分组筛选
  }

  // 设置常用
  async function setFavorite(configId: string, isFavorite: boolean) {
    const config = await api.setFavorite(configId, isFavorite)
    const index = configs.value.findIndex(c => c.id === configId)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }

  // 重新排序常用项
  async function reorderFavorites(orders: { configId: string; order: number }[]) {
    await api.reorderFavorites(orders)
    // 更新本地状态
    for (const { configId, order } of orders) {
      const index = configs.value.findIndex(c => c.id === configId)
      if (index !== -1) {
        configs.value[index].favoriteOrder = order
      }
    }
  }

  // 设置开机启动
  async function setAutoStart(configId: string, autoStart: boolean) {
    const config = await api.setTunnelAutostart(configId, autoStart)
    const index = configs.value.findIndex(c => c.id === configId)
    if (index !== -1) {
      configs.value[index] = config
    }
    return config
  }

  return {
    configs,
    loading,
    selectedGroupId,
    selectedFavoriteId,
    favorites,
    filteredConfigs,
    fetchConfigs,
    createConfig,
    updateConfig,
    removeConfig,
    search,
    setSelectedGroup,
    setSelectedFavorite,
    setFavorite,
    reorderFavorites,
    setAutoStart
  }
})
