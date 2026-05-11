import { defineStore } from 'pinia'
import { ref } from 'vue'
import { readConfig, saveConfig, type Config, type Repository, type Agent, type Settings } from '@/api/tauri'

export const useConfigStore = defineStore('config', () => {
  const config = ref<Config | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadConfig() {
    loading.value = true
    error.value = null
    try {
      config.value = await readConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveConfigData() {
    if (!config.value) return
    loading.value = true
    error.value = null
    try {
      await saveConfig(config.value)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  function addRepository(repo: Repository) {
    if (!config.value) return
    config.value.repositories.push(repo)
    saveConfigData()
  }

  function updateRepository(repoId: string, updates: Partial<Repository>) {
    if (!config.value) return
    const repo = config.value.repositories.find(r => r.id === repoId)
    if (repo) {
      Object.assign(repo, updates)
      saveConfigData()
    }
  }

  function removeRepository(repoId: string) {
    if (!config.value) return
    config.value.repositories = config.value.repositories.filter(r => r.id !== repoId)
    saveConfigData()
  }

  function updateAgent(agentId: string, enabled: boolean) {
    if (!config.value) return
    const agent = config.value.agents.find(a => a.id === agentId)
    if (agent) {
      agent.enabled = enabled
      saveConfigData()
    }
  }

  function updateSettings(updates: Partial<Settings>) {
    if (!config.value) return
    Object.assign(config.value.settings, updates)
    saveConfigData()
  }

  function batchUpdateSettings(updates: Partial<Settings>) {
    if (!config.value) return
    Object.assign(config.value.settings, updates)
  }

  function batchUpdateAgents(updates: Array<{ id: string; enabled: boolean }>) {
    if (!config.value) return
    for (const update of updates) {
      const agent = config.value.agents.find(a => a.id === update.id)
      if (agent) {
        agent.enabled = update.enabled
      }
    }
  }

  function addAgent(agent: Agent) {
    if (!config.value) return
    config.value.agents.push(agent)
    saveConfigData()
  }

  function updateAgentInfo(agentId: string, updates: { name?: string; path?: string }) {
    if (!config.value) return
    const agent = config.value.agents.find(a => a.id === agentId)
    if (agent) {
      Object.assign(agent, updates)
      saveConfigData()
    }
  }

  function removeAgent(agentId: string) {
    if (!config.value) return
    config.value.agents = config.value.agents.filter(a => a.id !== agentId)
    saveConfigData()
  }

  return {
    config,
    loading,
    error,
    loadConfig,
    saveConfigData,
    addRepository,
    updateRepository,
    removeRepository,
    updateAgent,
    updateSettings,
    batchUpdateSettings,
    batchUpdateAgents,
    addAgent,
    updateAgentInfo,
    removeAgent,
  }
})