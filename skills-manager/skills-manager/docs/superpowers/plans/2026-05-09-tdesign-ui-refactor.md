# TDesign UI Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor skills-manager from custom CSS to TDesign Vue-next components with sidebar navigation layout.

**Architecture:** Single-page layout with TDesign Layout components (t-header, t-aside, t-content), sidebar navigation (t-menu), skill card grid (t-card), slide-in detail panel, and dialog-based settings/repo addition.

**Tech Stack:** Vue 3, TDesign Vue-next (v1.9.3), Pinia, Vue Router, Tauri

---

## File Structure

### Files to Create
- `src/components/SkillCard.vue` - Skill card component for the grid
- `src/components/SkillDetailPanel.vue` - Slide-in detail panel
- `src/components/SettingsDialog.vue` - Settings dialog component

### Files to Modify
- `src/App.vue` - Rebuild as Layout framework with header/aside/content
- `src/views/Home.vue` - Simplify to content area only (card grid + detail panel)
- `src/views/AddRepoDialog.vue` - Refactor with TDesign components
- `src/views/Settings.vue` - DELETE (merged into SettingsDialog)
- `src/components/GitInstallDialog.vue` - Refactor with TDesign components
- `src/components/ConflictDialog.vue` - Refactor with TDesign components
- `src/router.ts` - Simplify to single route
- `src/styles/global.css` - Create minimal global styles

---

## Task 1: Setup Global Styles and Router

**Files:**
- Create: `src/styles/global.css`
- Modify: `src/router.ts`

- [ ] **Step 1: Create global styles file**

Create minimal global styles to complement TDesign:

```css
/* skills-manager/src/styles/global.css */

/* CSS Variables for custom values not in TDesign */
:root {
  --detail-panel-width: 380px;
  --sidebar-width: 260px;
  --header-height: 64px;
}

/* Ensure full height layout */
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
}

/* Skill card grid layout */
.skill-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  padding: 16px;
}

/* Detail panel slide animation */
.detail-panel {
  position: fixed;
  top: var(--header-height);
  right: 0;
  width: var(--detail-panel-width);
  height: calc(100vh - var(--header-height));
  background: var(--td-bg-color-container);
  border-left: 1px solid var(--td-component-border);
  transition: transform 0.3s ease;
  z-index: 100;
  overflow-y: auto;
}

.detail-panel.hidden {
  transform: translateX(100%);
}

/* Card selected state */
.skill-card.selected {
  border-color: var(--td-brand-color);
  box-shadow: 0 0 0 2px var(--td-brand-color-focus);
}
```

- [ ] **Step 2: Simplify router to single route**

Update router.ts to remove Settings route (settings will be a dialog):

```typescript
// skills-manager/src/router.ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/Home.vue'),
    },
  ],
})

export default router
```

- [ ] **Step 3: Commit changes**

```bash
git add src/styles/global.css src/router.ts
git commit -m "feat: setup global styles and simplify router for TDesign layout"
```

---

## Task 2: Rebuild App.vue as Layout Framework

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Rewrite App.vue with TDesign Layout**

Replace App.vue with Layout framework containing header, aside (sidebar), and content area:

