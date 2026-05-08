<template>
  <t-dialog
    v-model:visible="visible"
    header="技能冲突"
    width="500px"
    :confirm-btn="null"
    :cancel-btn="null"
    @close="onClose"
  >
    <t-space direction="vertical" size="large">
      <t-alert theme="warning" :message="`发现同名技能: ${conflict.skillName}`" />

      <t-text>该技能已在以下位置存在，请选择如何处理：</t-text>

      <t-list>
        <t-list-item v-for="option in options" :key="option.id">
          <t-list-item-meta :title="option.label" :description="option.description" />
          <template #action>
            <t-button size="small" @click="onSelect(option.id)">
              选择
            </t-button>
          </template>
        </t-list-item>
      </t-list>
    </t-space>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface ConflictInfo {
  skillName: string
  existingRepoId: string
  existingRepoName: string
  newRepoId: string
  newRepoName: string
}

const visible = ref(true)
const props = defineProps<{
  conflict: ConflictInfo
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', choice: 'existing' | 'new' | 'skip'): void
}>()

const options = computed(() => [
  {
    id: 'existing',
    label: `保留现有版本 (${props.conflict.existingRepoName})`,
    description: '不替换现有技能，保持原版本',
  },
  {
    id: 'new',
    label: `使用新版本 (${props.conflict.newRepoName})`,
    description: '替换现有技能为新仓库版本',
  },
  {
    id: 'skip',
    label: '跳过此技能',
    description: '不部署此技能',
  },
])

function onSelect(choice: string) {
  visible.value = false
  emit('select', choice as 'existing' | 'new' | 'skip')
}

function onClose() {
  emit('close')
}
</script>

<style scoped>
</style>