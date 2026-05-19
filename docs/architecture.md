# Bugoo 系统架构设计

## 1. 架构风格：Hexagonal Architecture（端口与适配器）

核心业务逻辑与外部依赖解耦，便于测试和扩展。

## 2. Rust 后端模块划分

```
src-tauri/src/
├── commands/        # Tauri IPC 命令（适配器层）
│   ├── translate.rs # 翻译命令
│   ├── words.rs    # 单词 CRUD
│   ├── review.rs   # 复习命令
│   └── settings.rs # 设置命令
├── domain/          # 领域模型（核心业务）
│   ├── models/
│   │   ├── word.rs     # Word 实体
│   │   ├── review.rs   # Review 实体
│   │   └── settings.rs # Settings 实体
│   └── services/
│       ├── word_service.rs
│       └── review_service.rs
├── ports/           # 接口定义（trait）
│   ├── inbound/     # 入站端口（被 frontend 调用）
│   └── outbound/   # 出站端口（调用外部服务）
│       ├── repository.rs    # 数据仓储
│       ├── translation.rs  # 翻译服务
│       └── notification.rs  # 通知服务
├── adapters/        # 接口实现
│   ├── outbound/
│   │   ├── sqlite.rs      # SQLite 存储适配器
│   │   ├── deepl.rs       # DeepL 翻译适配器
│   │   └── tts.rs         # 系统 TTS 适配器
├── scheduler/      # 调度器
│   ├── ebbinghaus.rs  # SM-2 算法
│   └── notification.rs # 通知调度
└── lib.rs          # Tauri 构建器
```

### 模块职责

| 模块 | 职责 |
|------|------|
| commands/ | 接收前端 IPC 调用，委托给 domain service |
| domain/models/ | 核心业务实体（Word, Review, Settings） |
| domain/services/ | 核心业务逻辑（CRUD、复习算法） |
| ports/ | 接口抽象定义（Repository trait 等） |
| adapters/ | 端口的具体实现（SQLite、DeepL、TTS） |
| scheduler/ | 定时任务调度（复习提醒、SM-2 重算） |

## 3. 前端组件结构

```
src/
├── main.tsx                    # 应用入口
├── App.tsx                     # 根组件（路由配置）
├── pages/                      # 页面级组件 ✅ 已实施
│   └── Home/                   # 首页模块 ✅ 已实施
│       ├── index.tsx           # 首页容器 ✅
│       └── components/         # 首页子组件 ✅
│           ├── index.tsx        # barrel 统一导出
│           ├── SearchInput.tsx  # 搜索框 ✅
│           ├── ViewToggle.tsx   # 视图切换 ✅
│           ├── StatusFilter.tsx # 状态筛选 ✅
│           ├── WordGrid.tsx     # 单词网格视图 ✅
│           ├── WordList.tsx     # 单词列表视图 ✅
│           ├── BottomBanner.tsx # 底部学习进度条 ✅
│           ├── DetailPanel.tsx  # 单词详情面板 ✅
│           └── TagSection/      # 标签管理子模块 ✅
│               ├── index.tsx    # barrel 统一导出
│               ├── TagSection.tsx # 标签筛选 ✅
│               ├── SortableTag.tsx # 可拖拽标签项 ✅
│               ├── TagContextMenu.tsx # 右键菜单 ✅
│               └── TagEditorDialog.tsx # 标签编辑弹窗 ✅
│   └── review/                 # 复习模块
│   │   ├── index.tsx
│   │   ├── ReviewCard.tsx
│   │   ├── RecallPhase.tsx
│   │   ├── HintPhase.tsx
│   │   ├── AnswerPhase.tsx
│   │   ├── RatingButtons.tsx
│   │   └── ReviewProgress.tsx
│   ├── selection-popup/
│   │   ├── index.tsx
│   │   ├── TranslationResult.tsx
│   │   └── SaveWordButton.tsx
│   ├── notification/
│   │   ├── index.tsx
│   │   ├── NotificationStack.tsx
│   │   └── ReviewReminder.tsx
│   ├── settings/
│   │   ├── index.tsx
│   │   ├── SettingsSidebar.tsx
│   │   └── panels/
│   │       ├── GeneralPanel.tsx
│   │       ├── TranslationPanel.tsx
│   │       ├── ReviewPanel.tsx
│   │       ├── NotificationPanel.tsx
│   │       ├── AppearancePanel.tsx
│   │       └── ShortcutsPanel.tsx
│   └── detail-panel/
│       ├── WordDetail.tsx
│       ├── WordHeader.tsx
│       ├── MemoryProgress.tsx
│       └── QuickRating.tsx
├── components/                 # 通用 UI 组件（待迁移）
├── hooks/                     # 自定义 Hooks
│   ├── useWords.ts            # 单词查询（React Query）
│   ├── useReview.ts           # 复习查询
│   ├── useTranslation.ts      # 翻译调用
│   ├── useNotifications.ts    # 通知管理
│   ├── useDebounce.ts         # 防抖
│   └── useKeyboard.ts         # 快捷键
├── stores/                    # Zustand stores
│   ├── uiStore.ts             # UI 状态
│   ├── filterStore.ts         # 筛选状态
│   ├── reviewStore.ts         # 复习会话
│   └── settingsStore.ts       # 设置状态
├── lib/                       # 工具库
│   ├── api/                   # 🔄 按模块划分的 API 接口
│   │   ├── words.api.ts       # 单词相关接口
│   │   ├── tags.api.ts        # 标签相关接口
│   │   ├── review.api.ts      # 复习相关接口
│   │   ├── translate.api.ts   # 翻译相关接口
│   │   ├── settings.api.ts    # 设置相关接口
│   │   └── index.ts           # 统一导出入口
│   └── tauri.ts               # Tauri invoke 基础封装
├── types/                     # TypeScript 类型
│   ├── word.ts
│   ├── tag.ts
│   ├── review.ts
│   ├── settings.ts
│   └── api.ts
└── pages/                     # 页面级组件
    └── FloatWindowPage.tsx     # 浮窗页面
```