```vue
<!-- skills-manager/src/App.vue -->
<template>
  <t-layout class="app-layout">
    <!-- Header -->
    <t-header class="app-header">
      <div class="header-brand">
        <span class="brand-icon">◈</span>
        <span class="brand-title">Skills Manager</span>
      </div>
      <div class="header-actions">
        <t-input
          v-model="searchQuery"
          placeholder="Search skills..."
          clearable
          class="search-input"
        >
          <template #prefix-icon>
            <search-icon />
          </template>
        </t-input>
        <t-button
          variant="outline"
          shape="circle"
          @click="showSettings = true"
        >
          <template #icon>
            <setting-icon />
          </template>
        </t-button>
      </div>
    </t-header>

    <!-- Main Layout: Sidebar + Content -->
    <t-layout>
      <!-- Sidebar -->
      <t-aside class="app-aside" :width="sidebarWidth">
        <t-menu
          :value="currentRepoId"
          :collapsed="false"
          theme="light"
          @change="onRepoChange"
        >
          <!-- Brand section -->
          <t-menu-item value="brand" :disabled="true">
            <template #icon>
              <span class="menu-brand-icon">◈</span>
            </template>
            Skills Manager
          </t-menu-item>

          <t-divider />

          <!-- Repository group -->
          <t-menu-group title="Repositories">
            <t-menu-item
              v-for="repo in configStore.config?.repositories"
              :key="repo.id"
              :value="repo.id"
            >
              <template #icon>
                <folder-icon />
              </template>
              {{ repo.name }}
              <template #suffix>
                <t-tag
                  v-if="getSyncStatus(repo.id) === 'syncing'"
                  theme="primary"
                  size="small"
                >syncing</t-tag>
                <t-tag
                  v-else-if="getSyncStatus(repo.id) === 'success'"
                  theme="success"
                  size="small"
                >synced</t-tag>
              </template>
            </t-menu-item>
          </t-menu-group>

          <!-- Action buttons -->
          <div class="sidebar-actions">
            <t-button
              variant="outline"
              block
              @click="showAddRepo = true"
            >
              <template #icon>
                <add-icon />
              </template>
              Add Repository
            </t-button>
            <t-button
              variant="outline"
              block
              :loading="syncStore.syncing"
              @click="onSyncAll"
            >
              <template #icon>
                <refresh-icon />
              </template>
              Sync All
            </t-button>
          </div>

          <t-divider />

          <!-- Settings entry -->
          <t-menu-item value="settings" @click="showSettings = true">
            <template #icon>
              <setting-icon />
            </template>
            Settings
          </t-menu-item>
        </t-menu>
      </t-aside>

      <!-- Content area (Home view) -->
      <t-content class="app-content">
        <router-view />
      </t-content>
    </t-layout>
  </t-layout>

  <!-- Dialogs -->
  <AddRepoDialog v-model:visible="showAddRepo" @success="onRepoAdded" />
  <SettingsDialog v-model:visible="showSettings" />
  <GitInstallDialog v-model:visible="showGitDialog" @installed="onGitInstalled" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import { useSyncStore } from '@/stores/sync'
import { checkGitInstalled, syncAllRepositories } from '@/api/tauri'
import AddRepoDialog from '@/views/AddRepoDialog.vue'
import SettingsDialog from '@/components/SettingsDialog.vue'
import GitInstallDialog from '@/components/GitInstallDialog.vue'

// TDesign icons (import from tdesign-icons-vue-next)
import { SearchIcon, SettingIcon, FolderIcon, AddIcon, RefreshIcon } from 'tdesign-icons-vue-next'

const configStore = useConfigStore()
const skillsStore = useSkillsStore()
const syncStore = useSyncStore()

const sidebarWidth = '260px'
const searchQuery = ref('')
const showAddRepo = ref(false)
const showSettings = ref(false)
const showGitDialog = ref(false)

const currentRepoId = computed(() => skillsStore.currentRepoId || '')

function onRepoChange(value: string) {
  if (value === 'settings') return
  skillsStore.setCurrentRepo(value)
}

function getSyncStatus(repoId: string) {
  return syncStore.getProgress(repoId)?.status || 'pending'
}

async function onSyncAll() {
  syncStore.startSync()
  syncStore.clearProgress()
  try {
    await syncAllRepositories()
    for (const repo of configStore.config?.repositories || []) {
      syncStore.endSync(repo.id)
    }
  } catch (error) {
    syncStore.syncing = false
  }
}

function onRepoAdded() {
  showAddRepo.value = false
}

function onGitInstalled() {
  showGitDialog.value = false
}

onMounted(async () => {
  configStore.loadConfig()
  try {
    const status = await checkGitInstalled()
    if (!status.installed) {
      showGitDialog.value = true
    }
  } catch (error) {
    console.error('Failed to check git:', error)
  }
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  background: var(--td-bg-color-container);
  border-bottom: 1px solid var(--td-component-border);
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.brand-icon {
  color: var(--td-brand-color);
  font-size: 20px;
}

.brand-title {
  font-weight: 600;
  font-size: 18px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.search-input {
  width: 240px;
}

.app-aside {
  background: var(--td-bg-color-container);
  border-right: 1px solid var(--td-component-border);
}

.menu-brand-icon {
  color: var(--td-brand-color);
}

.sidebar-actions {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.app-content {
  background: var(--td-bg-color-page);
  overflow-y: auto;
}
</style>
```

- [ ] **Step 2: Install TDesign icons package**

```bash
cd skills-manager && pnpm add tdesign-icons-vue-next
```

- [ ] **Step 3: Commit changes**

```bash
git add src/App.vue package.json pnpm-lock.yaml
git commit -m "feat: rebuild App.vue as TDesign Layout framework with sidebar"
```

---

## Task 3: Create SkillCard Component

**Files:**
- Create: `src/components/SkillCard.vue`

- [ ] **Step 1: Create SkillCard.vue component**

