# Bugoo 首页实现计划

## 上下文

本计划旨在实现 Bugoo 应用的首页功能（生词本列表）。首页是用户最常访问的页面，展示用户保存的所有单词，并提供搜索、筛选、排序等管理功能。

### 目标

- 用户可以在首页浏览、保存、搜索、筛选和管理生词
- 支持按状态（全部/待复习/学习中/已掌握）和标签筛选
- 支持按时间、字母序、掌握程度排序
- 支持批量操作和删除

### 涉及文件

- 前端：`src/components/home/` + `src/hooks/useWords.ts` + `src/stores/` + `src/types/`
- 后端：`src-tauri/src/commands/words.rs` + `src-tauri/src/db/` + `src-tauri/src/domain/` + `src-tauri/src/ports/` + `src-tauri/src/adapters/`

---

## 执行步骤

### Step 1: 数据库层实现

**目标：** 创建 words 表和 SQLite 适配器

**任务：**
- [ ] 创建 `src-tauri/src/db/mod.rs` — 数据库模块入口
- [ ] 创建 `src-tauri/src/db/migrations.rs` — 数据库迁移脚本（创建 words 表）
- [ ] 创建 `src-tauri/src/db/models/mod.rs` — 模型定义
- [ ] 创建 `src-tauri/src/db/models/word.rs` — Word 实体
- [ ] 创建 `src-tauri/src/ports/outbound/repository.rs` — Repository trait 定义
- [ ] 创建 `src-tauri/src/adapters/outbound/sqlite.rs` — SQLite 适配器实现
- [ ] 创建 `src-tauri/src/domain/models/mod.rs` — 领域模型
- [ ] 创建 `src-tauri/src/domain/models/word.rs` — 领域模型（独立于数据库）
- [ ] 创建 `src-tauri/src/domain/services/mod.rs` — 领域服务
- [ ] 创建 `src-tauri/src/domain/services/word_service.rs` — 单词服务
- [ ] 在 `lib.rs` 中初始化数据库连接

**验证命令：**
```bash
cd src-tauri && cargo build
```

**退出标准：** `cargo build` 成功，无编译错误

---

### Step 2: Tauri 命令层实现

**目标：** 实现 words CRUD 命令

**任务：**
- [ ] 创建 `src-tauri/src/commands/mod.rs` — 命令模块入口
- [ ] 创建 `src-tauri/src/commands/words.rs` — 单词 CRUD 命令
  - `add_word` — 添加单词
  - `get_words` — 获取单词列表（支持搜索）
  - `update_word` — 更新单词
  - `delete_word` — 删除单词
- [ ] 在 `lib.rs` 中注册命令处理器

**验证命令：**
```bash
cd src-tauri && cargo build
npm run tauri build  # 可选，检查完整构建
```

**退出标准：** 所有命令编译通过

---

### Step 3: 前端类型定义

**目标：** 定义 TypeScript 类型

**任务：**
- [ ] 创建 `src/types/word.ts` — Word 类型、WordStatus
- [ ] 创建 `src/types/api.ts` — API 响应类型

**验证命令：**
```bash
npx tsc --noEmit
```

**退出标准：** TypeScript 类型检查通过

---

### Step 4: 前端 API 层

**目标：** 封装 Tauri invoke 调用

**任务：**
- [ ] 更新 `src/lib/api.ts` — 扩展 API 函数
  - 添加 `updateWord(wordId, updates)`
  - 添加类型定义

**验证命令：**
```bash
npx tsc --noEmit
```

**退出标准：** TypeScript 类型检查通过

---

### Step 5: 前端状态管理

**目标：** 实现 Zustand stores

**任务：**
- [ ] 创建 `src/stores/uiStore.ts` — UI 状态（视图模式、侧边栏）
- [ ] 创建 `src/stores/filterStore.ts` — 筛选状态（状态筛选、标签、搜索）
- [ ] 创建 `src/stores/wordStore.ts` — 单词数据缓存（可选，React Query 已覆盖）

**验证命令：**
```bash
npx tsc --noEmit
```

**退出标准：** TypeScript 类型检查通过

---

### Step 6: 前端 Hooks

**目标：** 实现数据获取 hooks

**任务：**
- [ ] 创建 `src/hooks/useWords.ts` — 单词列表查询（React Query）
- [ ] 创建 `src/hooks/useDebounce.ts` — 防抖 hook

**验证命令：**
```bash
npx tsc --noEmit
```

**退出标准：** TypeScript 类型检查通过

---

### Step 7: 前端 Home 组件

**目标：** 实现首页 UI 组件

**任务：**
- [ ] 创建 `src/components/home/HomePage.tsx` — 首页容器
- [ ] 创建 `src/components/home/SearchBar.tsx` — 搜索框
- [ ] 创建 `src/components/home/FilterBar.tsx` — 筛选栏
- [ ] 创建 `src/components/home/WordList.tsx` — 单词列表
- [ ] 创建 `src/components/home/WordCard.tsx` — 单词卡片
- [ ] 创建 `src/components/home/EmptyState.tsx` — 空状态

**验证命令：**
```bash
npm run dev
# 访问 http://localhost:1420 检查页面渲染
```

**退出标准：** 页面正常渲染，无 console error

---

### Step 8: 集成测试

**目标：** 端到端验证功能

**任务：**
- [ ] 添加单词到生词本
- [ ] 搜索单词
- [ ] 按状态筛选
- [ ] 删除单词
- [ ] 验证 Rust tests（如果有）

**验证命令：**
```bash
cd src-tauri && cargo test
npm run dev  # 手动验证 UI
```

**退出标准：** 所有功能正常工作

---

## 并行说明

- **Step 1-2**（后端）必须按顺序执行，Step 2 依赖 Step 1
- **Step 3-6**（前端类型、API、Store、Hooks）可并行执行
- **Step 7**（UI 组件）依赖 Step 3-6
- **Step 8** 最后执行，依赖所有前置步骤

## 模型选择

| Step | 模型 |
|------|------|
| Step 1 (数据库) | haiku — 模式简单，直接实现 |
| Step 2 (命令) | haiku — 遵循模板实现 |
| Step 3-6 (前端基础) | haiku — 类型定义和简单 hooks |
| Step 7 (UI) | sonnet — 需要良好 UI 实现 |
| Step 8 (测试) | haiku — 验证性工作 |

## 回滚策略

每个 Step 都有独立的上下文，失败时只需重做该步骤。前端组件使用 HeroUI 组件，样式问题容易修复。
