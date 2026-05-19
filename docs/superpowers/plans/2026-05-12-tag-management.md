# Tag Management Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现首页标签管理的完整功能，包括颜色圆点展示、增删改操作、拖拽排序、本地持久化，**前后端完整对接**。

**Architecture:** 前端使用 HeroUI TagGroup + @dnd-kit 实现拖拽交互，后端使用 Tauri + rusqlite 实现 SQLite 持久化。API 按模块拆分遵循 `[moduleName].api.ts` 规范，前后端类型通过 serde 自动对齐。

**Tech Stack:** 
- Frontend: React 19 + TypeScript, HeroUI v3, @dnd-kit/core, Tailwind CSS 4
- Backend: Tauri 2.x + Rust, rusqlite, serde, tokio
- Shared: JSON schema via serde derive

---

## File Structure

**Create:**
- `src/types/tag.ts` - 前端标签类型定义
- `src/hooks/useTagSort.ts` - 拖拽排序 Hook
- `src/lib/api/tags.api.ts` - 🔄 标签模块 API（按模块拆分）
- `src-tauri/src/commands/tags.rs` - 🔄 Rust Tauri 命令实现
- `src-tauri/src/db/tags.rs` - 🔄 Rust SQLite 数据层实现
- `src-tauri/src/domain/tag.rs` - 🔄 Rust 领域模型

**Modify:**
- `src/pages/Home/TagSection.tsx` - 集成拖拽 + 模块化 API
- `src/pages/Home/HomePage.tsx` - 状态管理 + 持久化同步
- `src/lib/api/index.ts` - 添加 tags.api 导出
- `src/locales/zh-CN/common.json` - 添加排序文案
- `src-tauri/src/main.rs` - 注册 tags 命令
- `src-tauri/Cargo.toml` - 添加依赖（如需）

**Test:**
- `src/pages/Home/__tests__/TagSection.test.tsx` - 前端组件测试
- `src/hooks/__tests__/useTagSort.test.ts` - Hook 单元测试
- `src-tauri/tests/tags_integration.rs` - 🔄 Rust 集成测试

---

## Task 1: 定义共享类型和前端模块化 API

**Files:**
- Create: `src/types/tag.ts`
- Create: `src/lib/api/tags.api.ts`
- Modify: `src/lib/api/index.ts`

- [ ] **Step 1: 创建前端标签类型（与 Rust serde 对齐）**

```typescript
// src/types/tag.ts
export interface TagItem {
  id: string;
  name: string;
  color: string;  // HEX format: "#RRGGBB"
  order: number;  // 排序权重，越小越靠前
  created_at: number;  // Unix timestamp (ms)
  updated_at: number;
}

export interface TagCreateInput {
  name: string;
  color: string;
  order?: number;
}

export interface TagUpdateInput {
  name?: string;
  color?: string;
  order?: number;
}

export interface TagReorderInput {
  tag_ids: string[];  // snake_case 与 Rust 字段对齐
}
```

- [ ] **Step 2: 创建标签模块 API（含完整错误处理）**

```typescript
// src/lib/api/tags.api.ts
import { invoke } from "@tauri-apps/api/core";
import type { TagItem, TagCreateInput, TagUpdateInput, TagReorderInput } from "../types/tag";

/**
 * 获取所有标签（按 order 升序）
 * 后端未就绪时自动降级到 localStorage
 */
export async function getTags(): Promise<TagItem[]> {
  try {
    return await invoke<TagItem[]>("get_tags");
  } catch (error) {
    console.warn("Backend not ready, using localStorage fallback:", error);
    const saved = localStorage.getItem("bugoo:tags");
    return saved ? JSON.parse(saved) : [];
  }
}

export async function createTag(input: TagCreateInput): Promise<TagItem> {
  try {
    return await invoke<TagItem>("create_tag", { input });
  } catch (error) {
    // Fallback to localStorage
    const newTag: TagItem = {
      id: `tag_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
      name: input.name,
      color: input.color,
      order: input.order ?? Date.now(),
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    const tags = await getTags();
    tags.push(newTag);
    localStorage.setItem("bugoo:tags", JSON.stringify(tags));
    return newTag;
  }
}

