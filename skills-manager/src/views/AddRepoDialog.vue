<template>
  <t-dialog
    v-model:visible="visible"
    header="添加仓库"
    width="700px"
    :confirm-btn="currentStep === 2 ? '保存' : '下一步'"
    :cancel-btn="currentStep === 0 ? '取消' : '上一步'"
    @confirm="onConfirm"
    @cancel="onCancel"
    @close="onClose"
  >
    <!-- Step 0: Input form -->
    <div v-if="currentStep === 0" class="step-content">
      <t-form :data="formData" :rules="formRules" ref="formRef">
        <t-form-item label="仓库名称" name="name">
          <t-input v-model="formData.name" placeholder="自定义仓库名称" />
        </t-form-item>
        <t-form-item label="仓库 URL" name="url">
          <t-input v-model="formData.url" placeholder="https://github.com/user/skills" />
        </t-form-item>
        <t-form-item label="认证方式" name="authType">
          <t-select v-model="formData.authType" style="width: 100%">
            <t-option value="none" label="无需认证" />
            <t-option value="token" label="Token" />
            <t-option value="username-password" label="用户名密码" />
          </t-select>
        </t-form-item>
        <t-form-item v-if="formData.authType === 'token'" label="Token" name="token">
          <t-input v-model="formData.token" placeholder="ghp_xxx 或 glpat_xxx" />
        </t-form-item>
        <t-form-item v-if="formData.authType === 'username-password'" label="用户名" name="username">
          <t-input v-model="formData.username" placeholder="用户名" />
        </t-form-item>
        <t-form-item v-if="formData.authType === 'username-password'" label="密码" name="password">
          <t-input v-model="formData.password" type="password" placeholder="密码" />
        </t-form-item>
        <t-form-item label="同步间隔" name="syncInterval">
          <t-select v-model="formData.syncInterval" style="width: 100%">
            <t-option value="300" label="5 分钟" />
            <t-option value="900" label="15 分钟" />
            <t-option value="1800" label="30 分钟" />
            <t-option value="3600" label="1 小时" />
            <t-option value="7200" label="2 小时" />
            <t-option value="21600" label="6 小时" />
            <t-option value="43200" label="12 小时" />
            <t-option value="86400" label="每天" />
          </t-select>
        </t-form-item>
      </t-form>
    </div>

    <!-- Step 1: Preview skills -->
    <div v-if="currentStep === 1" class="step-content">
      <t-loading v-if="loading" text="正在获取技能列表..." />
      <t-alert v-else-if="error" theme="error" :message="error" />
      <div v-else class="preview-container">
        <t-space direction="vertical" size="large">
          <t-text>找到 {{ skills.length }} 个技能，请选择需要的技能：</t-text>
          <t-list>
            <t-list-item v-for="skill in skills" :key="skill.path">
              <t-list-item-meta :title="skill.name" :description="skill.description" />
              <template #action>
                <t-checkbox
                  :value="skill.path"
                  :checked="selectedSkills.includes(skill.path)"
                  @change="(val: boolean) => toggleSkill(skill.path, val)"
                />
              </template>
            </t-list-item>
          </t-list>
        </t-space>
      </div>
    </div>

    <!-- Step 2: Summary -->
    <div v-if="currentStep === 2" class="step-content">
      <t-space direction="vertical" size="large">
        <t-descriptions>
          <t-descriptions-item label="仓库名称">{{ formData.name }}</t-descriptions-item>
          <t-descriptions-item label="仓库 URL">{{ formData.url }}</t-descriptions-item>
          <t-descriptions-item label="认证方式">
            {{ formData.authType === 'none' ? '无需认证' : formData.authType === 'token' ? 'Token' : '用户名密码' }}
          </t-descriptions-item>
          <t-descriptions-item label="同步间隔">{{ syncIntervalLabel }}</t-descriptions-item>
          <t-descriptions-item label="已选技能">{{ selectedSkills.length }} 个</t-descriptions-item>
        </t-descriptions>
        <t-text>已选择技能：{{ selectedSkills.join(', ') || '无' }}</t-text>
      </t-space>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { fetchRepoSkills, type SkillMeta, type AuthConfig } from '@/api/tauri'
import { useConfigStore } from '@/stores/config'
import { v4 as uuidv4 } from 'uuid'

const visible = ref(true)
const currentStep = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)
const skills = ref<SkillMeta[]>([])
const selectedSkills = ref<string[]>([])
const formRef = ref()

const configStore = useConfigStore()

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
  name: [{ required: true, message: '请输入仓库名称', trigger: 'blur' }],
  url: [{ required: true, message: '请输入仓库 URL', trigger: 'blur' }],
}

const syncIntervalLabel = computed(() => {
  const labels: Record<string, string> = {
    '300': '5 分钟',
    '900': '15 分钟',
    '1800': '30 分钟',
    '3600': '1 小时',
    '7200': '2 小时',
    '21600': '6 小时',
    '43200': '12 小时',
    '86400': '每天',
  }
  return labels[formData.value.syncInterval] || '1 小时'
})

function toggleSkill(skillPath: string, checked: boolean) {
  if (checked) {
    if (!selectedSkills.value.includes(skillPath)) {
      selectedSkills.value.push(skillPath)
    }
  } else {
    selectedSkills.value = selectedSkills.value.filter(s => s !== skillPath)
  }
}

async function onConfirm() {
  if (currentStep.value === 0) {
    // Validate form
    const valid = await formRef.value?.validate()
    if (!valid) return

    // Fetch skills preview
    currentStep.value = 1
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
      selectedSkills.value = skills.value.map(s => s.path) // Select all by default
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  } else if (currentStep.value === 1) {
    // Go to summary
    if (selectedSkills.value.length === 0) {
      MessagePlugin.warning('请至少选择一个技能')
      return
    }
    currentStep.value = 2
  } else if (currentStep.value === 2) {
    // Save repository
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

    MessagePlugin.success('仓库已添加')
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
  emit('close')
}

const emit = defineEmits<{
  (e: 'close'): void
}>()
</script>

<style scoped>
.step-content {
  min-height: 300px;
}

.preview-container {
  max-height: 400px;
  overflow-y: auto;
}
</style>