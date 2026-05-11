<template>
  <t-drawer
    v-model:visible="visible"
    :header="'添加仓库'"
    :footer="true"
    size="560px"
    @close="onClose"
  >
    <!-- Steps indicator -->
    <t-steps :current="currentStep" class="steps-indicator">
      <t-step-item title="输入地址" />
      <t-step-item title="选择分支" />
      <t-step-item title="选择技能" />
      <t-step-item title="确认" />
    </t-steps>

    <!-- Step 0: Input URL + Auth -->
    <div v-if="currentStep === 0" class="step-content">
      <t-form :data="formData" :rules="formRules" ref="formRef" label-align="top">
        <t-form-item label="仓库 URL" name="url">
          <t-input v-model="formData.url" placeholder="https://github.com/user/repo" />
        </t-form-item>

        <t-form-item label="认证方式">
          <t-select v-model="formData.authType">
            <t-option value="none" label="无需认证" />
            <t-option value="token" label="令牌" />
            <t-option value="username-password" label="用户名和密码" />
          </t-select>
        </t-form-item>

        <t-form-item v-if="formData.authType === 'token'" label="访问令牌">
          <t-input v-model="formData.token" type="password" placeholder="ghp_xxx or glpat_xxx" />
        </t-form-item>

        <template v-if="formData.authType === 'username-password'">
          <t-form-item label="用户名">
            <t-input v-model="formData.username" placeholder="用户名" />
          </t-form-item>
          <t-form-item label="密码">
            <t-input v-model="formData.password" type="password" placeholder="密码" />
          </t-form-item>
        </template>
      </t-form>
    </div>

    <!-- Step 1: Select Branch -->
    <div v-if="currentStep === 1" class="step-content">
      <div v-if="loadingBranches" class="loading-container">
        <t-loading text="正在获取分支..." />
      </div>

      <t-alert v-else-if="branchError" theme="error" :message="branchError">
        <template #operation>
          <t-link theme="primary" @click="retryFetchBranches">重试</t-link>
        </template>
      </t-alert>

      <div v-else>
        <t-form label-align="top">
          <t-form-item label="选择分支">
            <t-select v-model="formData.branch" :options="branchOptions" />
          </t-form-item>
        </t-form>
      </div>
    </div>

    <!-- Step 2: Select Skills -->
    <div v-if="currentStep === 2" class="step-content">
      <div v-if="loading" class="loading-container">
        <t-loading text="正在获取仓库中的技能..." />
      </div>

      <t-alert v-else-if="error" theme="error" :message="error">
        <template #operation>
          <t-link theme="primary" @click="retryFetchSkills">重试</t-link>
        </template>
      </t-alert>

      <div v-else class="skills-panel">
        <div class="skills-header">
          <span>{{ skills.length }} 个技能</span>
          <t-link theme="primary" @click="selectAll">
            {{ selectedSkills.length === skills.length ? '取消全选' : '全选' }}
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
              <span class="skill-desc">{{ skill.description || '暂无描述' }}</span>
            </div>
          </t-checkbox>
        </t-checkbox-group>
      </div>
    </div>

    <!-- Step 3: Summary -->
    <div v-if="currentStep === 3" class="step-content">
      <t-descriptions title="汇总" :column="2" bordered>
        <t-descriptions-item label="名称">{{ generatedName }}</t-descriptions-item>
        <t-descriptions-item label="地址" :span="2">
          <code class="url-code">{{ formData.url }}</code>
        </t-descriptions-item>
        <t-descriptions-item label="分支">{{ formData.branch }}</t-descriptions-item>
        <t-descriptions-item label="认证">
          <t-tag theme="primary" variant="light">
            {{ formData.authType === 'none' ? '无' : formData.authType === 'token' ? '令牌' : '用户/密码' }}
          </t-tag>
        </t-descriptions-item>
        <t-descriptions-item label="已选技能" :span="2">
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
              +{{ selectedSkills.length - 5 }} 更多
            </t-tag>
          </div>
        </t-descriptions-item>
      </t-descriptions>
    </div>

    <!-- Footer buttons -->
    <template #footer>
      <div class="dialog-footer">
        <t-button variant="outline" @click="onCancel">
          {{ currentStep === 0 ? '取消' : '返回' }}
        </t-button>
        <t-button
          v-if="currentStep === 0"
          theme="primary"
          :loading="loadingBranches"
          :disabled="!formData.url"
          @click="onFetchBranches"
        >
          获取分支
        </t-button>
        <t-button
          v-else-if="currentStep < 3"
          theme="primary"
          :loading="currentStep === 1 && loading"
          :disabled="currentStep === 2 && selectedSkills.length === 0"
          @click="onConfirm"
        >
          继续
        </t-button>
        <t-button
          v-else
          theme="primary"
          @click="onSave"
        >
          保存仓库
        </t-button>
      </div>
    </template>
  </t-drawer>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import { fetchBranches, fetchRepoSkills, type SkillMeta, type AuthConfig } from '@/api/tauri'
