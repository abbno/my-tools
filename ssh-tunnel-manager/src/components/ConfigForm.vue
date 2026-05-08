<template>
  <t-drawer
    :visible="visible"
    :header="isEditMode ? '编辑配置' : '新建配置'"
    placement="right"
    :size="'500px'"
    :footer="true"
    :confirm-btn="{ content: '保存', loading: saving }"
    :on-confirm="handleSubmit"
    :on-close="handleClose"
    @update:visible="emit('update:visible', $event)"
  >
    <t-form
      ref="formRef"
      :data="formData"
      :rules="formRules"
      label-align="right"
      label-width="100px"
    >
      <!-- 基本信息 -->
      <t-divider>基本信息</t-divider>

      <t-form-item label="配置名称" name="name">
        <t-input
          v-model="formData.name"
          placeholder="请输入配置名称"
          clearable
        />
      </t-form-item>

      <t-form-item label="所属分组" name="groupId">
        <t-select
          v-model="formData.groupId"
          placeholder="请选择分组"
          clearable
        >
          <t-option
            v-for="group in groupStore.groups"
            :key="group.id"
            :value="group.id"
            :label="group.name"
          />
        </t-select>
      </t-form-item>

      <!-- 连接信息 -->
      <t-divider>连接信息</t-divider>

      <t-form-item label="主机地址" name="host">
        <t-input
          v-model="formData.host"
          placeholder="例如: 192.168.1.100 或 example.com"
          clearable
        />
      </t-form-item>

      <t-form-item label="端口" name="port">
        <t-input-number
          v-model="formData.port"
          :min="1"
          :max="65535"
          placeholder="SSH 端口"
          style="width: 100%"
        />
      </t-form-item>

      <t-form-item label="用户名" name="username">
        <t-input
          v-model="formData.username"
          placeholder="SSH 登录用户名"
          clearable
        />
      </t-form-item>

      <!-- 认证方式 -->
      <t-divider>认证方式</t-divider>

      <t-form-item label="认证类型" name="authType">
        <t-radio-group v-model="formData.authType">
          <t-radio value="password">密码认证</t-radio>
          <t-radio value="key">密钥认证</t-radio>
        </t-radio-group>
      </t-form-item>

      <!-- 密码认证 -->
      <template v-if="formData.authType === 'password'">
        <t-form-item label="密码" name="password">
          <t-input
            v-model="formData.password"
            type="password"
            placeholder="请输入 SSH 密码"
            clearable
          />
        </t-form-item>
      </template>

      <!-- 密钥认证 -->
      <template v-if="formData.authType === 'key'">
        <t-form-item label="密钥文件" name="keyPath">
          <t-input
            v-model="formData.keyPath"
            placeholder="密钥文件路径，例如: ~/.ssh/id_rsa"
            clearable
          />
        </t-form-item>

        <t-form-item label="密钥密码" name="keyPassphrase">
          <t-input
            v-model="formData.keyPassphrase"
            type="password"
            placeholder="密钥密码（如有）"
            clearable
          />
        </t-form-item>
      </template>

      <!-- 隧道配置 -->
      <t-divider>隧道配置</t-divider>

      <t-form-item label="隧道类型" name="tunnelType">
        <t-radio-group v-model="formData.tunnelType">
          <t-radio value="local">本地转发</t-radio>
          <t-radio value="remote">远程转发</t-radio>
          <t-radio value="dynamic">动态转发</t-radio>
        </t-radio-group>
      </t-form-item>

      <t-form-item label="本地地址" name="localHost">
        <t-input
          v-model="formData.localHost"
          placeholder="本地监听地址"
          clearable
        />
      </t-form-item>

      <t-form-item label="本地端口" name="localPort">
        <t-input-number
          v-model="formData.localPort"
          :min="1"
          :max="65535"
          placeholder="本地监听端口"
          style="width: 100%"
        />
      </t-form-item>

      <!-- 远程地址和端口（动态转发时不显示） -->
      <template v-if="formData.tunnelType !== 'dynamic'">
        <t-form-item label="远程地址" name="remoteHost">
          <t-input
            v-model="formData.remoteHost"
            placeholder="远程目标地址"
            clearable
          />
        </t-form-item>

        <t-form-item label="远程端口" name="remotePort">
          <t-input-number
            v-model="formData.remotePort"
            :min="1"
            :max="65535"
            placeholder="远程目标端口"
            style="width: 100%"
          />
        </t-form-item>
      </template>

      <!-- 高级选项 -->
      <t-divider>高级选项</t-divider>

      <t-form-item label="标记为常用" name="isFavorite">
        <t-switch v-model="formData.isFavorite" />
      </t-form-item>

      <t-form-item label="开机启动" name="autoStart">
        <t-switch v-model="formData.autoStart" />
      </t-form-item>

      <t-form-item label="自动重连" name="autoReconnect">
        <t-switch v-model="formData.autoReconnect" />
      </t-form-item>

      <t-form-item
        v-if="formData.autoReconnect"
        label="重连间隔"
        name="reconnectInterval"
      >
        <t-input-number
          v-model="formData.reconnectInterval"
          :min="1"
          :max="3600"
          placeholder="秒"
          style="width: 100%"
        >
          <template #suffix>秒</template>
        </t-input-number>
      </t-form-item>
    </t-form>
  </t-drawer>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import type { FormInstanceFunctions, FormRule } from 'tdesign-vue-next'
