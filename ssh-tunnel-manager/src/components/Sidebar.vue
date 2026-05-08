<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">分组</span>
      <t-button
        variant="text"
        shape="circle"
        size="small"
        @click="showCreateDialog = true"
      >
        <template #icon>
          <AddIcon />
        </template>
      </t-button>
    </div>

    <div class="sidebar-content">
      <!-- 常用隧道区域 -->
      <div class="favorites-section">
        <div
          class="favorites-header"
          :class="{ active: configStore.selectedFavoriteId === 'all' }"
          @click="selectFavorite('all')"
        >
          <StarFilledIcon class="star-icon" />
          <span class="favorites-title">常用</span>
          <span class="favorites-count">{{ configStore.favorites.length }}</span>
        </div>

        <!-- 常用隧道列表（支持拖拽） -->
        <div ref="favoritesListRef" class="favorites-list">
          <div
            v-for="config in configStore.favorites"
            :key="config.id"
            :data-id="config.id"
            class="favorite-item"
            :class="{ active: configStore.selectedFavoriteId === config.id }"
            @click="selectFavorite(config.id)"
          >
            <span class="favorite-name">{{ config.name }}</span>
          </div>
        </div>
      </div>

      <!-- 全部选项 -->
      <div
        class="group-item"
        :class="{ active: selectedGroupId === null }"
        @click="selectGroup(null)"
      >
        <span class="group-name">全部</span>
        <span class="group-count">{{ totalCount }}</span>
      </div>

      <!-- 分组列表 -->
      <div
        v-for="group in groupStore.groups"
        :key="group.id"
        class="group-item"
        :class="{ active: selectedGroupId === group.id }"
        @click="selectGroup(group.id)"
      >
        <span class="group-name">{{ group.name }}</span>
        <span class="group-count">{{ getGroupConfigCount(group.id) }}</span>
      </div>
    </div>

    <!-- 新建分组对话框 -->
    <t-dialog
      v-model:visible="showCreateDialog"
      header="新建分组"
      :confirm-btn="{ content: '确定', loading: creating }"
      :on-confirm="handleCreateGroup"
      :on-close="handleCloseDialog"
    >
      <t-form
        ref="formRef"
        :data="formData"
        :rules="formRules"
        @submit="handleFormSubmit"
      >
        <t-form-item label="分组名称" name="name">
          <t-input
            v-model="formData.name"
            placeholder="请输入分组名称"
            @enter="handleFormSubmit"
          />
        </t-form-item>
      </t-form>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { AddIcon, StarFilledIcon } from 'tdesign-icons-vue-next'
import Sortable, { SortableEvent } from 'sortablejs'
import type { FormInstanceFunctions, FormRule } from 'tdesign-vue-next'
import { useGroupStore } from '@/stores/group'
import { useConfigStore } from '@/stores/config'

// Stores
const groupStore = useGroupStore()
const configStore = useConfigStore()

// 状态
const showCreateDialog = ref(false)
const creating = ref(false)
const formRef = ref<FormInstanceFunctions>()
const favoritesListRef = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

// 表单数据
const formData = ref({
  name: ''
})

// 表单验证规则
const formRules: Record<string, FormRule[]> = {
  name: [
    { required: true, message: '请输入分组名称', trigger: 'blur' },
    { min: 1, max: 50, message: '分组名称长度为1-50个字符', trigger: 'blur' }
  ]
}

// 当前选中的分组ID
const selectedGroupId = computed(() => configStore.selectedGroupId)

// 总配置数量
const totalCount = computed(() => configStore.configs.length)

// 获取分组配置数量
function getGroupConfigCount(groupId: string): number {
  return configStore.configs.filter(c => c.groupId === groupId).length
}

// 选择分组
function selectGroup(groupId: string | null): void {
  configStore.setSelectedGroup(groupId)
}

// 创建分组
async function handleCreateGroup(): Promise<boolean> {
  const valid = await formRef.value?.validate()
  if (valid !== true) {
    return false
  }

  creating.value = true
  try {
    await groupStore.createGroup({
      name: formData.value.name.trim()
    })
    resetForm()
    showCreateDialog.value = false
    return true
  } catch (error) {
    console.error('创建分组失败:', error)
    return false
  } finally {
    creating.value = false
  }
}

// 表单提交
async function handleFormSubmit(): Promise<void> {
  await handleCreateGroup()
}

// 关闭对话框
function handleCloseDialog(): void {
  resetForm()
}

