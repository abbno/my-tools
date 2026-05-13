# UI Bug Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix four UI bugs: scroll issue, repository deletion, skill detail panel, and skill selection display.

**Architecture:** CSS fixes for scroll, add delete button with popconfirm, new Tauri command for reading skill content, markdown rendering in detail panel, selection badge on skill cards.

**Tech Stack:** Tauri commands, Vue 3, TDesign components, marked for markdown

---

## File Structure

**Create:**
- `src-tauri/src/commands/skill.rs` - read_skill_content command

**Modify:**
- `src/App.vue` - add delete button to repo menu items, fix scroll CSS
- `src/views/Home.vue` - fix scroll CSS
- `src/components/SkillDetailPanel.vue` - markdown rendering
- `src/components/SkillCard.vue` - add selection badge
- `src/api/tauri.ts` - add readSkillContent API
- `src/stores/skills.ts` - already has updateSkillIsSelected
- `src-tauri/src/commands/mod.rs` - export skill commands
- `src-tauri/src/lib.rs` - register new command

---

### Task 1: Fix Scroll CSS

**Files:**
- Modify: `src/App.vue:379` (app-content CSS)
- Modify: `src/views/Home.vue:107-110` (home-content CSS)

- [ ] **Step 1: Modify App.vue CSS**

Change `.app-content` from `overflow: hidden` to `overflow-y: auto`:

```css
/* Content */
.app-content {
  background: var(--bg-deep);
  overflow-y: auto;
}
```

- [ ] **Step 2: Modify Home.vue CSS**

Add `height: 100%` and `overflow-y: auto` to `.home-content`:

```css
.home-content {
  padding: 24px;
  height: 100%;
  overflow-y: auto;
}
```

- [ ] **Step 3: Verify scroll works**

Run: `pnpm tauri dev`
Test: Scroll the skill list when there are many skills

- [ ] **Step 4: Commit**

```bash
git add src/App.vue src/views/Home.vue
git commit -m "fix: enable scrolling for skill card list"
```

---

### Task 2: Add Repository Delete Button

**Files:**
- Modify: `src/App.vue:70-100` (repo menu items template)
- Modify: `src/App.vue` (add delete handler and state)
- Modify: `src/stores/skills.ts` (add removeSkillsByRepo function)

- [ ] **Step 1: Add delete state and handler to App.vue script**

Add to script section after existing functions:

```typescript
// Delete repository state
const showDeleteConfirm = ref(false)
const repoToDelete = ref<string | null>(null)

function confirmDeleteRepo(repoId: string) {
  repoToDelete.value = repoId
  showDeleteConfirm.value = true
}

async function onDeleteRepo() {
  if (!repoToDelete.value) return
  
  // Remove skills from this repo
  skillsStore.skills = skillsStore.skills.filter(s => s.repo_id !== repoToDelete.value)
  
  // Remove repository
  configStore.removeRepository(repoToDelete.value)
  
  // If deleted repo was selected, select another
  if (skillsStore.currentRepoId === repoToDelete.value) {
    const firstRepo = configStore.config?.repositories?.[0]?.id
    skillsStore.setCurrentRepo(firstRepo || null)
  }
  
  showDeleteConfirm.value = false
  repoToDelete.value = null
}
```

- [ ] **Step 2: Add delete button to menu item template**

Modify the t-menu-item template in App.vue (around line 70-100). Add a delete icon button in the suffix template alongside the sync status:

```vue
<t-menu-item
  v-for="repo in configStore.config?.repositories || []"
  :key="repo.id"
  :value="repo.id"
>
  <template #icon>
    <folder-icon />
  </template>
  <span class="repo-name">{{ repo.name }}</span>
  <template #suffix>
    <div class="repo-suffix">
      <span
        v-if="getSyncStatus(repo.id) === 'syncing'"
        class="status-syncing"
      >
        <t-loading size="small" />
      </span>
      <span
        v-else-if="getSyncStatus(repo.id) === 'success'"
        class="status-success"
      >
        ✓
      </span>
      <span
        v-else-if="getSyncStatus(repo.id) === 'error'"
        class="status-error"
      >
        ✗
      </span>
      <t-button
        size="small"
        shape="circle"
        variant="text"
        class="delete-btn"
        @click.stop="confirmDeleteRepo(repo.id)"
      >
        <template #icon>
          <close-icon />
        </template>
      </t-button>
    </div>
  </template>
</t-menu-item>
```

