<template>
  <!-- Settings page - full screen without layout -->
  <router-view v-if="isSettingsPage" />

  <!-- Main layout for other pages -->
  <t-layout v-else class="app-layout">
    <!-- Header -->
    <t-header class="app-header">
      <div class="header-content">
        <div class="brand">
          <span class="brand-icon">◈</span>
          <span class="brand-title">Skills Manager</span>
        </div>
        <div class="header-actions">
          <t-input
            v-model="searchQuery"
            placeholder="搜索技能..."
            clearable
            class="search-input"
            @input="onSearch"
          >
            <template #prefix-icon>
              <search-icon />
            </template>
          </t-input>
          <t-button
            variant="outline"
            shape="circle"
            @click="router.push('/settings')"
          >
            <setting-icon />
          </t-button>
        </div>
      </div>
    </t-header>

    <t-layout class="main-layout">
      <!-- Sidebar -->
      <t-aside class="app-aside">
        <div class="aside-header">
          <div class="aside-actions">
            <t-button
              variant="outline"
              :loading="syncStore.syncing"
              @click="onSyncAll"
            >
              <template #icon>
                <refresh-icon />
              </template>
              同步全部
            </t-button>
            <t-button
              variant="outline"
              @click="openAddRepo"
            >
              <template #icon>
                <add-icon />
              </template>
              添加
            </t-button>
          </div>
        </div>

        <t-menu
          :value="currentRepoId"
          @change="onRepoChange"
          class="repo-menu"
        >
          <t-menu-group title="仓库">
            <t-menu-item
              v-for="repo in configStore.config?.repositories || []"
              :key="repo.id"
              :value="repo.id"
            >
              <template #icon>
                <folder-icon />
              </template>
              <span class="repo-name">{{ repo.name }}</span>
              <template #suffix>
                <span
                  v-if="getSyncStatus(repo.id) === 'syncing'"
                  class="status-syncing"
                >
                  <t-loading size="small" />
                </span>
                <span
                  v-else-if="getSyncStatus(repo.id) === 'success'"
                  class="status-success"
                >
                  ✓
                </span>
                <span
                  v-else-if="getSyncStatus(repo.id) === 'error'"
                  class="status-error"
                >
                  ✗
                </span>
              </template>
            </t-menu-item>
          </t-menu-group>
        </t-menu>

        <!-- Empty state -->
        <div
          v-if="!configStore.config?.repositories?.length && !configStore.loading"
          class="empty-repos"
        >
          <span class="empty-icon">◇</span>
          <p>暂无仓库</p>
          <t-button
            theme="primary"
            @click="openAddRepo"
          >
            添加第一个仓库
          </t-button>
        </div>
      </t-aside>

      <!-- Main Content -->
      <t-content class="app-content">
        <router-view />
      </t-content>
    </t-layout>
  </t-layout>

  <!-- Dialogs -->
  <GitInstallDialog
    v-if="showGitDialog"
    @close="showGitDialog = false"
    @installed="showGitDialog = false"
  />
  <AddRepoDialog v-model:visible="showAddRepo" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import { useSyncStore } from '@/stores/sync'
import { checkGitInstalled, syncAllRepositories } from '@/api/tauri'
import GitInstallDialog from '@/components/GitInstallDialog.vue'
import AddRepoDialog from '@/views/AddRepoDialog.vue'
import {
  SearchIcon,
  SettingIcon,
  FolderIcon,
  AddIcon,
  RefreshIcon,
} from 'tdesign-icons-vue-next'

const configStore = useConfigStore()
const skillsStore = useSkillsStore()
const syncStore = useSyncStore()
const router = useRouter()
const route = useRoute()

// Check if on settings page
const isSettingsPage = computed(() => route.path === '/settings')

// Dialog visibility
const showGitDialog = ref(false)
const showAddRepo = ref(false)

// Local search query
const searchQuery = ref('')

// Current repo ID computed from skillsStore
const currentRepoId = computed(() => skillsStore.currentRepoId || '')

// Event listener cleanup
let unlisten: (() => void) | null = null

function onSearch() {
  skillsStore.setSearchQuery(searchQuery.value)
}

function onRepoChange(value: string) {
  skillsStore.setCurrentRepo(value || null)
}

function getSyncStatus(repoId: string): string {
  const progress = syncStore.getProgress(repoId)
  return progress?.status || 'pending'
}

function openAddRepo() {
  showAddRepo.value = true
}

async function onSyncAll() {
  syncStore.startSync()
  syncStore.clearProgress()

  try {
    await syncAllRepositories()
    for (const repo of configStore.config?.repositories || []) {
      syncStore.endSync(repo.id)
    }
  } catch (error) {
    syncStore.syncing = false
  }
}

onMounted(async () => {
  // Load config
  configStore.loadConfig()

  // Check git installation
  try {
    const status = await checkGitInstalled()
    if (!status.installed) {
      showGitDialog.value = true
    }
  } catch (error) {
    console.error('Failed to check git installation:', error)
  }

  // Listen for sync progress events
  unlisten = await listen('sync-progress', (event) => {
    const payload = event.payload as { repo_id: string; status: string; message?: string }
    if (payload.status === 'syncing') {
      syncStore.startSync(payload.repo_id)
    } else if (payload.status === 'success') {
      syncStore.endSync(payload.repo_id)
    } else if (payload.status === 'error') {
      syncStore.setError(payload.repo_id, payload.message || 'Unknown error')
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
  background: var(--bg-deep);
}

/* Header */
.app-header {
  height: 64px;
  background: #fff;
  border-bottom: 1px solid #e7e7e7;
}

.header-content {
  height: 100%;
  padding: 0 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.brand-icon {
  color: var(--td-brand-color);
  font-size: 20px;
}

.brand-title {
  font-weight: 700;
  color: var(--td-text-color-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.search-input {
  width: 240px;
}

/* Main Layout */
.main-layout {
  height: calc(100vh - 64px);
}

/* Sidebar */
.app-aside {
  width: 260px;
  background: #fff;
  border-right: 1px solid #e7e7e7;
  display: flex;
  flex-direction: column;
}

.aside-header {
  padding: 16px;
  border-bottom: 1px solid #e7e7e7;
}

.aside-actions {
  display: flex;
  gap: 8px;
}

.repo-menu {
  flex: 1;
  overflow-y: auto;
}

.repo-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-syncing {
  display: flex;
  align-items: center;
}

.status-success {
  color: var(--td-success-color);
}

.status-error {
  color: var(--td-error-color);
}

.empty-repos {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--td-text-color-placeholder);
}

.empty-icon {
  font-size: 32px;
  opacity: 0.5;
}

.empty-repos p {
  margin: 0;
  font-size: 13px;
}

/* Content */
.app-content {
  background: var(--bg-deep);
  overflow: hidden;
}
</style>