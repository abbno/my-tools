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
          <h3 class="section-title">描述</h3>
          <p class="section-text">{{ skill.description || '暂无描述' }}</p>
        </section>

        <t-divider />

        <!-- Source Repository Section -->
        <section class="content-section">
          <h3 class="section-title">来源仓库</h3>
          <t-tag theme="primary" variant="light">{{ repoName }}</t-tag>
        </section>

        <t-divider />

        <!-- Path Section -->
        <section class="content-section">
          <h3 class="section-title">路径</h3>
          <code class="path-code">{{ skill.path }}</code>
        </section>

        <!-- Preview Section -->
        <section class="content-section">
          <h3 class="section-title">SKILL.MD 预览</h3>
          <div v-if="loading" class="preview-placeholder">
            <t-loading text="加载预览中..." />
          </div>
          <div v-else-if="error" class="preview-error">
            <t-alert theme="error" :message="error" />
          </div>
          <div v-else-if="renderedContent" class="preview-content" v-html="renderedContent"></div>
          <div v-else class="preview-placeholder">
            <span class="preview-empty">暂无内容</span>
          </div>
        </section>
      </main>

      <!-- Footer -->
      <footer class="panel-footer">
        <t-button theme="primary" @click="onOpenInEditor">
          <template #icon><BrowseIcon /></template>
          在编辑器中打开
        </t-button>
        <t-button theme="default" variant="outline" @click="onRefresh">
          <template #icon><RefreshIcon /></template>
          刷新
        </t-button>
      </footer>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { CloseIcon, BrowseIcon, RefreshIcon } from 'tdesign-icons-vue-next'
import { marked } from 'marked'
import { readSkillContent } from '@/api/tauri'
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

const skillContent = ref<string>('')
const loading = ref(false)
const error = ref<string | null>(null)

// Rendered markdown HTML
const renderedContent = computed(() => {
  if (!skillContent.value) return ''
  return marked(skillContent.value) as string
})

// Load skill content when skill changes
watch(() => props.skill, async (newSkill) => {
  if (!newSkill) {
    skillContent.value = ''
    return
  }

  loading.value = true
  error.value = null

  try {
    skillContent.value = await readSkillContent(newSkill.repo_id, newSkill.path)
  } catch (e) {
    error.value = String(e)
    skillContent.value = ''
  } finally {
    loading.value = false
  }
}, { immediate: true })

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
    loading.value = true
    error.value = null
    readSkillContent(props.skill.repo_id, props.skill.path)
      .then(content => skillContent.value = content)
      .catch(e => error.value = String(e))
      .finally(() => loading.value = false)
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

.preview-empty {
  color: var(--text-muted);
  font-size: 14px;
}

.preview-error {
  padding: var(--space-md);
}

.preview-content {
  padding: var(--space-lg);
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.6;
  max-height: 400px;
  overflow-y: auto;
}

.preview-content :deep(h1) {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 var(--space-md) 0;
  color: var(--text-primary);
}

.preview-content :deep(h2) {
  font-size: 18px;
  font-weight: 600;
  margin: var(--space-lg) 0 var(--space-sm) 0;
  color: var(--text-primary);
}

.preview-content :deep(h3) {
  font-size: 16px;
  font-weight: 600;
  margin: var(--space-md) 0 var(--space-sm) 0;
  color: var(--text-secondary);
}

.preview-content :deep(p) {
  margin: 0 0 var(--space-md) 0;
}

.preview-content :deep(ul),
.preview-content :deep(ol) {
  margin: 0 0 var(--space-md) 0;
  padding-left: var(--space-lg);
}

.preview-content :deep(li) {
  margin-bottom: var(--space-xs);
}

.preview-content :deep(code) {
  background: var(--bg-elevated);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 13px;
}

.preview-content :deep(pre) {
  background: var(--bg-elevated);
  padding: var(--space-md);
  border-radius: var(--radius-sm);
  overflow-x: auto;
  margin: 0 0 var(--space-md) 0;
}

.preview-content :deep(pre code) {
  background: transparent;
  padding: 0;
}

.preview-content :deep(a) {
  color: var(--accent-primary);
  text-decoration: none;
}

.preview-content :deep(a:hover) {
  text-decoration: underline;
}

.preview-content :deep(blockquote) {
  border-left: 3px solid var(--accent-primary);
  padding-left: var(--space-md);
  margin: 0 0 var(--space-md) 0;
  color: var(--text-muted);
}

.preview-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-subtle);
  margin: var(--space-lg) 0;
}

.preview-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 0 0 var(--space-md) 0;
}

.preview-content :deep(th),
.preview-content :deep(td) {
  border: 1px solid var(--border-subtle);
  padding: var(--space-sm);
  text-align: left;
}

.preview-content :deep(th) {
  background: var(--bg-elevated);
  font-weight: 600;
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