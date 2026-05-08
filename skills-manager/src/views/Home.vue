<template>
  <div class="home-container">
    <t-layout>
      <t-sider class="repo-sider" width="200px">
        <div class="sider-header">
          <span>仓库列表</span>
          <t-space>
            <t-button
              size="small"
              variant="outline"
              :loading="syncStore.syncing"
              @click="onSyncAll"
            >
              同步
            </t-button>
            <t-button size="small" variant="outline" @click="showAddRepo = true">
              添加
            </t-button>
          </t-space>
        </div>
        <div class="repo-list">
          <t-loading v-if="configStore.loading" />
          <t-empty v-else-if="!configStore.config?.repositories?.length" description="暂无仓库" />
          <t-list v-else>
            <t-list-item
              v-for="repo in configStore.config.repositories"
              :key="repo.id"
              :class="{ active: skillsStore.currentRepoId === repo.id }"
              @click="skillsStore.setCurrentRepo(repo.id)"
            >
              <t-list-item-meta :title="repo.name" :description="repo.url" />
              <template #action>
                <t-tag
                  v-if="getSyncStatus(repo.id) === 'syncing'"
                  theme="primary"
                  size="small"
                >
                  同步中
                </t-tag>
                <t-tag
                  v-else-if="getSyncStatus(repo.id) === 'success'"
                  theme="success"
                  size="small"
                >
                  已同步
                </t-tag>
                <t-tag
                  v-else-if="getSyncStatus(repo.id) === 'error'"
                  theme="danger"
                  size="small"
                >
                  失败
                </t-tag>
              </template>
            </t-list-item>
          </t-list>
        </div>
        <div class="sider-footer">
          <t-button size="small" block @click="goSettings">设置</t-button>
        </div>
      </t-sider>
      <t-content>
        <t-layout>
          <t-header class="skill-header">
            <t-input
              v-model="searchQuery"
              placeholder="搜索技能"
              class="search-input"
              clearable
            />
          </t-header>
          <t-layout>
            <t-sider class="skill-sider" width="300px">
              <div class="skill-list">
                <t-empty v-if="!filteredSkills.length" description="暂无技能" />
                <t-list v-else>
                  <t-list-item
                    v-for="skill in filteredSkills"
                    :key="skill.path"
                    :class="{ active: skillsStore.selectedSkill?.path === skill.path }"
                    @click="skillsStore.selectSkill(skill)"
                  >
                    <t-list-item-meta :title="skill.name" :description="skill.description" />
                  </t-list-item>
                </t-list>
              </div>
            </t-sider>
            <t-content class="skill-detail">
              <div class="skill-detail-content">
                <t-empty v-if="!skillsStore.selectedSkill" description="请选择技能查看详情" />
                <template v-else>
                  <t-card :title="skillsStore.selectedSkill.name">
                    <t-descriptions>
                      <t-descriptions-item label="描述">
                        {{ skillsStore.selectedSkill.description }}
                      </t-descriptions-item>
                      <t-descriptions-item label="来源仓库">
                        {{ getRepoName(skillsStore.selectedSkill.repo_id) }}
                      </t-descriptions-item>
                      <t-descriptions-item label="路径">
                        {{ skillsStore.selectedSkill.path }}
                      </t-descriptions-item>
                    </t-descriptions>
                    <t-divider />
                    <t-text>SKILL.md 内容预览（待实现）</t-text>
                  </t-card>
                </template>
              </div>
            </t-content>
          </t-layout>
        </t-layout>
      </t-content>
    </t-layout>

    <!-- 添加仓库对话框 -->
    <AddRepoDialog
      v-if="showAddRepo"
      @close="showAddRepo = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { MessagePlugin } from 'tdesign-vue-next'
import { listen } from '@tauri-apps/api/event'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import { useSyncStore } from '@/stores/sync'
import { syncAllRepositories } from '@/api/tauri'
import AddRepoDialog from '@/views/AddRepoDialog.vue'

const router = useRouter()
const configStore = useConfigStore()
const skillsStore = useSkillsStore()
const syncStore = useSyncStore()

const searchQuery = ref('')
const showAddRepo = ref(false)
let unlisten: (() => void) | null = null

const filteredSkills = computed(() => {
  return skillsStore.filteredSkills()
})

function getRepoName(repoId: string): string {
  const repo = configStore.config?.repositories.find(r => r.id === repoId)
  return repo?.name || '未知'
}

function getSyncStatus(repoId: string): string {
  const progress = syncStore.getProgress(repoId)
  return progress?.status || 'pending'
}

function goSettings() {
  router.push('/settings')
}

async function onSyncAll() {
  syncStore.startSync()
  syncStore.clearProgress()

  try {
    const results = await syncAllRepositories()
    MessagePlugin.success('同步完成')

    // Mark all as success
    for (const repo of configStore.config?.repositories || []) {
      syncStore.endSync(repo.id)
    }
  } catch (error) {
    MessagePlugin.error('同步失败: ' + error)
    syncStore.syncing = false
  }
}

onMounted(async () => {
  configStore.loadConfig()

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
  if (unlisten) {
    unlisten()
  }
})
</script>

<style scoped>
.home-container {
  height: 100vh;
}

.repo-sider {
  background: #f5f5f5;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
}

.sider-header {
  padding: 16px;
  font-weight: 500;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.repo-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.repo-list .t-list-item {
  cursor: pointer;
  border-radius: 4px;
  margin-bottom: 4px;
}

.repo-list .t-list-item.active {
  background: #e6f7ff;
}

.sider-footer {
  padding: 16px;
}

.skill-header {
  padding: 16px;
  background: #fff;
  border-bottom: 1px solid #e0e0e0;
}

.search-input {
  width: 300px;
}

.skill-sider {
  background: #fafafa;
  border-right: 1px solid #e0e0e0;
}

.skill-list {
  padding: 8px;
  overflow-y: auto;
  height: calc(100vh - 120px);
}

.skill-list .t-list-item {
  cursor: pointer;
  border-radius: 4px;
  margin-bottom: 4px;
}

.skill-list .t-list-item.active {
  background: #e6f7ff;
}

.skill-detail {
  background: #fff;
  padding: 16px;
  overflow-y: auto;
}

.skill-detail-content {
  height: 100%;
}
</style>