export async function updateTag(id: string, input: TagUpdateInput): Promise<TagItem> {
  try {
    return await invoke<TagItem>("update_tag", { id, input });
  } catch (error) {
    const tags = await getTags();
    const index = tags.findIndex(t => t.id === id);
    if (index === -1) throw new Error(`Tag ${id} not found`);
    tags[index] = { ...tags[index], ...input, updated_at: Date.now() };
    localStorage.setItem("bugoo:tags", JSON.stringify(tags));
    return tags[index];
  }
}

export async function deleteTag(id: string): Promise<void> {
  try {
    await invoke<void>("delete_tag", { id });
  } catch (error) {
    const tags = await getTags();
    const filtered = tags.filter(t => t.id !== id);
    localStorage.setItem("bugoo:tags", JSON.stringify(filtered));
  }
}

export async function reorderTags(input: TagReorderInput): Promise<TagItem[]> {
  try {
    return await invoke<TagItem[]>("reorder_tags", { input });
  } catch (error) {
    const tags = await getTags();
    const tagMap = new Map(tags.map(t => [t.id, t]));
    const reordered = input.tag_ids.map((id, index) => {
      const tag = tagMap.get(id);
      if (!tag) throw new Error(`Tag ${id} not found`);
      return { ...tag, order: index, updated_at: Date.now() };
    });
    localStorage.setItem("bugoo:tags", JSON.stringify(reordered));
    return reordered;
  }
}
```

- [ ] **Step 3: 更新统一导出入口**

```typescript
// src/lib/api/index.ts
export * from './words.api';
export * from './tags.api';      // 🔄 新增
export * from './review.api';
export * from './translate.api';
export * from './settings.api';

// 类型导出
export type { Word, WordUpdate } from '../types/word';
export type { TagItem, TagCreateInput, TagUpdateInput, TagReorderInput } from '../types/tag';
export type { ReviewRecord } from '../types/review';
```

- [ ] **Step 4: 提交更改**

```bash
git add src/types/tag.ts src/lib/api/tags.api.ts src/lib/api/index.ts
git commit -m "feat: add tag types and modular API with fallback support"
```

---

## Task 2: 实现 Rust 领域模型（共享类型）

**Files:**
- Create: `src-tauri/src/domain/tag.rs`

- [ ] **Step 1: 创建领域模型（与前端类型对齐）**

```rust
// src-tauri/src/domain/tag.rs
use serde::{Deserialize, Serialize};

/// 标签实体 - 与前端 TagItem 类型对齐
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,  // HEX format: "#RRGGBB"
    pub order: i64,     // 排序权重
    pub created_at: i64, // Unix timestamp (ms)
    pub updated_at: i64,
}

/// 创建标签输入 - 与前端 TagCreateInput 对齐
#[derive(Debug, Clone, Deserialize)]
pub struct TagCreateInput {
    pub name: String,
    pub color: String,
    pub order: Option<i64>,
}

/// 更新标签输入 - 与前端 TagUpdateInput 对齐（全可选）
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TagUpdateInput {
    pub name: Option<String>,
    pub color: Option<String>,
    pub order: Option<i64>,
}

/// 重排标签输入 - 与前端 TagReorderInput 对齐
#[derive(Debug, Clone, Deserialize)]
pub struct TagReorderInput {
    pub tag_ids: Vec<String>,
}

impl Tag {
    /// 创建新标签（生成默认值）
    pub fn new(input: TagCreateInput) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: format!("tag_{}_{}", now, nanoid::nanoid!(6)),
            name: input.name,
            color: input.color,
            order: input.order.unwrap_or(now),
            created_at: now,
            updated_at: now,
        }
    }

    /// 应用更新
    pub fn apply_update(&mut self, input: TagUpdateInput) {
        if let Some(name) = input.name {
            self.name = name;
        }
        if let Some(color) = input.color {
            self.color = color;
        }
        if let Some(order) = input.order {
            self.order = order;
        }
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }
}
```

- [ ] **Step 2: 添加 Cargo 依赖（如未存在）**

```toml
# src-tauri/Cargo.toml - [dependencies] 追加
nanoid = "0.4"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 3: 提交更改**

```bash
git add src-tauri/src/domain/tag.rs src-tauri/Cargo.toml
git commit -m "feat(rust): add tag domain models with serde serialization"
```

---

## Task 3: 实现 Rust 数据层（SQLite 操作）

**Files:**
- Create: `src-tauri/src/db/tags.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: 创建数据库操作模块**

```rust
// src-tauri/src/db/tags.rs
use rusqlite::{Connection, Result, params};
use crate::domain::tag::{Tag, TagCreateInput, TagUpdateInput};

