# Skills Manager UI Bug Fixes Design

## 背景

用户反馈四个 UI 问题需要修复：
1. 已添加的仓库无法删除
2. 点击技能详情一直显示"加载预览中"，样式混乱
3. 技能卡片列表无法滚动
4. 技能卡片列表显示全部，未标记选中状态

## 问题分析

| 问题 | 根因 | 状态 |
|------|------|------|
| 仓库删除 | 后端有删除函数，UI 缺少删除按钮 | MISSING |
| 技能详情 | 静态占位符，无实际加载逻辑 | BROKEN |
| 列表滚动 | 父容器 overflow: hidden | BROKEN |
| 选中状态 | is_selected 字段未用于显示 | MISSING |

## 设计方案

### 1. 仓库删除功能

**实现位置：** `src/App.vue` 侧边栏仓库菜单项

**UI 设计：**
- 每个仓库菜单项右侧添加删除图标（× 或垃圾桶图标）
- 点击删除图标弹出确认对话框
- 确认后调用 `configStore.removeRepository(repoId)`
- 删除仓库时清理 skillsStore 中该仓库的技能数据

**代码变更：**
- App.vue: 在 t-menu-item suffix template 中添加删除按钮
- 使用 t-popconfirm 组件确认删除操作

### 2. 技能详情面板

**实现位置：** `src/components/SkillDetailPanel.vue`

**功能设计：**
- 读取技能目录下的 SKILL.md 文件内容
- 渲染 Markdown 格式显示

**新增 Tauri 命令：**
```rust
// src-tauri/src/commands/skill.rs
#[tauri::command]
pub fn read_skill_content(repo_id: String, skill_path: String) -> Result<String, String>
```

**实现逻辑：**
1. 根据 repo_id 获取仓库本地路径
2. 组合完整路径: `repo_path/skill_path/SKILL.md`
3. 读取文件内容返回

**前端渲染：**
- 安装 markdown 渲染库（如 marked）
- SkillDetailPanel.vue 接收 Markdown 内容并渲染
- 添加加载状态和错误处理

### 3. 技能列表滚动

**实现位置：** `src/App.vue` 和 `src/views/Home.vue` CSS

**CSS 修改：**

App.vue:
```css
.app-content {
  overflow-y: auto;  /* 原: overflow: hidden */
}
```

Home.vue:
```css
.home-content {
  height: 100%;
  overflow-y: auto;
}
```

### 4. 技能选择状态显示

**实现位置：** `src/components/SkillCard.vue`

**UI 设计：**
- 已选中的技能卡片右上角显示"已选"角标
- 使用 TDesign Tag 组件，theme="primary"，size="small"
- 角标定位: `position: absolute; top: 8px; right: 8px`

**交互逻辑：**
- 点击技能卡片触发 `@click` 事件
- 调用 `updateSkillSelection(skill.id, !skill.is_selected)`
- 更新成功后本地刷新 `skill.is_selected` 状态
- 角标根据 `is_selected` 显示/隐藏

**数据流：**
1. 用户点击技能卡片
2. 前端调用 `updateSkillSelection` API
3. 后端更新 SQLite skills 表 `is_selected` 字段
4. 前端更新 Pinia store 中技能状态
5. UI 角标状态更新

## 文件变更清单

**新增文件：**
- `src-tauri/src/commands/skill.rs` - 技能内容读取命令

**修改文件：**
- `src/App.vue` - 添加仓库删除按钮，修复滚动 CSS
- `src/views/Home.vue` - 修复滚动 CSS
- `src/components/SkillDetailPanel.vue` - Markdown 渲染实现
- `src/components/SkillCard.vue` - 添加选中角标和点击切换
- `src/api/tauri.ts` - 添加 readSkillContent API
- `src-tauri/src/lib.rs` - 注册新命令
- `package.json` - 添加 marked 依赖（如需要）

## 实现步骤

1. 修复滚动问题（CSS 修改）
2. 实现仓库删除功能
3. 添加技能内容读取命令
4. 实现技能详情面板 Markdown 渲染
5. 实现技能选中状态显示和切换
6. 测试完整功能