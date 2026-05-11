# Skills Manager 样式迁移实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 ssh-tunnel-manager 的视觉样式迁移到 skills-manager，保持浅色主题风格一致性。

**Architecture:** 在 `src/styles/global.css` 添加全局样式（滚动条、字体、用户选择），在组件 scoped CSS 中调整组件级样式，使用 #fff 背景、#e7e7e7 边框的浅色主题。

**Tech Stack:** Vue 3 Composition API, TDesign Vue-next, CSS scoped styles

---

## 文件结构

**修改的文件：**
- `src/styles/global.css` - 添加滚动条、字体、用户选择样式
- `src/App.vue` - 调整 header 和 sidebar 的 scoped CSS
- `src/components/SkillCard.vue` - 调整卡片视觉样式

---

### Task 1: 全局样式迁移

**Files:**
- Modify: `src/styles/global.css:17-21`

- [ ] **Step 1: 添加字体样式**

修改 `src/styles/global.css` 的 `html, body, #app` 部分，添加字体和用户选择样式：

```css
/* 基础布局 */
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 禁止选中文本 */
body {
  user-select: none;
}
```

- [ ] **Step 2: 添加滚动条样式**

在 `src/styles/global.css` 文件末尾添加滚动条样式：

```css
/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #dcdcdc;
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: #c0c0c0;
}
```

- [ ] **Step 3: Commit**

```bash
git add src/styles/global.css
git commit -m "style: add global scrollbar and font styles"
```

---

### Task 2: Header 工具栏样式调整

**Files:**
- Modify: `src/App.vue:243-247` (app-header CSS)
- Modify: `src/App.vue:249-255` (header-content CSS)

- [ ] **Step 1: 修改 app-header 样式**

将 `src/App.vue` scoped CSS 中的 `.app-header` 改为使用固定颜色：

```css
.app-header {
  height: 64px;
  background: #fff;
  border-bottom: 1px solid #e7e7e7;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/App.vue
git commit -m "style: update header to white background"
```

---

### Task 3: SkillCard 卡片样式调整

**Files:**
- Modify: `src/components/SkillCard.vue:52-59`

- [ ] **Step 1: 修改卡片基础样式**

将 `src/components/SkillCard.vue` scoped CSS 中的 `.skill-card` 样式改为：

```css
.skill-card {
  background: #fff;
  border: 1px solid #e7e7e7;
  border-radius: 6px;
  cursor: pointer;
  transition: box-shadow 0.2s ease;
}

.skill-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
```

注意：删除原来的 `transform: translateY(-2px)` hover 效果，改为 box-shadow。

- [ ] **Step 2: Commit**

```bash
git add src/components/SkillCard.vue
git commit -m "style: update SkillCard to white background with hover shadow"
```

---

### Task 4: Sidebar 侧边栏样式调整

**Files:**
- Modify: `src/App.vue:289-295` (app-aside CSS)

- [ ] **Step 1: 修改 app-aside 样式**

将 `src/App.vue` scoped CSS 中的 `.app-aside` 改为使用固定颜色：

```css
.app-aside {
  width: 260px;
  background: #fff;
  border-right: 1px solid #e7e7e7;
  display: flex;
  flex-direction: column;
}
```

- [ ] **Step 2: 修改 aside-header 样式**

将 `.aside-header` 的 border-bottom 改为固定颜色：

```css
.aside-header {
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e7e7e7;
}
```

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "style: update sidebar to white background"
```

---

### Task 5: CSS 变量清理

**Files:**
- Modify: `src/styles/global.css:4-14`

- [ ] **Step 1: 清理不再使用的 CSS 变量**

从 `src/styles/global.css` 的 `:root` 中删除以下不再使用的变量（已被固定颜色替代）：

```css
/* 删除这些变量 */
--bg-primary: var(--td-bg-color-container);
--border-subtle: var(--td-component-border);
```

保留以下仍在使用的变量：

```css
:root {
  --detail-panel-width: 380px;
  --sidebar-width: 260px;
  --header-height: 64px;

  /* 仅保留仍在使用的 */
  --bg-deep: var(--td-bg-color-page);
  --bg-secondary: var(--td-bg-color-secondarycontainer);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles/global.css
git commit -m "style: remove unused CSS variables after style migration"
```