// 重置表单
function resetForm(): void {
  formData.value.name = ''
  formRef.value?.reset()
}

// 选择常用筛选
function selectFavorite(favoriteId: string | null): void {
  configStore.setSelectedFavorite(favoriteId)
}

// 初始化 Sortable
function initSortable() {
  if (!favoritesListRef.value) return

  // 如果已存在实例，先销毁
  if (sortableInstance) {
    sortableInstance.destroy()
  }

  sortableInstance = new Sortable(favoritesListRef.value, {
    animation: 150,
    ghostClass: 'sortable-ghost',
    chosenClass: 'sortable-chosen',
    dragClass: 'sortable-drag',
    forceFallback: true,
    fallbackTolerance: 3,
    fallbackOnBody: true,
    swapThreshold: 0.65,
    onEnd: async (evt: SortableEvent) => {
      const { oldIndex, newIndex } = evt
      if (oldIndex === newIndex) return

      // 根据新的 DOM 顺序重新排列
      const newOrders: { configId: string; order: number }[] = []
      const items = favoritesListRef.value?.querySelectorAll('.favorite-item')
      if (items) {
        items.forEach((item, index) => {
          const configId = item.getAttribute('data-id')
          if (configId) {
            newOrders.push({ configId, order: index + 1 })
          }
        })
      }

      try {
        await configStore.reorderFavorites(newOrders)
      } catch (error) {
        console.error('重排常用失败:', error)
        // 恢复原始顺序
        nextTick(() => {
          initSortable()
        })
      }
    }
  })
}

// 监听 favorites 变化，重新初始化 Sortable
watch(
  () => configStore.favorites,
  () => {
    nextTick(() => {
      initSortable()
    })
  },
  { deep: true }
)

// 组件挂载时初始化 Sortable
onMounted(() => {
  nextTick(() => {
    initSortable()
  })
})
</script>

<style scoped>
.sidebar {
  width: 200px;
  height: 100%;
  background-color: var(--td-bg-color-container);
  border-right: 1px solid var(--td-component-border);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 12px;
  border-bottom: 1px solid var(--td-component-border);
}

.sidebar-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.group-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.group-item:hover {
  background-color: var(--td-bg-color-container-hover);
}

.group-item.active {
  background-color: var(--td-brand-color-light);
}

.group-name {
  font-size: 14px;
  color: var(--td-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-item.active .group-name {
  color: var(--td-brand-color);
  font-weight: 500;
}

.group-count {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background-color: var(--td-bg-color-secondarycontainer);
  padding: 2px 8px;
  border-radius: 10px;
  min-width: 20px;
  text-align: center;
}

.group-item.active .group-count {
  background-color: var(--td-brand-color-focus);
  color: var(--td-brand-color);
}

/* 常用区域样式 */
.favorites-section {
  padding: 8px 0;
  border-bottom: 1px solid var(--td-component-border);
}

.favorites-header {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.favorites-header:hover {
  background-color: var(--td-bg-color-container-hover);
}

.favorites-header.active {
  background-color: var(--td-brand-color-light);
}

.star-icon {
  color: var(--td-brand-color);
  margin-right: 8px;
}

.favorites-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.favorites-header.active .favorites-title {
  color: var(--td-brand-color);
}

.favorites-count {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background-color: var(--td-bg-color-secondarycontainer);
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: auto;
  min-width: 20px;
  text-align: center;
}

.favorites-header.active .favorites-count {
  background-color: var(--td-brand-color-focus);
  color: var(--td-brand-color);
}

.favorites-list {
  min-height: 0;
}

.favorite-item {
  display: flex;
  align-items: center;
  padding: 8px 16px 8px 32px;
  cursor: grab;
  transition: background-color 0.2s;
  user-select: none;
}

.favorite-item:active {
  cursor: grabbing;
}

.favorite-item:hover {
  background-color: var(--td-bg-color-container-hover);
}

.favorite-item.active {
  background-color: var(--td-brand-color-light);
}

.favorite-name {
  font-size: 13px;
  color: var(--td-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.favorite-item.active .favorite-name {
  color: var(--td-brand-color);
}

/* Sortable 拖拽样式 */
.sortable-ghost {
  opacity: 0.4;
  background-color: var(--td-brand-color-light);
}

.sortable-chosen {
  background-color: var(--td-bg-color-container-hover);
}

.sortable-drag {
  opacity: 1;
  cursor: grabbing !important;
}
</style>