import { v4 as uuidv4 } from 'uuid'

const configStore = useConfigStore()
const skillsStore = useSkillsStore()

const visible = defineModel<boolean>('visible', { default: false })

const currentStep = ref(0)
const loadingBranches = ref(false)
const branchError = ref<string | null>(null)
const branches = ref<string[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const skills = ref<SkillMeta[]>([])
const selectedSkills = ref<string[]>([])
const formRef = ref()

const formData = ref({
  url: '',
  branch: '',
  authType: 'none',
  token: '',
  username: '',
  password: '',
})

const formRules = {
  url: [{ required: true, message: 'URL is required' }],
}

const branchOptions = computed(() =>
  branches.value.map(b => ({ label: b, value: b }))
)

// Generate name from URL and branch
const generatedName = computed(() => {
  const url = formData.value.url
  const branch = formData.value.branch

  // Parse URL: https://github.com/owner/repo -> owner/repo
  let name = url
    .replace(/^https?:\/\//, '')
    .replace(/^git@/, '')
    .replace(/\.git$/, '')

  // Remove domain
  const parts = name.split('/')
  if (parts.length >= 3) {
    name = parts.slice(2).join('/')
  }

  return `${name}(${branch})`
})

// Reset state when dialog opens
watch(visible, (val) => {
  if (val) {
    currentStep.value = 0
    loadingBranches.value = false
    branchError.value = null
    branches.value = []
    loading.value = false
    error.value = null
    skills.value = []
    selectedSkills.value = []
    formData.value = {
      url: '',
      branch: '',
      authType: 'none',
      token: '',
      username: '',
      password: '',
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

function getAuthConfig(): AuthConfig {
  return {
    type: formData.value.authType as 'none' | 'token' | 'username-password',
    token: formData.value.authType === 'token' ? formData.value.token : undefined,
    username: formData.value.authType === 'username-password' ? formData.value.username : undefined,
    password: formData.value.authType === 'username-password' ? formData.value.password : undefined,
  }
}

async function onFetchBranches() {
  const valid = await formRef.value?.validate()
  if (valid !== true) return

  loadingBranches.value = true
  branchError.value = null

  try {
    const auth = getAuthConfig()
    branches.value = await fetchBranches(formData.value.url, auth)
    if (branches.value.length > 0) {
      formData.value.branch = branches.value[0]
      currentStep.value = 1
    } else {
      branchError.value = '未找到任何分支'
    }
  } catch (e) {
    branchError.value = String(e)
  } finally {
    loadingBranches.value = false
  }
}

function retryFetchBranches() {
  branchError.value = null
  onFetchBranches()
}

async function fetchSkillsPreview() {
  loading.value = true
  error.value = null
  try {
    const auth = getAuthConfig()
    skills.value = await fetchRepoSkills(formData.value.url, formData.value.branch, auth)
    selectedSkills.value = skills.value.map(s => s.path)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function retryFetchSkills() {
  error.value = null
  fetchSkillsPreview()
}

async function onConfirm() {
  if (currentStep.value === 1) {
    // Go to fetch skills
    currentStep.value = 2
    await fetchSkillsPreview()
  } else if (currentStep.value === 2) {
    if (selectedSkills.value.length === 0) {
      MessagePlugin.warning('请至少选择一个技能')
      return
    }
    currentStep.value = 3
  }
}

async function onSave() {
  const auth = getAuthConfig()
  configStore.addRepository({
    id: uuidv4(),
    name: generatedName.value,
    url: formData.value.url,
    branch: formData.value.branch,
    auth,
    sync_interval: 3600,
    selected_skills: selectedSkills.value,
    last_sync: null,
    enabled: true,
  })
  MessagePlugin.success('仓库添加成功')

  // Select the new repo
  skillsStore.setCurrentRepo(configStore.config?.repositories?.slice(-1)[0]?.id || null)

  visible.value = false
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