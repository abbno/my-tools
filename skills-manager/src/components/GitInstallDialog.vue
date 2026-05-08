<template>
  <t-dialog
    v-model:visible="visible"
    header="Git 未安装"
    :confirm-btn="null"
    :cancel-btn="null"
    width="500px"
    @close="onClose"
  >
    <t-space direction="vertical" size="large">
      <t-alert theme="warning" message="Git 未检测到，仓库同步功能将无法使用" />

      <div class="install-guide">
        <t-text>请安装 Git 后重新启动应用：</t-text>

        <t-divider />

        <div class="platform-guide">
          <t-text strong>Windows:</t-text>
          <t-code>winget install Git.Git</t-code>
        </div>

        <div class="platform-guide">
          <t-text strong>macOS:</t-text>
          <t-code>brew install git</t-code>
        </div>

        <div class="platform-guide">
          <t-text strong>Linux (Debian/Ubuntu):</t-text>
          <t-code>sudo apt install git</t-code>
        </div>

        <t-divider />

        <t-link href="https://git-scm.com/downloads" target="_blank">
          下载 Git 安装包
        </t-link>
      </div>

      <t-space>
        <t-button variant="outline" @click="onLater">稍后安装</t-button>
        <t-button theme="primary" @click="onRecheck">已安装，重新检测</t-button>
      </t-space>
    </t-space>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { checkGitInstalled } from '@/api/tauri'
import { MessagePlugin } from 'tdesign-vue-next'

const visible = ref(true)
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'installed'): void
}>()

async function onRecheck() {
  try {
    const status = await checkGitInstalled()
    if (status.installed) {
      MessagePlugin.success('Git 已安装: ' + status.version)
      visible.value = false
      emit('installed')
    } else {
      MessagePlugin.warning('Git 仍未检测到，请确认安装后重试')
    }
  } catch (error) {
    MessagePlugin.error('检测失败: ' + error)
  }
}

function onLater() {
  visible.value = false
  emit('close')
}

function onClose() {
  emit('close')
}
</script>

<style scoped>
.install-guide {
  padding: 16px 0;
}

.platform-guide {
  margin: 12px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>