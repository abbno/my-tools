<template>
  <t-dialog
    v-model:visible="visible"
    header="技能冲突"
    :width="440"
    :footer="false"
    :close-btn="true"
    @close="onClose"
  >
    <div class="conflict-content">
      <t-alert theme="warning">
        发现重复技能： <strong>{{ conflict.skillName }}</strong>
      </t-alert>

      <p class="conflict-intro">该技能已存在。请选择处理方式：</p>

      <t-radio-group v-model="selectedOption" class="options-group">
        <div
          v-for="option in options"
          :key="option.id"
          class="option-item"
          @click="selectedOption = option.id"
        >
          <t-radio :value="option.id">
            <div class="option-content">
              <span class="option-label">{{ option.label }}</span>
              <span class="option-desc">{{ option.description }}</span>
            </div>
          </t-radio>
        </div>
      </t-radio-group>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <t-button variant="outline" @click="onClose">取消</t-button>
        <t-button theme="primary" @click="onConfirm">确认</t-button>
      </div>
    </template>
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
const selectedOption = ref<'existing' | 'new' | 'skip'>('existing')

const props = defineProps<{
  conflict: ConflictInfo
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', choice: 'existing' | 'new' | 'skip'): void
}>()

const options = computed(() => [
  {
    id: 'existing' as const,
    label: `保留现有版本 (${props.conflict.existingRepoName})`,
    description: '不替换，保留当前版本',
  },
  {
    id: 'new' as const,
    label: `使用新版本 (${props.conflict.newRepoName})`,
    description: '替换为新仓库版本',
  },
  {
    id: 'skip' as const,
    label: '跳过此技能',
    description: '不部署此技能',
  },
])

function onConfirm() {
  visible.value = false
  emit('select', selectedOption.value)
}

function onClose() {
  emit('close')
}
</script>

<style scoped>
.conflict-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.conflict-intro {
  margin: 0;
  color: var(--td-text-color-secondary);
  font-size: 14px;
}

.options-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.option-item {
  padding: 12px;
  background: var(--td-bg-color-container);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
  cursor: pointer;
  transition: all 0.2s ease;
}

.option-item:hover {
  border-color: var(--td-brand-color);
  background: var(--td-bg-color-container-hover);
}

.option-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-label {
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.option-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>