### 前端状态管理策略

| 数据类型 | 管理方式 | 理由 |
|----------|----------|------|
| 服务端数据（单词、复习任务） | React Query + Tauri IPC | 缓存、加载状态、后端同步 |
| UI 状态（侧边栏、视图模式） | Zustand | 前端私有、快速变化 |
| 筛选/搜索状态 | Zustand + URL params | 可分享链接 |
| 复习会话进度 | Zustand | 客户端临时状态 |

### 路由结构

```
/           → HomePage
/review     → ReviewPage
/settings   → SettingsPage
FloatWindow → FloatWindowPage（通过 URL param ?text=）
```

## 4. 前端 API 模块规范 🔄

### 文件命名规则

```
src/lib/api/[moduleName].api.ts
```

| 模块 | 文件名 | 职责 |
|------|--------|------|
| 单词 | `words.api.ts` | 单词 CRUD、查询、筛选 |
| 标签 | `tags.api.ts` | 标签 CRUD、排序、关联 |
| 复习 | `review.api.ts` | 复习任务获取、提交评分 |
| 翻译 | `translate.api.ts` | 文本翻译调用 |
| 设置 | `settings.api.ts` | 应用配置读写 |
| 通知 | `notifications.api.ts` | 通知发送、权限管理 |

### 接口函数命名规范

```typescript
// ✅ 推荐：动词 + 名词，小写驼峰
export async function getWords(search?: string): Promise<Word[]>
export async function createTag(input: TagCreateInput): Promise<TagItem>
export async function updateWord(id: string, updates: WordUpdate): Promise<Word>
export async function deleteTag(id: string): Promise<void>

// ❌ 避免：名词开头、全大写、缩写歧义
export async function WordsGet()  // 动词后置
export async function CREATE_TAG()  // 全大写
export async function delTag()  // 缩写歧义
```

### 统一导出入口

```typescript
// src/lib/api/index.ts
export * from './words.api'
export * from './tags.api'
export * from './review.api'
export * from './translate.api'
export * from './settings.api'
export * from './notifications.api'

// 类型统一导出
export type { Word, WordUpdate } from '../types/word'
export type { TagItem, TagCreateInput } from '../types/tag'
// ... 其他类型
```

### 错误处理规范

```typescript
// 所有 API 函数统一返回格式
export type ApiResult<T> = {
  success: boolean
  data?: T
  error?: {
    code: string
    message: string
    details?: unknown
  }
}

// 或使用 Result 类型 + try/catch + localStorage fallback
export async function getTags(): Promise<TagItem[]> {
  try {
    return await invoke<TagItem[]>("get_tags")
  } catch (error) {
    console.warn("Backend not ready, using localStorage fallback:", error)
    const saved = localStorage.getItem("bugoo:tags")
    return saved ? JSON.parse(saved) : []
  }
}
```

### 模块依赖关系

```
[moduleName].api.ts
    ↓ 依赖
src/types/[moduleName].ts  # 类型定义
src/lib/tauri.ts          # Tauri invoke 基础封装
    ↓ 可选依赖
src/lib/utils.ts          # 通用工具函数
```

## 5. 数据库 Schema

