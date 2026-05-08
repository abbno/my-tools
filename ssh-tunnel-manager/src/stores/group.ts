import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Group, CreateGroupRequest } from '@/types'
import * as api from '@/api/tauri'

export const useGroupStore = defineStore('group', () => {
  const groups = ref<Group[]>([])
  const loading = ref(false)

  async function fetchGroups() {
    loading.value = true
    try {
      groups.value = await api.getGroups()
    } finally {
      loading.value = false
    }
  }

  async function createGroup(request: CreateGroupRequest) {
    const group = await api.saveGroup(request)
    groups.value.push(group)
    groups.value.sort((a, b) => a.sortOrder - b.sortOrder)
    return group
  }

  async function removeGroup(id: string) {
    await api.deleteGroup(id)
    groups.value = groups.value.filter(g => g.id !== id)
  }

  return {
    groups,
    loading,
    fetchGroups,
    createGroup,
    removeGroup
  }
})
