import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface SyncProgress {
  repo_id: string
  status: 'pending' | 'syncing' | 'success' | 'error'
  message?: string
  progress?: number
}

export const useSyncStore = defineStore('sync', () => {
  const syncing = ref(false)
  const progress = ref<Map<string, SyncProgress>>(new Map())

  function startSync(repoId?: string) {
    syncing.value = true
    if (repoId) {
      progress.value.set(repoId, { repo_id: repoId, status: 'syncing' })
    }
  }

  function updateProgress(repoId: string, update: Partial<SyncProgress>) {
    const current = progress.value.get(repoId)
    if (current) {
      progress.value.set(repoId, { ...current, ...update })
    }
  }

  function endSync(repoId?: string) {
    if (repoId) {
      const current = progress.value.get(repoId)
      if (current) {
        progress.value.set(repoId, { ...current, status: 'success' })
      }
    }
    syncing.value = false
  }

  function setError(repoId: string, message: string) {
    const current = progress.value.get(repoId)
    if (current) {
      progress.value.set(repoId, { ...current, status: 'error', message })
    }
    syncing.value = false
  }

  function clearProgress() {
    progress.value.clear()
  }

  function getProgress(repoId: string): SyncProgress | undefined {
    return progress.value.get(repoId)
  }

  return {
    syncing,
    progress,
    startSync,
    updateProgress,
    endSync,
    setError,
    clearProgress,
    getProgress,
  }
})