### words 表
```sql
CREATE TABLE words (
    id TEXT PRIMARY KEY,
    word TEXT NOT NULL,
    translation TEXT NOT NULL,
    phonetic TEXT,
    source_lang TEXT DEFAULT 'EN',
    target_lang TEXT DEFAULT 'ZH',
    status TEXT DEFAULT 'new',  -- new, learning, mastered
    tags TEXT DEFAULT '',
    notes TEXT DEFAULT '',
    audio_url TEXT,
    ease_factor REAL DEFAULT 2.5,
    interval INTEGER DEFAULT 0,
    repetitions INTEGER DEFAULT 0,
    next_review_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### tags 表 🔄 新增
```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL,           -- HEX format: "#RRGGBB"
    order INTEGER NOT NULL DEFAULT 0,  -- 排序权重
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### word_tags 关联表 🔄 新增
```sql
CREATE TABLE word_tags (
    word_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (word_id, tag_id),
    FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

### review_records 表
```sql
CREATE TABLE review_records (
    id TEXT PRIMARY KEY,
    word_id TEXT NOT NULL,
    rating INTEGER NOT NULL,      -- 0-5 SM-2 评分
    reviewed_at INTEGER NOT NULL,
    next_review_at INTEGER NOT NULL,
    FOREIGN KEY (word_id) REFERENCES words(id)
);
```

### settings 表
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

## 6. Tauri 命令接口

```rust
// commands/translate.rs
#[tauri::command]
fn translate_text(text: String, source_lang: String, target_lang: String) -> Result<Translation>

// commands/words.rs
#[tauri::command]
fn add_word(word: String, translation: String, source_lang: String, target_lang: String, tags: String) -> Result<Word>
#[tauri::command]
fn get_words(search: Option<String>) -> Result<Vec<Word>>
#[tauri::command]
fn delete_word(word_id: String) -> Result<()>
#[tauri::command]
fn update_word(word_id: String, updates: WordUpdate) -> Result<Word>

// commands/tags.rs 🔄 新增
#[tauri::command]
fn get_tags() -> Result<Vec<Tag>>
#[tauri::command]
fn create_tag(name: String, color: String, order: Option<i64>) -> Result<Tag>
#[tauri::command]
fn update_tag(id: String, name: Option<String>, color: Option<String>, order: Option<i64>) -> Result<Tag>
#[tauri::command]
fn delete_tag(id: String) -> Result<()>
#[tauri::command]
fn reorder_tags(tag_ids: Vec<String>) -> Result<Vec<Tag>>

// commands/review.rs
#[tauri::command]
fn get_due_reviews() -> Result<Vec<Word>>
#[tauri::command]
fn submit_review(word_id: String, rating: u8) -> Result<Word>

// commands/tts.rs
#[tauri::command]
fn speak_text(text: String, lang: String) -> Result<()>

// commands/window.rs
#[tauri::command]
fn open_float_window() -> Result<()>
```

## 7. 数据流

```
前端 (React)
    ↓ invoke()
Tauri Command (commands/)
    ↓ 调用
Domain Service (domain/services/)
    ↓ 使用
Repository Port (ports/outbound/)
    ↓ 实现
SQLite Adapter (adapters/sqlite.rs)
```

## 8. SM-2 算法实现

```rust
pub struct SM2Input {
    pub quality: u8,        // 0-5 评分
    pub repetitions: u32,
    pub ease_factor: f64,
    pub interval: u32,
}

pub struct SM2Output {
    pub repetitions: u32,
    pub ease_factor: f64,
    pub interval: u32,
    pub next_review: DateTime,
}

pub fn calculate_sm2(input: SM2Input) -> SM2Output {
    // SM-2 算法实现
}
```

## 9. 通知系统

- 使用 `tauri-plugin-notification` 发送系统通知
- 通知调度器每分钟检查一次待复习单词
- 支持静默时段配置
- 通知包含"认识"/"不认识"快速操作按钮

## 10. 状态管理策略

| 状态类型 | 管理方式 |
|---------|---------|
| 服务端数据 | Tauri IPC + React Query |
| UI 状态 | Zustand |
| URL 状态 | React Router search params |
| 表单状态 | React Hook Form |

## 11. 核心设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 架构风格 | Hexagonal | 解耦核心业务与外部依赖，便于测试和扩展 |
| 翻译 API Key | 存 Rust 后端 | 安全，不暴露给前端 |
| 复习算法 | SM-2 | 经典间隔重复算法，实现简单可靠 |
| 状态管理 | Zustand + React Query | 前端轻量状态，后端数据通过 IPC |
| API 组织 | 按模块拆分 `[name].api.ts` | 职责清晰、便于维护、支持 Tree Shaking |
| 类型定义 | 独立 `types/` 目录 | 前后端类型对齐，避免循环依赖 |
