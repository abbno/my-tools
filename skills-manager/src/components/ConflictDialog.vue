<template>
  <t-dialog
    v-model:visible="visible"
    header="Skill Conflict"
    :width="440"
    :footer="false"
    :close-btn="true"
    @close="onClose"
  >
    <div class="conflict-content">
      <t-alert theme="warning">
        Found duplicate skill: <strong>{{ conflict.skillName }}</strong>
      </t-alert>

      <p class="conflict-intro">This skill already exists. Choose how to handle it:</p>

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
        <t-button variant="outline" @click="onClose">Cancel</t-button>
        <t-button theme="primary" @click="onConfirm">Confirm</t-button>
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
    label: `Keep existing (${props.conflict.existingRepoName})`,
    description: 'Do not replace, keep the current version',
  },
  {
    id: 'new' as const,
    label: `Use new version (${props.conflict.newRepoName})`,
    description: 'Replace with the new repository version',
  },
  {
    id: 'skip' as const,
    label: 'Skip this skill',
    description: 'Do not deploy this skill',
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