```vue
<!-- skills-manager/src/components/SkillCard.vue -->
<template>
  <t-card
    :class="['skill-card', { selected: selected }]"
    :bordered="true"
    :hover-shadow="true"
    @click="onClick"
  >
    <template #header>
      <div class="card-header">
        <span class="skill-icon">◈</span>
        <span class="skill-name">{{ skill.name }}</span>
      </div>
    </template>

    <div class="skill-description">
      {{ skill.description || 'No description available' }}
    </div>

    <template #footer>
      <div class="skill-meta">
        <t-tag size="small" theme="primary">{{ repoName }}</t-tag>
      </div>
    </template>
  </t-card>
</template>

<script setup lang="ts">
import type { SkillMeta } from '@/stores/skills'

const props = defineProps<{
  skill: SkillMeta
  repoName: string
  selected: boolean
}>()

const emit = defineEmits<{
  (e: 'click', skill: SkillMeta): void
}>()

function onClick() {
  emit('click', props.skill)
}
</script>

<style scoped>
.skill-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.skill-card:hover {
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.skill-icon {
  color: var(--td-brand-color);
}

.skill-name {
  font-weight: 600;
}

.skill-description {
  font-size: 14px;
  color: var(--td-text-color-secondary);
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.skill-meta {
  display: flex;
  align-items: center;
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/components/SkillCard.vue
git commit -m "feat: create SkillCard component with TDesign t-card"
```

---

## Task 4: Create SkillDetailPanel Component

**Files:**
- Create: `src/components/SkillDetailPanel.vue`

- [ ] **Step 1: Create SkillDetailPanel.vue component**

```vue
<!-- skills-manager/src/components/SkillDetailPanel.vue -->
<template>
  <Transition name="slide">
    <div v-if="skill" class="detail-panel">
      <!-- Header -->
      <div class="panel-header">
        <div class="header-title">
          <span class="title-icon">◈</span>
          <span class="title-text">{{ skill.name }}</span>
        </div>
        <t-button
          variant="text"
          shape="circle"
          @click="onClose"
        >
          <template #icon>
            <close-icon />
          </template>
        </t-button>
      </div>

      <!-- Content -->
      <div class="panel-content">
        <!-- Description -->
        <div class="section">
          <div class="section-label">DESCRIPTION</div>
          <div class="section-value">
            {{ skill.description || 'No description available' }}
          </div>
        </div>

        <t-divider />

        <!-- Source Repository -->
        <div class="section">
          <div class="section-label">SOURCE REPOSITORY</div>
          <t-tag theme="primary" size="large">{{ repoName }}</t-tag>
        </div>

        <!-- Path -->
        <div class="section">
          <div class="section-label">PATH</div>
          <code class="path-code">{{ skill.path }}</code>
        </div>

        <t-divider />

        <!-- Preview placeholder -->
        <div class="section">
          <div class="section-label">SKILL.MD PREVIEW</div>
          <div class="preview-placeholder">
            <t-loading text="Loading preview..." />
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <t-button variant="base" block>
          <template #icon>
            <browse-icon />
          </template>
          Open in Editor
        </t-button>
        <t-button variant="outline" block>
          <template #icon>
            <refresh-icon />
          </template>
          Refresh
        </t-button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import type { SkillMeta } from '@/stores/skills'
import { CloseIcon, BrowseIcon, RefreshIcon } from 'tdesign-icons-vue-next'

const props = defineProps<{
  skill: SkillMeta | null
  repoName: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

function onClose() {
  emit('close')
}
</script>

<style scoped>
.detail-panel {
  position: fixed;
  top: 64px;
  right: 0;
  width: 380px;
  height: calc(100vh - 64px);
  background: var(--td-bg-color-container);
  border-left: 1px solid var(--td-component-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 100;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  border-bottom: 1px solid var(--td-component-border);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  color: var(--td-brand-color);
  font-size: 18px;
}

.title-text {
  font-weight: 600;
  font-size: 16px;
}

.panel-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.section {
  margin-bottom: 16px;
}

.section-label {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  margin-bottom: 8px;
  letter-spacing: 0.05em;
}

.section-value {
  font-size: 14px;
  color: var(--td-text-color-primary);
  line-height: 1.6;
}

.path-code {
  display: block;
  background: var(--td-bg-color-secondarycontainer);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--td-text-color-secondary);
  overflow-x: auto;
}

.preview-placeholder {
  background: var(--td-bg-color-secondarycontainer);
  padding: 24px;
  border-radius: 6px;
  text-align: center;
}

.panel-footer {
  padding: 16px;
  border-top: 1px solid var(--td-component-border);
  display: flex;
  gap: 8px;
}

/* Slide transition */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/components/SkillDetailPanel.vue
git commit -m "feat: create SkillDetailPanel component with slide-in animation"
```

---

## Task 5: Rewrite Home.vue as Content Area