- [ ] **Step 3: Add CloseIcon import**

Add to the imports in App.vue script:

```typescript
import {
  SearchIcon,
  SettingIcon,
  FolderIcon,
  AddIcon,
  RefreshIcon,
  CloseIcon,
} from 'tdesign-icons-vue-next'
```

- [ ] **Step 4: Add CSS for delete button**

Add to App.vue style section:

```css
.repo-suffix {
  display: flex;
  align-items: center;
  gap: 8px;
}

.delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
}

.t-menu-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  color: var(--td-error-color);
}
```

- [ ] **Step 5: Add delete confirm dialog to template**

Add after the AddRepoDialog in template:

```vue
<!-- Delete confirm dialog -->
<t-dialog
  v-model:visible="showDeleteConfirm"
  header="确认删除"
  body="删除仓库将同时移除该仓库下的所有技能数据，是否继续？"
  :confirm-btn="{ content: '删除', theme: 'danger' }"
  @confirm="onDeleteRepo"
/>
```

- [ ] **Step 6: Verify delete works**

Run: `pnpm tauri dev`
Test: Hover over repo menu item, see delete button, click and confirm delete

- [ ] **Step 7: Commit**

```bash
git add src/App.vue
git commit -m "feat: add repository delete button with confirmation"
```

---

### Task 3: Add read_skill_content Tauri Command

**Files:**
- Create: `src-tauri/src/commands/skill.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create skill.rs command file**

```rust
// src-tauri/src/commands/skill.rs
use std::fs;
use std::path::PathBuf;
use crate::git::get_repo_path;

#[tauri::command]
pub fn read_skill_content(repo_id: String, skill_path: String) -> Result<String, String> {
    let repo_path = get_repo_path(&repo_id)?;
    let skill_md_path = repo_path.join(&skill_path).join("SKILL.md");
    
    if !skill_md_path.exists() {
        return Err(format!("SKILL.md not found at {}", skill_md_path.to_string_lossy()));
    }
    
    fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))
}
```

- [ ] **Step 2: Export skill module in commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod system;
pub mod config;
pub mod repo;
pub mod skill;

pub use system::*;
pub use config::*;
pub use repo::*;
pub use skill::*;
```

- [ ] **Step 3: Register command in lib.rs**

Add `commands::read_skill_content` to invoke_handler:

```rust
.invoke_handler(tauri::generate_handler![
    commands::check_git_installed,
    commands::get_system_info,
    commands::read_config,
    commands::save_config,
    commands::fetch_branches,
    commands::fetch_repo_skills,
    commands::sync_repository,
    commands::deploy_skill,
    commands::undeploy_skill,
    commands::get_skills,
    commands::update_skill_selection,
    commands::clear_repo_skills,
    commands::read_skill_content,  // ADD THIS
    symlink::create_skill_symlink,
    symlink::remove_skill_symlink,
    symlink::check_symlinks,
    scheduler::sync_all_repositories,
    scheduler::get_sync_status,
])
```

- [ ] **Step 4: Verify command builds**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/skill.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add read_skill_content Tauri command"
```

---

### Task 4: Add Frontend API for readSkillContent

**Files:**
- Modify: `src/api/tauri.ts`

- [ ] **Step 1: Add readSkillContent function**

Add after existing skill API functions (around line 99):

```typescript
// Read skill content API
export async function readSkillContent(repoId: string, skillPath: string): Promise<string> {
  return invoke<string>('read_skill_content', { repoId, skillPath })
}
```

- [ ] **Step 2: Verify frontend builds**

Run: `pnpm build`
Expected: Builds successfully

- [ ] **Step 3: Commit**

```bash
git add src/api/tauri.ts
git commit -m "feat: add readSkillContent frontend API"
```

---

### Task 5: Implement SkillDetailPanel Markdown Rendering

**Files:**
- Modify: `src/components/SkillDetailPanel.vue`

- [ ] **Step 1: Add marked dependency**

Run: `pnpm add marked`

- [ ] **Step 2: Modify SkillDetailPanel.vue script**

Replace the entire script section with:

```typescript
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
```

- [ ] **Step 3: Modify template preview section**

Replace the preview placeholder section (lines 37-42) with:

```vue
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
```

- [ ] **Step 4: Add CSS for markdown content**

Add to style section:

```css
.preview-content {
  padding: var(--space-md);
  background: var(--bg-secondary);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  font-size: 14px;
  line-height: 1.6;
}

