<template>
  <t-dialog
    v-model:visible="visible"
    :header="isEditMode ? '编辑 Agent' : '添加 Agent'"
    width="480px"
    :footer="true"
    @close="onClose"
  >
    <t-form :data="formData" :rules="formRules" ref="formRef" label-align="top">
      <t-form-item label="Agent 名称" name="name">
        <t-input v-model="formData.name" placeholder="输入 Agent 名称" />
      </t-form-item>

      <t-form-item label="Agent 路径" name="path">
        <t-input v-model="formData.path" placeholder="~/xxx 或完整路径" />
        <div class="path-hint">
          提示：支持 ~/ 开头表示家目录，如 ~/.claude<br />
          或输入完整路径，如 /home/user/.claude
        </div>
      </t-form-item>

      <t-form-item label="启用状态">
        <t-switch v-model="formData.enabled" />
      </t-form-item>
    </t-form>

    <template #footer>
      <t-button variant="outline" @click="onClose">取消</t-button>
      <t-button theme="primary" :loading="saving" @click="onSave">
        {{ isEditMode ? '保存' : '添加' }}
      </t-button>
    </template>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import { getSystemInfo } from '@/api/tauri'
import type { Agent } from '@/api/tauri'
import { v4 as uuidv4 } from 'uuid'

const configStore = useConfigStore()

const visible = defineModel<boolean>('visible', { default: false })
const editAgent = defineModel<Agent | null>('editAgent', { default: null })

const formRef = ref()
const saving = ref(false)
const homeDir = ref('')

const formData = ref({
  name: '',
  path: '',
  enabled: true,
})

const formRules = {
  name: [{ required: true, message: 'Agent 名称必填' }],
  path: [{ required: true, message: 'Agent 路径必填' }],
}

const isEditMode = computed(() => !!editAgent.value)

// Reset form when dialog opens
watch(visible, async (val) => {
  if (val) {
    // Get home directory for path resolution
    try {
      const sysInfo = await getSystemInfo()
      homeDir.value = sysInfo.home_dir
    } catch {
      homeDir.value = ''
    }

    // Prefill form for edit mode
    if (editAgent.value) {
      formData.value = {
        name: editAgent.value.name,
        path: editAgent.value.path,
        enabled: editAgent.value.enabled,
      }
    } else {
      formData.value = {
        name: '',
        path: '',
        enabled: true,
      }
    }
  }
})

function resolvePath(inputPath: string): string {
  if (!homeDir.value) return inputPath
  // Support ~/ and ~\ (Windows)
  if (inputPath.startsWith('~/') || inputPath.startsWith('~\\')) {
    return homeDir.value + inputPath.slice(1)
  }
  return inputPath
}

async function onSave() {
  const valid = await formRef.value?.validate()
  if (valid !== true) return

  saving.value = true
  try {
    const resolvedPath = resolvePath(formData.value.path)

    if (isEditMode.value && editAgent.value) {
      // Update existing agent
      configStore.updateAgentInfo(editAgent.value.id, {
        name: formData.value.name,
        path: resolvedPath,
      })
      // Also update enabled state if changed
      if (formData.value.enabled !== editAgent.value.enabled) {
        configStore.updateAgent(editAgent.value.id, formData.value.enabled)
      }
      MessagePlugin.success('Agent 已更新')
    } else {
      // Create new agent
      const newAgent: Agent = {
        id: uuidv4(),
        name: formData.value.name,
        path: resolvedPath,
        enabled: formData.value.enabled,
      }
      configStore.addAgent(newAgent)
      MessagePlugin.success('Agent 已添加')
    }

    visible.value = false
    editAgent.value = null
  } finally {
    saving.value = false
  }
}

function onClose() {
  visible.value = false
  editAgent.value = null
}
</script>

<style scoped>
.path-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  line-height: 1.5;
}
</style>