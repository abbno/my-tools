# 常用隧道功能设计文档

## 1. 功能概述

**目标：** 在 SSH 隧道管理器中增加"常用隧道"功能，方便用户快速访问高频使用的隧道配置。

**核心需求：**
- 侧边栏新增"常用隧道"独立区域，点击可筛选显示
- 支持将配置标记/取消标记为常用
- 常用隧道支持手动拖拽排序
- 标记入口：配置表单复选框 + 隧道卡片快捷按钮

---

## 2. 数据模型变更

### Config 表新增字段

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| is_favorite | BOOLEAN | false | 是否标记为常用 |
| favorite_order | INTEGER | 0 | 常用排序序号 |

### 排序逻辑

- 非常用隧道：`favorite_order = 0`
- 常用隧道：`favorite_order > 0`，值越小排序越靠前
- 查询常用隧道时按 `favorite_order ASC` 排序

### 数据迁移

- 新增字段对现有数据无影响，默认值为 false 和 0
- SQLite ALTER TABLE 添加新列即可，无需数据迁移脚本

---

## 3. 后端 API 设计

### 新增 Command

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| get_favorites | - | Config[] | 获取常用隧道列表，按 favorite_order 排序 |
| set_favorite | config_id: string, is_favorite: bool | Config | 标记/取消标记为常用，自动维护 favorite_order |
| reorder_favorites | orders: {id: string, order: number}[] | - | 批量更新常用隧道排序 |

### 修改现有 Command

| Command | 变更 |
|---------|------|
| save_config | 请求体新增可选字段 is_favorite |
| get_configs | 返回结果包含 is_favorite、favorite_order 字段 |

### set_favorite 行为逻辑

```
标记为常用 (is_favorite = true):
  - 查询当前最大 favorite_order
  - 设置该配置 favorite_order = max + 1
  - 设置 is_favorite = true

取消标记 (is_favorite = false):
  - 设置 is_favorite = false
  - 设置 favorite_order = 0
  - 其他常用隧道 order 自动重排（填补空缺）
```

---

## 4. 前端界面设计

### 侧边栏布局

```
┌─────────────────┐
│  分组      [+]  │
├─────────────────┤
│  ★ 常用         │  ← 新增区域标题，点击筛选全部常用
│    ├ 数据库隧道 │  ← 常用项，点击筛选单个隧道
│    ├ Redis隧道  │  ← 支持拖拽排序
│    └ ...
├─────────────────┤
│  □ 全部         │
│  □ 默认分组     │
│  □ 生产环境     │
│  □ 测试环境     │
└─────────────────┘
```

### 交互设计

| 操作 | 行为 |
|------|------|
| 点击"★ 常用"标题 | 筛选显示所有常用隧道 |
| 点击常用项具体名称 | 筛选显示该单个隧道 |
| 拖拽常用项 | 调整顺序，调用 reorder_favorites API |
| 首次标记为常用 | 自动添加到常用列表末尾 |

### 隧道卡片变更

- 右上角新增"收藏"图标按钮
- 空心星 ★：未标记为常用
- 实心星 ★：已标记为常用
- 点击切换状态，调用 set_favorite API

### 配置表单变更

- 底部新增"标记为常用"复选框
- 新建/编辑配置时可设置

---

## 5. 实现文件清单

### 后端 (Rust)

| 文件 | 变更 |
|------|------|
| src-tauri/src/models/config.rs | 新增 is_favorite、favorite_order 字段 |
| src-tauri/src/db/sqlite.rs | ALTER TABLE 添加新列，新增查询方法 |
| src-tauri/src/commands/config.rs | 新增 get_favorites、set_favorite、reorder_favorites 命令 |

### 前端 (Vue/TypeScript)

| 文件 | 变更 |
|------|------|
| src/types/index.ts | Config 接口新增 is_favorite、favoriteOrder 字段 |
| src/api/tauri.ts | 新增 setFavorite、reorderFavorites、getFavorites 方法 |
| src/stores/config.ts | 新增 favorites 状态和相关方法 |
| src/components/Sidebar.vue | 新增常用隧道区域，支持拖拽排序 |
| src/components/TunnelCard.vue | 新增收藏图标按钮 |
| src/components/ConfigForm.vue | 新增"标记为常用"复选框 |

---

## 6. 测试要点

| 测试项 | 验证内容 |
|--------|----------|
| 标记为常用 | 配置被正确标记，出现在侧边栏常用列表 |
| 取消标记 | 配置从常用列表移除，其他常用项 order 自动重排 |
| 拖拽排序 | 顺序调整后持久化保存，重启应用顺序保持 |
| 筛选功能 | 点击常用区域正确筛选显示对应隧道 |
| 导入导出 | 常用标记随配置一起导出导入 |
| 数据迁移 | 旧版本数据升级后新字段默认值正确 |