**Files:**
- Modify: `src/views/Home.vue`

- [ ] **Step 1: Rewrite Home.vue to use SkillCard and SkillDetailPanel**

```vue
<!-- skills-manager/src/views/Home.vue -->
<template>
  <div class="home-content">
    <!-- Skills grid -->
    <div v-if="filteredSkills.length" class="skill-grid">
      <SkillCard
        v-for="skill in filteredSkills"
        :key="skill.path"
        :skill="skill"
        :repo-name="getRepoName(skill.repo_id)"
        :selected="skillsStore.selectedSkill?.path === skill.path"
        @click="onSkillClick"
      />
    </div>

    <!-- Empty state -->
    <div v-else class="empty-state">
      <t-empty
        description="No skills found"
        :image="EmptyIcon"
      >
        <template #action>
          <t-button
            v-if="!configStore.config?.repositories?.length"
            theme="primary"
            @click="emit('addRepo')"
          >
            Add First Repository
          </t-button>
        </template>
      </t-empty>
    </div>

    <!-- Detail panel -->
    <SkillDetailPanel
      :skill="skillsStore.selectedSkill"
      :repo-name="selectedRepoName"
      @close="onPanelClose"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useConfigStore } from '@/stores/config'
import { useSkillsStore } from '@/stores/skills'
import SkillCard from '@/components/SkillCard.vue'
import SkillDetailPanel from '@/components/SkillDetailPanel.vue'
import { EmptyIcon } from 'tdesign-icons-vue-next'

const configStore = useConfigStore()
const skillsStore = useSkillsStore()

const emit = defineEmits<{
  (e: 'addRepo'): void
}>()

const filteredSkills = computed(() => {
  let skills = skillsStore.skills
  if (skillsStore.currentRepoId) {
    skills = skills.filter(s => s.repo_id === skillsStore.currentRepoId)
  }
  if (skillsStore.searchQuery) {
    const query = skillsStore.searchQuery.toLowerCase()
    skills = skills.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query)
    )
  }
  return skills
})

const selectedRepoName = computed(() => {
  if (!skillsStore.selectedSkill) return ''
  return getRepoName(skillsStore.selectedSkill.repo_id)
})

function getRepoName(repoId: string): string {
  const repo = configStore.config?.repositories.find(r => r.id === repoId)
  return repo?.name || 'Unknown'
}

function onSkillClick(skill: SkillMeta) {
  skillsStore.selectSkill(skill)
}

function onPanelClose() {
  skillsStore.selectSkill(null)
}
</script>

<style scoped>
.home-content {
  padding: 24px;
  min-height: 100%;
}

.skill-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/views/Home.vue
git commit -m "feat: rewrite Home.vue as content area with skill grid and detail panel"
```

---

## Task 6: Create SettingsDialog Component

**Files:**
- Create: `src/components/SettingsDialog.vue`

- [ ] **Step 1: Create SettingsDialog.vue component**

