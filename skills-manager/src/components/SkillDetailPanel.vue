<template>
  <Transition name="slide">
    <div v-if="skill" class="detail-panel">
      <!-- Header -->
      <header class="panel-header">
        <h2 class="panel-title">{{ skill.name }}</h2>
        <button class="btn-close" @click="onClose">
          <CloseIcon />
        </button>
      </header>

      <!-- Content -->
      <main class="panel-content">
        <!-- Description Section -->
        <section class="content-section">
          <h3 class="section-title">DESCRIPTION</h3>
          <p class="section-text">{{ skill.description }}</p>
        </section>

        <t-divider />

        <!-- Source Repository Section -->
        <section class="content-section">
          <h3 class="section-title">SOURCE REPOSITORY</h3>
          <t-tag theme="primary" variant="light">{{ repoName }}</t-tag>
        </section>

        <t-divider />

        <!-- Path Section -->
        <section class="content-section">
          <h3 class="section-title">PATH</h3>
          <code class="path-code">{{ skill.path }}</code>
        </section>

        <!-- Preview Placeholder -->
        <section class="content-section">
          <h3 class="section-title">SKILL.MD PREVIEW</h3>
          <div class="preview-placeholder">
            <t-loading text="Loading preview..." />
          </div>
        </section>
      </main>

      <!-- Footer -->
      <footer class="panel-footer">
        <t-button theme="primary" @click="onOpenInEditor">
          <template #icon><BrowseIcon /></template>
          Open in Editor
        </t-button>
        <t-button theme="default" variant="outline" @click="onRefresh">
          <template #icon><RefreshIcon /></template>
          Refresh
        </t-button>
      </footer>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { CloseIcon, BrowseIcon, RefreshIcon } from 'tdesign-icons-vue-next'
import type { SkillMeta } from '@/stores/skills'

const props = defineProps<{
  skill: SkillMeta | null
  repoName: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'openInEditor', skill: SkillMeta): void
  (e: 'refresh', skill: SkillMeta): void
}>()

function onClose() {
  emit('close')
}

function onOpenInEditor() {
  if (props.skill) {
    emit('openInEditor', props.skill)
  }
}

function onRefresh() {
  if (props.skill) {
    emit('refresh', props.skill)
  }
}
</script>

<style scoped>
.detail-panel {
  position: fixed;
  top: 0;
  right: 0;
  width: 380px;
  height: 100vh;
  background: var(--bg-card);
  border-left: 1px solid var(--border-medium);
  box-shadow: var(--shadow-elevated);
  display: flex;
  flex-direction: column;
  z-index: 100;
}

.panel-header {
  padding: var(--space-lg);
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-subtle);
}

.panel-title {
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-close {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: var(--space-sm);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
}

.btn-close:hover {
  color: var(--text-primary);
  background: var(--bg-elevated);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg);
}

.content-section {
  margin-bottom: var(--space-xl);
}

.content-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-family: var(--font-body);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 var(--space-sm) 0;
}

.section-text {
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0;
}

.path-code {
  display: block;
  padding: var(--space-sm) var(--space-md);
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
}

.preview-placeholder {
  padding: var(--space-xl);
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
}

.panel-footer {
  padding: var(--space-lg);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  gap: var(--space-md);
}

/* Slide transition */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

.slide-enter-to,
.slide-leave-from {
  transform: translateX(0);
  opacity: 1;
}
</style>