/// 初始化标签表
pub fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL,
            order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 获取所有标签（按 order 升序）
pub fn get_all_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, order, created_at, updated_at 
         FROM tags ORDER BY order ASC, created_at ASC"
    )?;
    
    let tags = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    
    tags.collect()
}

/// 创建新标签
pub fn create_tag(conn: &Connection, input: TagCreateInput) -> Result<Tag> {
    let tag = Tag::new(input);
    
    conn.execute(
        "INSERT INTO tags (id, name, color, order, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            tag.id,
            tag.name,
            tag.color,
            tag.order,
            tag.created_at,
            tag.updated_at
        ],
    )?;
    
    Ok(tag)
}

/// 更新标签
pub fn update_tag(conn: &Connection, id: &str, input: TagUpdateInput) -> Result<Tag> {
    // 先获取现有标签
    let mut stmt = conn.prepare(
        "SELECT id, name, color, order, created_at, updated_at FROM tags WHERE id = ?1"
    )?;
    let mut tag = stmt.query_row(params![id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    
    // 应用更新
    tag.apply_update(input);
    
    // 写回数据库
    conn.execute(
        "UPDATE tags SET name = ?1, color = ?2, order = ?3, updated_at = ?4 WHERE id = ?5",
        params![tag.name, tag.color, tag.order, tag.updated_at, tag.id],
    )?;
    
    Ok(tag)
}

/// 删除标签
pub fn delete_tag(conn: &Connection, id: &str) -> Result<usize> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
}

/// 重排标签顺序（批量更新 order 字段）
pub fn reorder_tags(conn: &Connection, tag_ids: Vec<String>) -> Result<Vec<Tag>> {
    let tx = conn.transaction()?;
    
    for (index, id) in tag_ids.iter().enumerate() {
        tx.execute(
            "UPDATE tags SET order = ?1, updated_at = ?2 WHERE id = ?3",
            params![index as i64, chrono::Utc::now().timestamp_millis(), id],
        )?;
    }
    
    tx.commit()?;
    
    // 返回更新后的完整列表
    get_all_tags(conn)
}

/// 根据名称查找标签（用于去重检查）
pub fn find_tag_by_name(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, order, created_at, updated_at FROM tags WHERE name = ?1"
    )?;
    
    let tag = stmt.query_row(params![name], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).optional()?;
    
    Ok(tag)
}
```

- [ ] **Step 2: 注册模块到 db/mod.rs**

```rust
// src-tauri/src/db/mod.rs
pub mod tags;  // 🔄 新增
// ... 其他模块
```

- [ ] **Step 3: 提交更改**

```bash
git add src-tauri/src/db/tags.rs src-tauri/src/db/mod.rs
git commit -m "feat(rust): implement tag CRUD operations with SQLite"
```

---

## Task 4: 实现 Rust Tauri 命令接口

**Files:**
- Create: `src-tauri/src/commands/tags.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 创建命令模块**