import type { Config, AuthType, TunnelType, CreateConfigRequest, UpdateConfigRequest } from '@/types'
import { useConfigStore } from '@/stores/config'
import { useGroupStore } from '@/stores/group'
import { MessagePlugin } from 'tdesign-vue-next'

// Props
const props = defineProps<{
  visible: boolean
  config?: Config | null
  defaultGroupId?: string | null
}>()

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean]
  'saved': [config: Config]
}>()

// Stores
const configStore = useConfigStore()
const groupStore = useGroupStore()

// 表单引用
const formRef = ref<FormInstanceFunctions>()

// 保存状态
const saving = ref(false)

// 是否为编辑模式
const isEditMode = computed(() => !!props.config?.id)

// 默认表单数据
const defaultFormData = () => ({
  name: '',
  groupId: null as string | null,
  host: '',
  port: 22,
  username: '',
  authType: 'password' as AuthType,
  password: '',
  keyPath: '',
  keyPassphrase: '',
  tunnelType: 'local' as TunnelType,
  localHost: '127.0.0.1',
  localPort: 8080,
  remoteHost: 'localhost',
  remotePort: 80,
  autoReconnect: false,
  reconnectInterval: 10,
  isFavorite: false,
  autoStart: false
})

// 表单数据
const formData = ref(defaultFormData())

// 表单验证规则
const formRules: Record<string, FormRule[]> = {
  name: [
    { required: true, message: '请输入配置名称', trigger: 'blur' },
    { min: 1, max: 100, message: '配置名称长度为1-100个字符', trigger: 'blur' }
  ],
  host: [
    { required: true, message: '请输入主机地址', trigger: 'blur' }
  ],
  port: [
    { required: true, message: '请输入端口', trigger: 'blur' }
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' }
  ],
  password: [
    {
      required: true,
      trigger: 'blur',
      validator: (val: string) => {
        if (formData.value.authType === 'password' && !val) {
          return { result: false, message: '请输入密码', type: 'error' }
        }
        return { result: true, message: '' }
      }
    }
  ],
  keyPath: [
    {
      required: true,
      trigger: 'blur',
      validator: (val: string) => {
        if (formData.value.authType === 'key' && !val) {
          return { result: false, message: '请输入密钥文件路径', type: 'error' }
        }
        return { result: true, message: '' }
      }
    }
  ],
  localPort: [
    { required: true, message: '请输入本地端口', trigger: 'blur' }
  ],
  remoteHost: [
    {
      required: true,
      trigger: 'blur',
      validator: (val: string) => {
        if (formData.value.tunnelType !== 'dynamic' && !val) {
          return { result: false, message: '请输入远程地址', type: 'error' }
        }
        return { result: true, message: '' }
      }
    }
  ],
  remotePort: [
    {
      required: true,
      trigger: 'blur',
      validator: (val: number | null) => {
        if (formData.value.tunnelType !== 'dynamic' && !val) {
          return { result: false, message: '请输入远程端口', type: 'error' }
        }
        return { result: true, message: '' }
      }
    }
  ]
}

