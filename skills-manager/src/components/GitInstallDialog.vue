<template>
  <t-dialog
    v-model:visible="visible"
    header="Git Not Installed"
    :width="480"
    :close-on-overlay-click="false"
    :close-on-esc-keydown="false"
    @close="onClose"
  >
    <t-alert theme="warning" message="Git is not detected. Repository sync features will be unavailable." />

    <div class="install-guide">
      <p class="guide-intro">Install Git and restart the application:</p>

      <div class="guide-grid">
        <div class="guide-item">
          <t-tag variant="light" theme="default">Windows</t-tag>
          <code class="guide-code">winget install Git.Git</code>
        </div>
        <div class="guide-item">
          <t-tag variant="light" theme="default">macOS</t-tag>
          <code class="guide-code">brew install git</code>
        </div>
        <div class="guide-item">
          <t-tag variant="light" theme="default">Linux</t-tag>
          <code class="guide-code">sudo apt install git</code>
        </div>
      </div>

      <t-link href="https://git-scm.com/downloads" target="_blank" hover="color">
        Download Git Installer
      </t-link>
    </div>

    <template #footer>
      <t-button variant="outline" @click="onLater">Later</t-button>
      <t-button theme="primary" @click="onRecheck" :loading="checking">
        <template #icon><RefreshIcon /></template>
        Recheck
      </t-button>
    </template>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { MessagePlugin } from 'tdesign-vue-next'
import { RefreshIcon } from 'tdesign-icons-vue-next'
import { checkGitInstalled } from '@/api/tauri'

const visible = ref(true)
const checking = ref(false)
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'installed'): void
}>()

async function onRecheck() {
  checking.value = true
  try {
    const status = await checkGitInstalled()
    if (status.installed) {
      visible.value = false
      emit('installed')
      MessagePlugin.success('Git is now installed!')
    } else {
      MessagePlugin.warning('Git is still not detected. Please install Git first.')
    }
  } catch (error) {
    console.error('Check failed:', error)
    MessagePlugin.error('Failed to check Git installation.')
  } finally {
    checking.value = false
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
  display: flex;
  flex-direction: column;
  gap: var(--td-comp-margin-l);
  margin-top: var(--td-comp-margin-l);
}

.guide-intro {
  font-size: var(--td-font-size-body-medium);
  color: var(--td-text-color-secondary);
}

.guide-grid {
  display: flex;
  flex-direction: column;
  gap: var(--td-comp-margin-m);
}

.guide-item {
  display: flex;
  align-items: center;
  gap: var(--td-comp-margin-l);
}

.guide-code {
  font-family: var(--td-font-family-mono);
  font-size: var(--td-font-size-body-small);
  color: var(--td-text-color-primary);
  background: var(--td-bg-color-container);
  padding: var(--td-comp-paddingTB-xs) var(--td-comp-paddingLR-s);
  border-radius: var(--td-radius-default);
}
</style>