```vue
<!-- skills-manager/src/components/SettingsDialog.vue -->
<template>
  <t-dialog
    v-model:visible="visible"
    header="Settings"
    :width="600"
    :footer="false"
    destroy-on-close
    placement="center"
  >
    <div class="settings-content">
      <!-- Agent Configuration -->
      <div class="settings-section">
        <div class="section-title">Agent Configuration</div>
        <div v-if="!configStore.config?.agents?.length" class="empty-agents">
          <t-empty description="No agents configured" size="small" />
        </div>
        <div v-else class="agent-list">
          <div
            v-for="agent in configStore.config?.agents"
            :key="agent.id"
            class="agent-item"
          >
            <div class="agent-info">
              <span class="agent-name">{{ agent.name }}</span>
              <code class="agent-path">{{ agent.path }}</code>
            </div>
            <t-switch
              :value="agent.enabled"
              @change="(val: boolean) => configStore.updateAgent(agent.id, val)"
            />
          </div>
        </div>
      </div>

      <t-divider />

      <!-- Sync Settings -->
      <div class="settings-section">
        <div class="section-title">Sync Settings</div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Auto Sync</span>
            <span class="setting-desc">Automatically synchronize repositories</span>
          </div>
          <t-switch
            :value="configStore.config?.settings.auto_sync"
            @change="(val: boolean) => configStore.updateSettings({ auto_sync: val })"
          />
        </div>
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Default Sync Interval</span>
            <span class="setting-desc">Frequency for automatic synchronization</span>
          </div>
          <t-select
            :value="configStore.config?.settings.default_sync_interval || 3600"
            @change="(val: number) => configStore.updateSettings({ default_sync_interval: val })"
            style="width: 120px"
          >
            <t-option :value="300" label="5 minutes" />
            <t-option :value="900" label="15 minutes" />
            <t-option :value="1800" label="30 minutes" />
            <t-option :value="3600" label="1 hour" />
            <t-option :value="7200" label="2 hours" />
            <t-option :value="21600" label="6 hours" />
            <t-option :value="43200" label="12 hours" />
            <t-option :value="86400" label="Daily" />
          </t-select>
        </div>
      </div>

      <t-divider />

      <!-- About -->
      <div class="settings-section">
        <div class="section-title">About</div>
        <div class="about-content">
          <div class="about-brand">
            <span class="about-icon">◈</span>
            <span class="about-name">Skills Manager</span>
          </div>
          <p class="about-version">Version 0.1.0</p>
          <p class="about-desc">A curated knowledge management system for AI agents</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="settings-footer">
        <t-button variant="outline" @click="visible = false">Close</t-button>
      </div>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()

const visible = defineModel<boolean>('visible', { default: false })
</script>

<style scoped>
.settings-content {
  padding: 16px 0;
}

.settings-section {
  padding: 16px 24px;
}

.section-title {
  font-weight: 600;
  font-size: 16px;
  margin-bottom: 16px;
}

.agent-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.agent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-secondarycontainer);
  border-radius: 6px;
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-weight: 500;
  display: block;
}

.agent-path {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  background: var(--td-bg-color-container-hover);
  padding: 2px 8px;
  border-radius: 4px;
  display: inline-block;
  margin-top: 4px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--td-bg-color-secondarycontainer);
  border-radius: 6px;
  margin-bottom: 12px;
}

.setting-info {
  flex: 1;
}

.setting-label {
  font-weight: 500;
  display: block;
}

.setting-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  display: block;
  margin-top: 2px;
}

.about-content {
  text-align: center;
}

.about-brand {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 8px;
}

.about-icon {
  color: var(--td-brand-color);
  font-size: 24px;
}

.about-name {
  font-weight: 700;
  font-size: 18px;
}

.about-version {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  margin: 0;
}

.about-desc {
  font-size: 14px;
  color: var(--td-text-color-secondary);
  margin: 8px 0 0 0;
}

.settings-footer {
  padding: 16px 24px;
  display: flex;
  justify-content: flex-end;
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/components/SettingsDialog.vue
git commit -m "feat: create SettingsDialog component with TDesign components"
```

---

## Task 7: Refactor AddRepoDialog with TDesign

**Files:**
- Modify: `src/views/AddRepoDialog.vue`

- [ ] **Step 1: Rewrite AddRepoDialog.vue with TDesign components**

