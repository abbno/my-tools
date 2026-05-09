<template>
  <t-dialog
    v-model:visible="visible"
    width="600px"
    :header="'Add Repository'"
    :confirm-btn="null"
    :cancel-btn="null"
    @close="onClose"
  >
    <!-- Steps indicator -->
    <t-steps :current="currentStep" class="steps-indicator">
      <t-step-item title="Repository Info" />
      <t-step-item title="Select Skills" />
      <t-step-item title="Confirm" />
    </t-steps>

    <!-- Step 0: Input form -->
    <div v-if="currentStep === 0" class="step-content">
      <t-form :data="formData" :rules="formRules" ref="formRef" label-align="top">
        <t-form-item label="Repository Name" name="name">
          <t-input v-model="formData.name" placeholder="My Skills Repo" />
        </t-form-item>

        <t-form-item label="Repository URL" name="url">
          <t-input v-model="formData.url" placeholder="https://github.com/user/skills-repo" />
        </t-form-item>

        <t-form-item label="Authentication">
          <t-select v-model="formData.authType">
            <t-option value="none" label="None Required" />
            <t-option value="token" label="Token" />
            <t-option value="username-password" label="Username & Password" />
          </t-select>
        </t-form-item>

        <t-form-item v-if="formData.authType === 'token'" label="Access Token">
          <t-input v-model="formData.token" type="password" placeholder="ghp_xxx or glpat_xxx" />
        </t-form-item>

        <template v-if="formData.authType === 'username-password'">
          <t-form-item label="Username">
            <t-input v-model="formData.username" placeholder="Username" />
          </t-form-item>
          <t-form-item label="Password">
            <t-input v-model="formData.password" type="password" placeholder="Password" />
          </t-form-item>
        </template>

        <t-form-item label="Sync Interval">
          <t-select v-model="formData.syncInterval">
            <t-option value="300" label="5 minutes" />
            <t-option value="900" label="15 minutes" />
            <t-option value="1800" label="30 minutes" />
            <t-option value="3600" label="1 hour" />
            <t-option value="7200" label="2 hours" />
            <t-option value="21600" label="6 hours" />
            <t-option value="43200" label="12 hours" />
            <t-option value="86400" label="Daily" />
          </t-select>
        </t-form-item>
      </t-form>
    </div>

    <!-- Step 1: Preview skills -->
    <div v-if="currentStep === 1" class="step-content">
      <div v-if="loading" class="loading-container">
        <t-loading text="Fetching skills from repository..." />
      </div>

      <t-alert v-else-if="error" theme="error" :message="error">
        <template #operation>
          <t-link theme="primary" @click="retryFetch">Retry</t-link>
        </template>
      </t-alert>

      <div v-else class="skills-panel">
        <div class="skills-header">
          <span>{{ skills.length }} skills found</span>
          <t-link theme="primary" @click="selectAll">
            {{ selectedSkills.length === skills.length ? 'Deselect All' : 'Select All' }}
          </t-link>
        </div>
        <t-checkbox-group v-model="selectedSkills" class="skills-list">
          <t-checkbox
            v-for="skill in skills"
            :key="skill.path"
            :value="skill.path"
            class="skill-item"
          >
            <div class="skill-info">
              <span class="skill-name">{{ skill.name }}</span>
              <span class="skill-desc">{{ skill.description || 'No description' }}</span>
            </div>
          </t-checkbox>
        </t-checkbox-group>
      </div>
    </div>

    <!-- Step 2: Summary -->
    <div v-if="currentStep === 2" class="step-content">
      <t-descriptions title="Summary" :column="2" bordered>
        <t-descriptions-item label="Name">{{ formData.name }}</t-descriptions-item>
        <t-descriptions-item label="URL" :span="2">
          <code class="url-code">{{ formData.url }}</code>
        </t-descriptions-item>
        <t-descriptions-item label="Auth">
          <t-tag theme="primary" variant="light">
            {{ formData.authType === 'none' ? 'None' : formData.authType === 'token' ? 'Token' : 'User/Pass' }}
          </t-tag>
        </t-descriptions-item>
        <t-descriptions-item label="Sync">{{ syncIntervalLabel }}</t-descriptions-item>
        <t-descriptions-item label="Skills Selected" :span="2">
          <div class="selected-skills">
            <t-tag
              v-for="path in selectedSkills.slice(0, 5)"
              :key="path"
              theme="default"
              variant="light"
            >
              {{ getSkillName(path) }}
            </t-tag>
            <t-tag v-if="selectedSkills.length > 5" theme="primary" variant="light">
              +{{ selectedSkills.length - 5 }} more
            </t-tag>
          </div>
        </t-descriptions-item>
      </t-descriptions>
    </div>

    <!-- Footer buttons -->
    <template #footer>
      <div class="dialog-footer">
        <t-button variant="outline" @click="onCancel">
          {{ currentStep === 0 ? 'Cancel' : 'Back' }}
        </t-button>
        <t-button theme="primary" @click="onConfirm">
          {{ currentStep === 2 ? 'Save Repository' : 'Continue' }}
        </t-button>
      </div>
    </template>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import { fetchRepoSkills, type SkillMeta, type AuthConfig } from '@/api/tauri'