.preview-content h1,
.preview-content h2,
.preview-content h3 {
  margin-top: 16px;
  margin-bottom: 8px;
}

.preview-content h1 { font-size: 20px; }
.preview-content h2 { font-size: 16px; }
.preview-content h3 { font-size: 14px; }

.preview-content p {
  margin-bottom: 12px;
}

.preview-content code {
  background: var(--bg-elevated);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.preview-content pre {
  background: var(--bg-elevated);
  padding: 12px;
  border-radius: var(--radius-sm);
  overflow-x: auto;
}

.preview-content pre code {
  background: none;
  padding: 0;
}

.preview-content ul,
.preview-content ol {
  padding-left: 20px;
  margin-bottom: 12px;
}

.preview-content li {
  margin-bottom: 4px;
}

.preview-error {
  padding: var(--space-md);
}

.preview-empty {
  color: var(--text-muted);
}
```

- [ ] **Step 5: Verify panel works**

Run: `pnpm tauri dev`
Test: Click a skill card, see SKILL.md content rendered in detail panel

- [ ] **Step 6: Commit**

```bash
git add src/components/SkillDetailPanel.vue package.json pnpm-lock.yaml
git commit -m "feat: render SKILL.md content in detail panel"
```

---

### Task 6: Add Selection Badge to SkillCard

**Files:**
- Modify: `src/components/SkillCard.vue`
- Modify: `src/views/Home.vue` (update click handler)

- [ ] **Step 1: Modify SkillCard.vue template**

Replace entire template with:

```vue
<template>
  <t-card
    :class="['skill-card', { selected: selected, 'is-selected-skill': skill.is_selected }]"
    :bordered="true"
    :hover-shadow="true"
    role="button"
    :aria-selected="selected"
    tabindex="0"
    @click="onClick"
    @keydown.enter="onClick"
    @keydown.space.prevent="onClick"
  >
    <!-- Selection badge -->
    <div v-if="skill.is_selected" class="selection-badge">
      <t-tag theme="primary" size="small" variant="light">已选</t-tag>
    </div>
    
    <template #header>
      <div class="card-header">
        <span class="skill-icon">◈</span>
        <span class="skill-name">{{ skill.name }}</span>
      </div>
    </template>

    <div class="skill-description">
      {{ skill.description || '暂无描述' }}
    </div>

    <template #footer>
      <div class="skill-meta">
        <t-tag size="small" theme="primary">{{ repoName }}</t-tag>
      </div>
    </template>
  </t-card>
</template>
```

- [ ] **Step 2: Add CSS for selection badge**

Add to style section:

```css
.skill-card {
  position: relative;
  background: #fff;
  border: 1px solid #e7e7e7;
  border-radius: 6px;
  cursor: pointer;
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
}

.skill-card.is-selected-skill {
  border-color: var(--td-brand-color);
}

.selection-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 1;
}
```

- [ ] **Step 3: Modify Home.vue click handler**

Update onSkillClick to toggle selection:

```typescript
import { updateSkillSelection } from '@/api/tauri'

async function onSkillClick(skill: SkillMeta) {
  // Toggle selection
  const newSelected = !skill.is_selected
  
  try {
    await updateSkillSelection(skill.id, newSelected)
    skillsStore.updateSkillIsSelected(skill.id, newSelected)
  } catch (e) {
    console.error('Failed to update selection:', e)
  }
  
  // Also set as selected for detail panel
  skillsStore.selectSkill(skill)
}
```

- [ ] **Step 4: Verify badge works**

Run: `pnpm tauri dev`
Test: Click skill card, see badge appear/disappear, state persists

- [ ] **Step 5: Commit**

```bash
git add src/components/SkillCard.vue src/views/Home.vue
git commit -m "feat: add selection badge and toggle on skill cards"
```

---

### Task 7: Integration Test

- [ ] **Step 1: Build full application**

Run: `pnpm tauri build`
Expected: Builds successfully

- [ ] **Step 2: Test all fixes**

1. Scroll skill list when there are many skills
2. Hover over repo item, click delete button, confirm deletion
3. Click skill card, see SKILL.md rendered in detail panel
4. Click skill card, see "已选" badge appear/disappear
5. Restart app, verify selection state persisted

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete UI bug fixes - scroll, delete, detail panel, selection"
```