```vue
<!-- skills-manager/src/views/AddRepoDialog.vue -->
<template>
  <t-dialog
    v-model:visible="visible"
    :header="dialogHeader"
    :width="600"
    :footer="false"
    destroy-on-close
    placement="center"
  >
    <!-- Steps indicator -->
    <t-steps :current="currentStep" :items="stepItems" class="dialog-steps" />

    <!-- Step content -->
    <div class="dialog-content">
      <!-- Step 0: Form -->
      <div v-if="currentStep === 0" class="step-form">
        <t-form
          :data="formData"
          :rules="formRules"
          ref="formRef"
          label-align="top"
        >
          <t-row :gutter="16">
            <t-col :span="6">
              <t-form-item label="Repository Name" name="name">
                <t-input
                  v-model="formData.name"
                  placeholder="My Skills Repo"
                  clearable
                />
              </t-form-item>
            </t-col>
            <t-col :span="6">
              <t-form-item label="Repository URL" name="url">
                <t-input
                  v-model="formData.url"
                  placeholder="https://github.com/user/skills-repo"
                  clearable
                />
              </t-form-item>
            </t-col>
          </t-row>

          <t-form-item label="Authentication" name="authType">
            <t-select v-model="formData.authType">
              <t-option value="none" label="None Required" />
              <t-option value="token" label="Token" />
              <t-option value="username-password" label="Username & Password" />
            </t-select>
          </t-form-item>

          <t-form-item
            v-if="formData.authType === 'token'"
            label="Access Token"
            name="token"
          >
            <t-input
              v-model="formData.token"
              type="password"
              placeholder="ghp_xxx or glpat_xxx"
              clearable
            />
          </t-form-item>

          <t-row v-if="formData.authType === 'username-password'" :gutter="16">
            <t-col :span="6">
              <t-form-item label="Username" name="username">
                <t-input v-model="formData.username" placeholder="Username" clearable />
              </t-form-item>
            </t-col>
            <t-col :span="6">
              <t-form-item label="Password" name="password">
                <t-input
                  v-model="formData.password"
                  type="password"
                  placeholder="Password"
                  clearable
                />
              </t-form-item>
            </t-col>
          </t-row>

          <t-form-item label="Sync Interval" name="syncInterval">
            <t-select v-model="formData.syncInterval">
              <t-option :value="300" label="5 minutes" />
              <t-option :value="900" label="15 minutes" />
              <t-option :value="1800" label="30 minutes" />
              <t-option :value="3600" label="1 hour" />
              <t-option :value="7200" label="2 hours" />
              <t-option :value="21600" label="6 hours" />
              <t-option :value="43200" label="12 hours" />
              <t-option :value="86400" label="Daily" />
            </t-select>
          </t-form-item>
        </t-form>
      </div>

      <!-- Step 1: Preview -->
      <div v-if="currentStep === 1" class="step-preview">
        <div v-if="loading" class="loading-state">
          <t-loading text="Fetching skills from repository..." size="large" />
        </div>

        <t-alert
          v-else-if="error"
          theme="error"
          :message="error"
        >
          <template #operation>
            <t-button size="small" @click="retryFetch">Retry</t-button>
          </template>
        </t-alert>

        <div v-else class="preview-content">
          <div class="preview-header">
            <t-tag theme="primary" size="large">
              {{ skills.length }} skills found
            </t-tag>
            <t-button variant="outline" size="small" @click="selectAll">
              {{ selectedSkills.length === skills.length ? 'Deselect All' : 'Select All' }}
            </t-button>
          </div>

          <t-checkbox-group v-model="selectedSkills" class="skills-list">
            <t-checkbox
              v-for="skill in skills"
              :key="skill.path"
              :value="skill.path"
              class="skill-checkbox"
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
      <div v-if="currentStep === 2" class="step-summary">
        <t-descriptions :column="2" bordered>
          <t-descriptions-item label="Name">{{ formData.name }}</t-descriptions-item>
          <t-descriptions-item label="URL">
            <code class="url-code">{{ formData.url }}</code>
          </t-descriptions-item>
          <t-descriptions-item label="Auth">
            <t-tag theme="primary" size="small">
              {{ formData.authType === 'none' ? 'None' : formData.authType === 'token' ? 'Token' : 'User/Pass' }}
            </t-tag>
          </t-descriptions-item>
          <t-descriptions-item label="Sync">{{ syncIntervalLabel }}</t-descriptions-item>
          <t-descriptions-item label="Skills Selected" :span="2">
            <div class="selected-skills">
              <t-tag
                v-for="path in selectedSkills.slice(0, 3)"
                :key="path"
                size="small"
              >
                {{ getSkillName(path) }}
              </t-tag>
              <t-tag v-if="selectedSkills.length > 3" size="small" theme="primary">
                +{{ selectedSkills.length - 3 }} more
              </t-tag>
            </div>
          </t-descriptions-item>
        </t-descriptions>
      </div>
    </div>

    <!-- Footer -->
    <div class="dialog-footer">
      <t-button variant="outline" @click="onCancel">
        {{ currentStep === 0 ? 'Cancel' : 'Back' }}
      </t-button>
      <t-button
        theme="primary"
        :disabled="currentStep === 1 && selectedSkills.length === 0"
        @click="onConfirm"
      >
        {{ currentStep === 2 ? 'Save Repository' : 'Continue' }}
      </t-button>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { useConfigStore } from '@/stores/config'
import { fetchRepoSkills, type SkillMeta, type AuthConfig } from '@/api/tauri'
import { MessagePlugin } from 'tdesign-vue-next'

const configStore = useConfigStore()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{
  (e: 'success'): void
}>()

const currentStep = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)
const skills = ref<SkillMeta[]>([])
const selectedSkills = ref<string[]>([])
const formRef = ref()

const stepItems = [
  { title: 'Repository Info', content: 'Enter repository details' },
  { title: 'Preview Skills', content: 'Select skills to sync' },
  { title: 'Confirm', content: 'Review and save' },
]

const dialogHeader = computed(() => `Add Repository - Step ${currentStep.value + 1} of 3`)

const formData = ref({
  name: '',
  url: '',
  authType: 'none',
  token: '',
  username: '',
  password: '',
  syncInterval: 3600,
})

const formRules = {
  name: [{ required: true, message: 'Name is required', trigger: 'blur' }],
  url: [{ required: true, message: 'URL is required', trigger: 'blur' }],
}

const syncIntervalLabel = computed(() => {
  const labels: Record<number, string> = {
    300: '5 min',
    900: '15 min',
    1800: '30 min',
    3600: '1 hour',
    7200: '2 hours',
    21600: '6 hours',
    43200: '12 hours',
    86400: 'Daily',
  }
  return labels[formData.value.syncInterval] || '1 hour'
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
    if (!valid) return
    currentStep.value = 1
    await fetchSkillsPreview()
  } else if (currentStep.value === 1) {
    if (selectedSkills.value.length === 0) return
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
      sync_interval: formData.value.syncInterval,
      selected_skills: selectedSkills.value,
      last_sync: null,
      enabled: true,
    })
    MessagePlugin.success('Repository added successfully')
    emit('success')
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
</script>

<style scoped>
.dialog-steps {
  padding: 16px 24px;
}

.dialog-content {
  padding: 16px 24px;
  min-height: 300px;
}

.step-preview {
  min-height: 300px;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
}

.preview-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 240px;
  overflow-y: auto;
}

.skill-checkbox {
  display: flex;
  padding: 12px;
  background: var(--td-bg-color-secondarycontainer);
  border-radius: 6px;
}

.skill-info {
  flex: 1;
}

.skill-name {
  font-weight: 500;
  display: block;
}

.skill-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  display: block;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.url-code {
  font-size: 12px;
  background: var(--td-bg-color-secondarycontainer);
  padding: 4px 8px;
  border-radius: 4px;
}

.selected-skills {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.dialog-footer {
  padding: 16px 24px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid var(--td-component-border);
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/views/AddRepoDialog.vue
git commit -m "feat: refactor AddRepoDialog with TDesign components and steps"
```

