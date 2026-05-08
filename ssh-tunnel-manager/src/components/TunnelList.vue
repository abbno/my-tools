<template>
  <div class="tunnel-list">
    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-actions">
        <t-button theme="primary" @click="handleCreate">
          <template #icon>
            <AddIcon />
          </template>
          新建配置
        </t-button>
        <t-button theme="default" variant="outline" @click="handleImport">
          <template #icon>
            <UploadIcon />
          </template>
          导入
        </t-button>
        <t-button theme="default" variant="outline" @click="handleExport">
          <template #icon>
            <DownloadIcon />
          </template>
          导出
        </t-button>
      </div>
      <div class="toolbar-search">
        <t-input
          v-model="searchKeyword"
          placeholder="搜索隧道配置..."
          clearable
          @enter="handleSearch"
          @clear="handleClearSearch"
        >
          <template #prefix-icon>
            <SearchIcon />
          </template>
        </t-input>
        <t-button
          variant="outline"
          shape="circle"
          @click="handleSettings"
          title="设置"
        >
          <template #icon>
            <SettingIcon />
          </template>
        </t-button>
      </div>
    </div>

    <!-- 列表区域 -->
    <div class="list-container">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-wrapper">
        <t-loading size="large" text="加载中..." />
      </div>

      <!-- 空状态 -->
      <div v-else-if="isEmpty" class="empty-wrapper">
        <t-empty description="暂无隧道配置">
          <template #action>
            <t-button theme="primary" size="small" @click="handleCreate">
              创建第一个隧道
            </t-button>
          </template>
        </t-empty>
      </div>

      <!-- 隧道卡片网格 -->
      <div v-else class="tunnel-grid">
        <TunnelCard
          v-for="config in displayConfigs"
          :key="config.id"
          :config="config"
          @edit="handleEdit"
          @delete="handleDelete"
          @show-log="handleShowLog"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { AddIcon, UploadIcon, DownloadIcon, SearchIcon, SettingIcon } from 'tdesign-icons-vue-next'
import type { Config } from '@/types'
import { useConfigStore } from '@/stores/config'
import { useTunnelStore } from '@/stores/tunnel'
import TunnelCard from './TunnelCard.vue'

// Router
const router = useRouter()

// Emits
const emit = defineEmits<{
  create: []
  import: []
  export: []
  edit: [config: Config]
  delete: [configId: string]
  showLog: [configId: string]
}>()

// Stores
const configStore = useConfigStore()
const tunnelStore = useTunnelStore()

// 搜索关键词
const searchKeyword = ref('')

// 计算属性
const loading = computed(() => configStore.loading)

const displayConfigs = computed(() => {
  return configStore.filteredConfigs
})

const isEmpty = computed(() => {
  return displayConfigs.value.length === 0
})

// 方法
function handleCreate() {
  emit('create')
}

function handleImport() {
  emit('import')
}

function handleExport() {
  emit('export')
}

async function handleSearch() {
  if (searchKeyword.value.trim()) {
    await configStore.search(searchKeyword.value.trim())
  }
}

async function handleClearSearch() {
  searchKeyword.value = ''
  await configStore.fetchConfigs()
}

function handleEdit(config: Config) {
  emit('edit', config)
}

function handleDelete(configId: string) {
  emit('delete', configId)
}

function handleShowLog(configId: string) {
  emit('showLog', configId)
}

function handleSettings() {
  router.push('/settings')
}

// 组件挂载时加载数据和设置事件监听
onMounted(async () => {
  // 初始化隧道状态事件监听
  await tunnelStore.setupEventListener()

  await Promise.all([
    configStore.fetchConfigs(),
    tunnelStore.fetchAllRunning()
  ])
})
</script>

<style scoped>
.tunnel-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f5f5f5;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: #fff;
  border-bottom: 1px solid #e7e7e7;
  flex-shrink: 0;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-search {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-search .t-input {
  width: 240px;
}

/* 列表区域 */
.list-container {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

/* 加载状态 */
.loading-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
}

/* 空状态 */
.empty-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 300px;
}

/* 隧道卡片网格 */
.tunnel-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 16px;
}

/* 响应式调整 */
@media (max-width: 900px) {
  .tunnel-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .toolbar {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .toolbar-actions {
    justify-content: center;
  }

  .toolbar-search {
    width: 100%;
  }
}
</style>
