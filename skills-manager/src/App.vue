<template>
  <router-view />
  <GitInstallDialog
    v-if="showGitDialog"
    @close="showGitDialog = false"
    @installed="showGitDialog = false"
  />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import GitInstallDialog from '@/components/GitInstallDialog.vue'
import { checkGitInstalled } from '@/api/tauri'

const showGitDialog = ref(false)

onMounted(async () => {
  try {
    const status = await checkGitInstalled()
    if (!status.installed) {
      showGitDialog.value = true
    }
  } catch (error) {
    console.error('Failed to check git installation:', error)
  }
})
</script>

<style>
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
}
</style>