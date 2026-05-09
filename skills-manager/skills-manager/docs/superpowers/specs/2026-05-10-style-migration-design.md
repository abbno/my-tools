# Skills Manager 样式迁移设计规范

## 目标

将 ssh-tunnel-manager 的视觉样式迁移到 skills-manager，保持浅色主题风格的一致性。

## 背景

两个项目都是 Tauri + TDesign Vue-next 技术栈的桌面应用。ssh-tunnel-manager 已有成熟的浅色主题视觉样式，skills-manager 刚完成 TDesign UI 重构，需要调整视觉风格以保持一致性。

## 架构

采用全局主题迁移方案：
- 在 `src/styles/global.css` 中添加全局样式（滚动条、字体、用户选择）
- 在各组件的 scoped CSS 中调整组件级样式
- 使用 TDesign CSS 变量作为基础，添加自定义样式覆盖

## 技术栈

- Vue 3 Composition API + `<script setup>`
- TDesign Vue-next 组件库
- CSS scoped 样式
- TDesign CSS 变量（`--td-*`）

---

## 第一部分：全局样式

### 文件修改
- **修改**: `src/styles/global.css`

### 内容

添加以下全局样式：

```css
/* 基础布局 */
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* 用户选择 */
body {
  user-select: none;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-thumb {
  background: #dcdcdc;
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: #c0c0c0;
}

::-webkit-scrollbar-track {
  background: transparent;
}
```

---

## 第二部分：工具栏样式

### 文件修改
- **修改**: `src/App.vue` 的 scoped CSS

### 变更内容

调整 header-actions 区域的布局和样式：

1. **布局结构**：
   - 左侧：品牌标识
   - 右侧：搜索框 + 设置按钮
   - 保持现有的 flex 布局

2. **样式调整**：
```css
.header-content {
  height: 100%;
  padding: 0 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #fff;
  border-bottom: 1px solid #e7e7e7;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.search-input {
  width: 240px;
}
```

---

## 第三部分：SkillCard 样式

### 文件修改
- **修改**: `src/components/SkillCard.vue` 的 scoped CSS

### 变更内容

保持组件结构不变，调整视觉样式：

1. **卡片样式**：
```css
.skill-card {
  background: #fff;
  border: 1px solid #e7e7e7;
  border-radius: 6px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.skill-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
```

2. **保持现有结构**：
   - 卡片头部（技能名称 + 类型标签）
   - 卡片内容（描述文本）
   - 卡片底部（仓库来源）
   - 无状态指示器

---

## 第四部分：侧边栏样式

### 文件修改
- **修改**: `src/App.vue` 中的侧边栏 scoped CSS

### 变更内容

1. **仓库列表项样式**：
```css
.repo-menu-item {
  display: flex;
  justify-content: space-between;
  padding: 10px 16px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.repo-menu-item:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.repo-menu-item.active {
  background-color: var(--td-brand-color-light);
}
```

2. **仓库数量徽章**：
```css
.repo-count {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
  background-color: var(--td-brand-color-light);
  color: var(--td-brand-color);
}
```

3. **侧边栏容器样式**：
```css
.app-aside {
  width: 260px;
  background: #fff;
  border-right: 1px solid #e7e7e7;
  display: flex;
  flex-direction: column;
}
```

---

## 测试验证

- 启动开发服务器验证样式效果
- 测试滚动条在各个区域的显示
- 测试卡片 hover 效果
- 测试侧边栏 hover 和选中状态
- 测试搜索框和按钮布局

## 不包含的内容

- 功能逻辑变更
- 组件结构重构
- 深色主题支持
- 新增状态指示器