import { v4 as uuidv4 } from 'uuid'

const configStore = useConfigStore()

const visible = defineModel<boolean>('visible', { default: false })

const currentStep = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)
const skills = ref<SkillMeta[]>([])
const selectedSkills = ref<string[]>([])
const formRef = ref()

const formData = ref({
  name: '',
  url: '',
  authType: 'none',
  token: '',
  username: '',
  password: '',
  syncInterval: '3600',
})

const formRules = {
  name: [{ required: true, message: 'Name is required' }],
  url: [{ required: true, message: 'URL is required' }],
}

const syncIntervalLabel = computed(() => {
  const labels: Record<string, string> = {
    '300': '5 min',
    '900': '15 min',
    '1800': '30 min',
    '3600': '1 hour',
    '7200': '2 hours',
    '21600': '6 hours',
    '43200': '12 hours',
    '86400': 'Daily',
  }
  return labels[formData.value.syncInterval] || '1 hour'
})

// Reset state when dialog opens
watch(visible, (val) => {
  if (val) {
    currentStep.value = 0
    loading.value = false
    error.value = null
    skills.value = []
    selectedSkills.value = []
    formData.value = {
      name: '',
      url: '',
      authType: 'none',
      token: '',
      username: '',
      password: '',
      syncInterval: '3600',
    }
  }
})

function getSkillName(path: string): string {
  const skill = skills.value.find(s => s.path === path)
  return skill?.name || path
}

function selectAll() {
  if (selectedSkills.value.length === skills.value.length) {
    selectedSkills.value = []
  } else {
    selectedSkills.value = skills.value.map(s => s.path)
  }
}

async function fetchSkillsPreview() {
  loading.value = true
  error.value = null
  try {
    const auth: AuthConfig = {
      type: formData.value.authType as 'none' | 'token' | 'username-password',
      token: formData.value.authType === 'token' ? formData.value.token : undefined,
      username: formData.value.authType === 'username-password' ? formData.value.username : undefined,
      password: formData.value.authType === 'username-password' ? formData.value.password : undefined,
    }
    skills.value = await fetchRepoSkills(formData.value.url, auth)
    selectedSkills.value = skills.value.map(s => s.path)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function retryFetch() {
  error.value = null
  fetchSkillsPreview()
}

async function onConfirm() {
  if (currentStep.value === 0) {
    const valid = await formRef.value?.validate()
    if (valid !== true) return
    currentStep.value = 1
    await fetchSkillsPreview()
  } else if (currentStep.value === 1) {
    if (selectedSkills.value.length === 0) {
      MessagePlugin.warning('Please select at least one skill')
      return
    }
    currentStep.value = 2
  } else if (currentStep.value === 2) {
    const auth: AuthConfig = {
      type: formData.value.authType as 'none' | 'token' | 'username-password',
      token: formData.value.authType === 'token' ? formData.value.token : undefined,
      username: formData.value.authType === 'username-password' ? formData.value.username : undefined,
      password: formData.value.authType === 'username-password' ? formData.value.password : undefined,
    }
    configStore.addRepository({
      id: uuidv4(),
      name: formData.value.name,
      url: formData.value.url,
      auth,
      sync_interval: Number(formData.value.syncInterval),
      selected_skills: selectedSkills.value,
      last_sync: null,
      enabled: true,
    })
    MessagePlugin.success('Repository added successfully')
    visible.value = false
  }
}

function onCancel() {
  if (currentStep.value === 0) {
    visible.value = false
  } else {
    currentStep.value--
  }
}

function onClose() {
  visible.value = false
}
</script>

<style scoped>
.steps-indicator {
  margin-bottom: 24px;
}

.step-content {
  min-height: 280px;
  padding: 16px 0;
}

.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

.skills-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skills-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
  color: var(--td-text-color-secondary);
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 240px;
  overflow-y: auto;
}

.skill-item {
  padding: 12px;
  background: var(--td-bg-color-container);
  border: 1px solid var(--td-component-border);
  border-radius: var(--td-radius-default);
  transition: all 0.2s;
}

.skill-item:hover {
  border-color: var(--td-brand-color);
}

.skill-item :deep(.t-checkbox__label) {
  width: 100%;
}

.skill-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.skill-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
}

.skill-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.url-code {
  font-family: var(--td-font-family-mono);
  font-size: 12px;
  background: var(--td-bg-color-specialcomponent);
  padding: 4px 8px;
  border-radius: 4px;
}

.selected-skills {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>