---

## Task 8: Refactor GitInstallDialog with TDesign

**Files:**
- Modify: `src/components/GitInstallDialog.vue`

- [ ] **Step 1: Rewrite GitInstallDialog.vue with TDesign components**

```vue
<!-- skills-manager/src/components/GitInstallDialog.vue -->
<template>
  <t-dialog
    v-model:visible="visible"
    header="Git Not Installed"
    :width="480"
    :footer="false"
    destroy-on-close
    placement="center"
  >
    <div class="dialog-content">
      <t-alert
        theme="warning"
        :message="Git is not detected. Repository sync features will be unavailable."
      />

      <div class="install-guide">
        <p class="guide-intro">Install Git and restart the application:</p>

        <div class="guide-list">
          <div class="guide-item">
            <t-tag theme="default" size="small">Windows</t-tag>
            <code class="guide-code">winget install Git.Git</code>
          </div>
          <div class="guide-item">
            <t-tag theme="default" size="small">macOS</t-tag>
            <code class="guide-code">brew install git</code>
          </div>
          <div class="guide-item">
            <t-tag theme="default" size="small">Linux</t-tag>
            <code class="guide-code">sudo apt install git</code>
          </div>
        </div>

        <t-link
          href="https://git-scm.com/downloads"
          target="_blank"
          theme="primary"
          hover="color"
        >
          Download Git Installer
        </t-link>
      </div>
    </div>

    <div class="dialog-footer">
      <t-button variant="outline" @click="onLater">Later</t-button>
      <t-button theme="primary" @click="onRecheck">
        <template #icon>
          <refresh-icon />
        </template>
        Recheck
      </t-button>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { checkGitInstalled } from '@/api/tauri'
import { MessagePlugin } from 'tdesign-vue-next'
import { RefreshIcon } from 'tdesign-icons-vue-next'

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{
  (e: 'installed'): void
}>()

async function onRecheck() {
  try {
    const status = await checkGitInstalled()
    if (status.installed) {
      MessagePlugin.success('Git detected successfully')
      visible.value = false
      emit('installed')
    } else {
      MessagePlugin.warning('Git still not detected')
    }
  } catch (error) {
    MessagePlugin.error('Check failed')
  }
}

function onLater() {
  visible.value = false
}
</script>

<style scoped>
.dialog-content {
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.install-guide {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.guide-intro {
  color: var(--td-text-color-secondary);
  margin: 0;
}

.guide-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.guide-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.guide-code {
  background: var(--td-bg-color-secondarycontainer);
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
}

.dialog-footer {
  padding: 16px 24px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid var(--td-component-border);
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/components/GitInstallDialog.vue
git commit -m "feat: refactor GitInstallDialog with TDesign components"
```

---

## Task 9: Refactor ConflictDialog with TDesign

**Files:**
- Modify: `src/components/ConflictDialog.vue`

- [ ] **Step 1: Rewrite ConflictDialog.vue with TDesign components**