// 监听 visible 变化，每次打开时重新填充表单
watch(
  () => props.visible,
  async (newVisible) => {
    console.log('ConfigForm visible changed:', newVisible, 'config:', props.config, 'defaultGroupId:', props.defaultGroupId)
    if (newVisible) {
      // 等待下一个 tick 确保 props 已更新
      await nextTick()
      console.log('ConfigForm filling data with config:', props.config, 'defaultGroupId:', props.defaultGroupId)
      fillFormData(props.config)
      console.log('ConfigForm formData after fill:', formData.value)
      // 不调用 reset，因为它会清除刚填充的数据
    }
  },
  { immediate: true }
)

// 监听 defaultGroupId 变化，当表单打开且为新建模式时更新分组
watch(
  () => props.defaultGroupId,
  (newGroupId) => {
    // 只有在新建模式（没有 config）且表单可见时才更新
    if (props.visible && !props.config && formData.value.groupId !== newGroupId) {
      console.log('ConfigForm defaultGroupId changed:', newGroupId)
      formData.value.groupId = newGroupId || null
    }
  }
)

// 填充表单数据的函数
function fillFormData(config: Config | null | undefined) {
  if (config) {
    // 编辑模式：填充现有数据
    formData.value = {
      name: config.name,
      groupId: config.groupId,
      host: config.host,
      port: config.port,
      username: config.username,
      authType: config.authType,
      password: config.password || '',
      keyPath: config.keyPath || '',
      keyPassphrase: config.keyPassphrase || '',
      tunnelType: config.tunnelType,
      localHost: config.localHost,
      localPort: config.localPort,
      remoteHost: config.remoteHost || 'localhost',
      remotePort: config.remotePort || 80,
      autoReconnect: config.autoReconnect,
      reconnectInterval: config.reconnectInterval,
      isFavorite: config.isFavorite,
      autoStart: config.autoStart
    }
  } else {
    // 新建模式：重置表单，并使用传入的默认分组
    const defaults = defaultFormData()
    defaults.groupId = props.defaultGroupId || null
    formData.value = defaults
  }
}

// 处理提交
async function handleSubmit(): Promise<boolean> {
  const valid = await formRef.value?.validate()
  if (valid !== true) {
    return false
  }

  saving.value = true
  try {
    // 构建请求数据
    const requestData: CreateConfigRequest | UpdateConfigRequest = {
      name: formData.value.name.trim(),
      groupId: formData.value.groupId,
      host: formData.value.host.trim(),
      port: formData.value.port,
      username: formData.value.username.trim(),
      authType: formData.value.authType,
      password: formData.value.authType === 'password' ? formData.value.password : null,
      keyPath: formData.value.authType === 'key' ? formData.value.keyPath.trim() : null,
      keyPassphrase: formData.value.authType === 'key' && formData.value.keyPassphrase
        ? formData.value.keyPassphrase
        : null,
      tunnelType: formData.value.tunnelType,
      localHost: formData.value.localHost.trim(),
      localPort: formData.value.localPort,
      remoteHost: formData.value.tunnelType !== 'dynamic' ? formData.value.remoteHost?.trim() || null : null,
      remotePort: formData.value.tunnelType !== 'dynamic' ? formData.value.remotePort : null,
      autoReconnect: formData.value.autoReconnect,
      reconnectInterval: formData.value.reconnectInterval,
      isFavorite: formData.value.isFavorite,
      autoStart: formData.value.autoStart
    }

    let savedConfig: Config

    if (isEditMode.value && props.config?.id) {
      // 更新配置
      savedConfig = await configStore.updateConfig({
        ...requestData,
        id: props.config.id
      } as UpdateConfigRequest)
      MessagePlugin.success('配置更新成功')
    } else {
      // 创建配置
      savedConfig = await configStore.createConfig(requestData as CreateConfigRequest)
      MessagePlugin.success('配置创建成功')
    }

    emit('saved', savedConfig)
    handleClose()
    return true
  } catch (error) {
    console.error('保存配置失败:', error)
    MessagePlugin.error(isEditMode.value ? '配置更新失败' : '配置创建失败')
    return false
  } finally {
    saving.value = false
  }
}

// 处理关闭
function handleClose(): void {
  emit('update:visible', false)
}
</script>

<style scoped>
:deep(.t-divider) {
  margin: 24px 0 16px;
}

:deep(.t-divider__inner-text) {
  font-size: 14px;
  font-weight: 500;
  color: var(--td-text-color-primary);
}

:deep(.t-form-item) {
  margin-bottom: 20px;
}

:deep(.t-radio-group) {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}
</style>