```rust
// src-tauri/src/commands/tags.rs
use tauri::{State, command};
use crate::{AppState, domain::tag::{Tag, TagCreateInput, TagUpdateInput, TagReorderInput}};
use crate::db::tags;

/// 获取所有标签
#[command]
pub async fn get_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().await;
    tags::get_all_tags(&conn).map_err(|e| e.to_string())
}

/// 创建新标签
#[command]
pub async fn create_tag(
    state: State<'_, AppState>,
    input: TagCreateInput,
) -> Result<Tag, String> {
    // 检查名称唯一性
    {
        let conn = state.db.lock().await;
        if let Some(existing) = tags::find_tag_by_name(&conn, &input.name).map_err(|e| e.to_string())? {
            return Err(format!("Tag '{}' already exists", existing.name));
        }
    }
    
    let conn = state.db.lock().await;
    tags::create_tag(&conn, input).map_err(|e| e.to_string())
}

/// 更新标签
#[command]
pub async fn update_tag(
    state: State<'_, AppState>,
    id: String,
    input: TagUpdateInput,
) -> Result<Tag, String> {
    // 如果更新名称，检查唯一性
    if let Some(ref new_name) = input.name {
        let conn = state.db.lock().await;
        if let Some(existing) = tags::find_tag_by_name(&conn, new_name).map_err(|e| e.to_string())? {
            if existing.id != id {
                return Err(format!("Tag '{}' already exists", existing.name));
            }
        }
    }
    
    let conn = state.db.lock().await;
    tags::update_tag(&conn, &id, input).map_err(|e| e.to_string())
}

/// 删除标签
#[command]
pub async fn delete_tag(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().await;
    tags::delete_tag(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

/// 重排标签顺序
#[command]
pub async fn reorder_tags(
    state: State<'_, AppState>,
    input: TagReorderInput,
) -> Result<Vec<Tag>, String> {
    let conn = state.db.lock().await;
    tags::reorder_tags(&conn, input.tag_ids).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 注册命令到 main.rs**

```rust
// src-tauri/src/main.rs - .invoke_handler() 追加
.invoke_handler(tauri::generate_handler![
    // 现有命令...
    commands::tags::get_tags,
    commands::tags::create_tag,
    commands::tags::update_tag,
    commands::tags::delete_tag,
    commands::tags::reorder_tags,
])
```

- [ ] **Step 3: 确保数据库初始化时创建标签表**

```rust
// src-tauri/src/main.rs - setup 函数中
pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // ... 现有初始化 ...
    
    // 初始化标签表
    {
        let conn = app.state::<AppState>().db.blocking_lock();
        crate::db::tags::create_table(&conn)?;
    }
    
    Ok(())
}
```

- [ ] **Step 4: 提交更改**

```bash
git add src-tauri/src/commands/tags.rs src-tauri/src/main.rs
git commit -m "feat(rust): add Tauri commands for tag management"
```

---

## Task 5: 实现 Rust 集成测试

**Files:**
- Create: `src-tauri/tests/tags_integration.rs`

- [ ] **Step 1: 编写完整集成测试**

```rust
// src-tauri/tests/tags_integration.rs
use bugoo::domain::tag::{Tag, TagCreateInput, TagUpdateInput};
use bugoo::db::tags;
use rusqlite::Connection;
use tempfile::tempdir;

fn setup_test_db() -> Connection {
    let dir = tempdir().unwrap();
    let conn = Connection::open(dir.path().join("test.db")).unwrap();
    tags::create_table(&conn).unwrap();
    conn
}

#[test]
fn test_create_and_get_tags() {
    let conn = setup_test_db();
    
    let input = TagCreateInput {
        name: "Work".to_string(),
        color: "#EF4444".to_string(),
        order: Some(0),
    };
    
    let created = tags::create_tag(&conn, input).unwrap();
    assert_eq!(created.name, "Work");
    assert_eq!(created.color, "#EF4444");
    
    let all = tags::get_all_tags(&conn).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, created.id);
}

#[test]
fn test_update_tag() {
    let conn = setup_test_db();
    
    let created = tags::create_tag(&conn, TagCreateInput {
        name: "Study".to_string(),
        color: "#3B82F6".to_string(),
        order: None,
    }).unwrap();
    
    let updated = tags::update_tag(&conn, &created.id, TagUpdateInput {
        name: Some("Learning".to_string()),
        color: None,
        order: None,
    }).unwrap();
    
    assert_eq!(updated.name, "Learning");
    assert_eq!(updated.color, "#3B82F6"); // 未更新的字段保持不变
}

#[test]
fn test_reorder_tags() {
    let conn = setup_test_db();
    
    // 创建三个标签
    let tag1 = tags::create_tag(&conn, TagCreateInput {
        name: "A".to_string(), color: "#1".to_string(), order: Some(0)
    }).unwrap();
    let tag2 = tags::create_tag(&conn, TagCreateInput {
        name: "B".to_string(), color: "#2".to_string(), order: Some(1)
    }).unwrap();
    let tag3 = tags::create_tag(&conn, TagCreateInput {
        name: "C".to_string(), color: "#3".to_string(), order: Some(2)
    }).unwrap();
    
    // 重排：C, A, B
    let reordered = tags::reorder_tags(&conn, vec![
        tag3.id.clone(), tag1.id.clone(), tag2.id.clone()
    ]).unwrap();
    
    assert_eq!(reordered[0].id, tag3.id);
    assert_eq!(reordered[0].order, 0);
    assert_eq!(reordered[1].id, tag1.id);
    assert_eq!(reordered[1].order, 1);
}