```vue
<!-- skills-manager/src/components/ConflictDialog.vue -->
<template>
  <t-dialog
    v-model:visible="visible"
    header="Skill Conflict"
    :width="440"
    :footer="false"
    destroy-on-close
    placement="center"
  >
    <div class="dialog-content">
      <t-alert
        theme="warning"
        :message="`Found duplicate skill: ${conflict.skillName}`"
      />

      <p class="conflict-intro">This skill already exists. Choose how to handle it:</p>

      <t-radio-group v-model="selectedOption" class="options-group">
        <t-radio
          value="existing"
          class="option-item"
        >
          <div class="option-content">
            <span class="option-label">Keep existing ({{ conflict.existingRepoName }})</span>
            <span class="option-desc">Do not replace, keep the current version</span>
          </div>
        </t-radio>

        <t-radio
          value="new"
          class="option-item"
        >
          <div class="option-content">
            <span class="option-label">Use new version ({{ conflict.newRepoName }})</span>
            <span class="option-desc">Replace with the new repository version</span>
          </div>
        </t-radio>

        <t-radio
          value="skip"
          class="option-item"
        >
          <div class="option-content">
            <span class="option-label">Skip this skill</span>
            <span class="option-desc">Do not deploy this skill</span>
          </div>
        </t-radio>
      </t-radio-group>
    </div>

    <div class="dialog-footer">
      <t-button variant="outline" @click="onClose">Cancel</t-button>
      <t-button theme="primary" @click="onConfirm">Confirm</t-button>
    </div>
  </t-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface ConflictInfo {
  skillName: string
  existingRepoId: string
  existingRepoName: string
  newRepoId: string
  newRepoName: string
}

const props = defineProps<{
  conflict: ConflictInfo
}>()

const visible = defineModel<boolean>('visible', { default: false })
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', choice: 'existing' | 'new' | 'skip'): void
}>()

const selectedOption = ref<'existing' | 'new' | 'skip'>('existing')

function onClose() {
  visible.value = false
  emit('close')
}

function onConfirm() {
  visible.value = false
  emit('select', selectedOption.value)
}
</script>

<style scoped>
.dialog-content {
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.conflict-intro {
  color: var(--td-text-color-secondary);
  margin: 0;
}

.options-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.option-item {
  display: flex;
  padding: 12px;
  background: var(--td-bg-color-secondarycontainer);
  border-radius: 6px;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.option-item.t-radio--checked {
  border-color: var(--td-brand-color);
  background: var(--td-brand-color-hover);
}

.option-content {
  flex: 1;
  margin-left: 8px;
}

.option-label {
  font-weight: 500;
  display: block;
}

.option-desc {
  font-size: 12px;
  color: var(--td-text-color-placeholder);
  display: block;
  margin-top: 4px;
}

.dialog-footer {
  padding: 16px 24px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid var(--td-component-border);
}
</style>
```

- [ ] **Step 2: Commit changes**

```bash
git add src/components/ConflictDialog.vue
git commit -m "feat: refactor ConflictDialog with TDesign components"
```

---

## Task 10: Delete Settings.vue and Update Imports

**Files:**
- Delete: `src/views/Settings.vue`

- [ ] **Step 1: Delete Settings.vue file**

Settings functionality is now in SettingsDialog.vue:

```bash
rm src/views/Settings.vue
git add -A
git commit -m "refactor: delete Settings.vue (merged into SettingsDialog)"
```

- [ ] **Step 2: Verify no remaining imports of Settings.vue**

```bash
grep -r "Settings.vue" src/ && echo "Found references - fix them" || echo "No references found"
```

---

## Task 11: Test and Finalize

**Files:**
- Verify all components work

- [ ] **Step 1: Start development server**

```bash
cd skills-manager && pnpm tauri:dev
```

- [ ] **Step 2: Test core functionality**

Manually verify:
1. Layout displays correctly (header + sidebar + content)
2. Repository list shows in sidebar
3. Skill cards display in grid
4. Detail panel slides in when clicking a card
5. Settings dialog opens from sidebar button
6. Add repo dialog works through 3 steps
7. Git install dialog shows when git missing
8. Search filters skills correctly

- [ ] **Step 3: Final commit for any remaining fixes**

```bash
git add -A
git commit -m "fix: final adjustments for TDesign UI refactor"
```

---

## Spec Coverage Verification

| Spec Section | Task |
|--------------|------|
| 整体布局架构 | Task 2 |
| 侧边栏设计 | Task 2 |
| 技能卡片网格 | Task 3, Task 5 |
| 右侧详情面板 | Task 4, Task 5 |
| 添加仓库对话框 | Task 7 |
| 设置对话框 | Task 6 |
| Git 安装对话框 | Task 8 |
| 冲突对话框 | Task 9 |
| 文件结构规划 | Tasks 1-10 |
| 全局样式 | Task 1 |