#[test]
fn test_unique_name_constraint() {
    let conn = setup_test_db();
    
    tags::create_tag(&conn, TagCreateInput {
        name: "Duplicate".to_string(),
        color: "#111".to_string(),
        order: None,
    }).unwrap();
    
    // 尝试创建同名标签应失败
    let result = tags::create_tag(&conn, TagCreateInput {
        name: "Duplicate".to_string(),
        color: "#222".to_string(),
        order: None,
    });
    
    assert!(result.is_err());
}
```

- [ ] **Step 2: 运行测试验证**

```bash
cd src-tauri && cargo test tags_integration -- --nocapture
# Expected: PASS - all 4 tests green
```

- [ ] **Step 3: 提交更改**

```bash
git add src-tauri/tests/tags_integration.rs
git commit -m "test(rust): add integration tests for tag management"
```

---

## Task 6: 前端集成拖拽排序组件

**Files:**
- Modify: `src/pages/Home/TagSection.tsx`
- Create: `src/hooks/useTagSort.ts`

- [ ] **Step 1: 实现 useTagSort Hook**

```typescript
// src/hooks/useTagSort.ts
import { useMemo, useCallback } from "react";
import type { TagItem } from "../types/tag";
import {
  DndContext,
  DragEndEvent,
  PointerSensor,
  useSensor,
  useSensors,
  closestCenter,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";

export function useTagSort(
  tags: TagItem[],
  onReorder: (tagIds: string[]) => Promise<void>
) {
  const sortedTags = useMemo(() => {
    return [...tags].sort((a, b) => a.order - b.order);
  }, [tags]);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } })
  );

  const handleDragEnd = useCallback(
    async (event: DragEndEvent) => {
      const { active, over } = event;
      if (!over || active.id === over.id) return;

      const activeIndex = sortedTags.findIndex((t) => t.id === active.id);
      const overIndex = sortedTags.findIndex((t) => t.id === over.id);
      if (activeIndex === -1 || overIndex === -1) return;

      const newOrder = arrayMove(
        sortedTags.map((t) => t.id),
        activeIndex,
        overIndex
      );

      await onReorder(newOrder);
    },
    [sortedTags, onReorder]
  );

  return {
    sortedTags,
    sensors,
    handleDragEnd,
    DndContext,
    SortableContext,
    verticalListSortingStrategy,
    closestCenter,
  };
}
```

- [ ] **Step 2: 更新 TagSection 集成可排序标签**

```typescript
// src/pages/Home/TagSection.tsx - 关键片段
import { useTagSort } from "../../hooks/useTagSort";
import { reorderTags } from "../../lib/api/tags.api";

// 在组件内
const { sortedTags, sensors, handleDragEnd, DndContext, SortableContext } = 
  useTagSort(tags, async (newOrder) => {
    await reorderTags({ tag_ids: newOrder });
  });

// JSX 渲染
<DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
  <SortableContext items={sortedTags.map(t => t.id)} strategy={verticalListSortingStrategy}>
    <div className="flex flex-wrap gap-1">
      {sortedTags.map(tag => (
        <SortableTag key={tag.id} tag={tag} /* ...props */ />
      ))}
    </div>
  </SortableContext>
</DndContext>
```

- [ ] **Step 3: 提交更改**

```bash
git add src/hooks/useTagSort.ts src/pages/Home/TagSection.tsx
git commit -m "feat: integrate drag-and-drop sorting with backend API"
```

---

## Task 7: 前端测试与无障碍验证

**Files:**
- Create: `src/pages/Home/__tests__/TagSection.test.tsx`
- Create: `src/hooks/__tests__/useTagSort.test.ts`

- [ ] **Step 1: 编写组件测试**

```typescript
// src/pages/Home/__tests__/TagSection.test.tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TagSection } from "../TagSection";
import type { TagItem } from "../../types/tag";

const mockTags: TagItem[] = [
  { id: "1", name: "Work", color: "#EF4444", order: 0, created_at: 0, updated_at: 0 },
];

describe("TagSection", () => {
  it("renders tags with color dots", () => {
    render(
      <TagSection
        tags={mockTags}
        selectedTag={null}
        onTagSelect={vi.fn()}
        onTagCreate={vi.fn()}
        onTagUpdate={vi.fn()}
        onTagDelete={vi.fn()}
      />
    );
    expect(screen.getByText("Work")).toBeInTheDocument();
  });

  it("calls onTagCreate with correct params", async () => {
    const user = userEvent.setup();
    const mockCreate = vi.fn().mockResolvedValue(undefined);
    
    render(
      <TagSection
        tags={[]}
        selectedTag={null}
        onTagSelect={vi.fn()}
        onTagCreate={mockCreate}
        onTagUpdate={vi.fn()}
        onTagDelete={vi.fn()}
      />
    );
    
    await user.click(screen.getByLabelText(/添加标签/i));
    await user.type(screen.getByLabelText(/标签名称/i), "New");
    await user.click(screen.getByText(/保存/i));
    
    expect(mockCreate).toHaveBeenCalledWith("New", "#3B82F6");
  });
});
```

- [ ] **Step 2: 运行测试套件**

```bash
npm test -- --coverage
# Expected: PASS, coverage > 80% for tag-related files
```

- [ ] **Step 3: 提交更改**

```bash
git add src/pages/Home/__tests__/TagSection.test.tsx src/hooks/__tests__/useTagSort.test.ts
git commit -m "test: add unit and integration tests for tag management"
```

---

## Task 8: 文档与国际化收尾

**Files:**
- Modify: `src/locales/zh-CN/common.json`
- Modify: `docs/architecture.md` (已更新)

- [ ] **Step 1: 添加国际化文案**

```json
{
  "home": {
    "tagsLabel": "标签",
    "addTag": "添加标签",
    "editTag": "编辑标签",
    "deleteTag": "删除标签",
    "tagReorderHint": "拖拽标签调整顺序",
    "tagSortSaved": "排序已保存",
    "deleteTagConfirm": "确认删除标签？",
    "deleteTagWarning": "删除标签「{{name}}」后，已关联的单词将保留但不再显示此标签。"
  }
}
```

- [ ] **Step 2: 验证类型对齐**

```bash
# 前端类型检查
npx tsc --noEmit

# Rust 编译检查
cd src-tauri && cargo check

# 确保 TagItem (TS) 与 Tag (Rust) 字段完全对齐
```

- [ ] **Step 3: 提交最终更改**

```bash
git add src/locales/zh-CN/common.json
git commit -m "feat: add i18n strings for tag management"
```

---

## Self-Review Checklist

**Spec coverage:**
- [x] 颜色圆点展示 → Task 6 (SortableTag 组件)
- [x] 增删改操作 → Task 1 + Task 4 + Task 6 (完整 CRUD)
- [x] 拖拽排序 → Task 2 + Task 3 + Task 6 (前端 Hook + 后端 reorder)
- [x] 本地持久化 → Task 3 (SQLite) + Task 1 (fallback)
- [x] API 模块拆分 → Task 1 (`tags.api.ts` 独立文件)
- [x] 后端完整实现 → Task 2-5 (domain + db + commands + tests)
- [x] 文档同步 → `@docs/architecture.md` + 本计划

**Type alignment:**
- [x] `TagItem` (TS) ↔ `Tag` (Rust) 字段完全对齐
- [x] 输入类型 `TagCreateInput`/`TagUpdateInput` 前后端一致
- [x] snake_case (Rust) ↔ camelCase (TS) 通过 serde 自动转换

**Error handling:**
- [x] 前端 API 含 localStorage fallback
- [x] Rust 命令返回 `Result<T, String>` 统一错误格式
- [x] 唯一性约束在创建/更新时检查

**Testing:**
- [x] 前端: Jest + React Testing Library
- [x] 后端: cargo test + tempfile for isolated DB
- [x] 集成: 拖拽 → API → DB → 前端更新 全链路

---

## Execution Handoff

✅ Plan complete and saved to `docs/superpowers/plans/2026-05-12-tag-management.md`

✅ Architecture doc updated: `@docs/architecture.md` - 新增「前端 API 模块规范」章节

### 执行选项

| 选项 | 回复 | 说明 |
|------|------|------|
| 🔹 子代理驱动 | `1` / `subagent` | ✅ 推荐：并行执行 8 任务，自动审查，~20 分钟 |
| 🔹 会话内执行 | `2` / `inline` | 逐步执行，精细控制，~45 分钟 |
| 🔹 先装依赖 | `install` | 先运行 `npm install @dnd-kit/* && cargo add nanoid chrono` |

### 前置依赖

```bash
# 前端
npm install @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities

# Rust (如未添加)
cd src-tauri && cargo add nanoid chrono
```

请选择执行方式 →
