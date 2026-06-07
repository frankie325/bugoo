# Type Safety

## 类型组织

- **共享类型**（与后端 IPC 对齐）放 `src/types/`
- **API 返回类型**从 `lib/api/` 模块导出
- **组件 Props** 在组件文件内用 `interface` 定义
- **Store 类型**在 store 文件内定义

## 类型与后端对齐

TypeScript 类型用 `camelCase`，Rust 字段也用 `#[serde(rename_all = "camelCase")]`：

```ts
// src/types/tag.ts
export interface TagItem {
  id: string;
  name: string;
  color: string;       // HEX: "#RRGGBB"
  sort_order: number;  // snake_case 与 Rust 字段对齐
  created_at: number;  // Unix timestamp (ms)
  updated_at: number;
}
```

```rust
// src-tauri/src/domain/models/tag.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    // ...
}
```

注意：某些字段特意保持 `snake_case` 以匹配 Rust 端序列化（如 `sort_order`），注释中标注。

## 接口 vs 类型别名

- `interface` 用于对象形状（组件 Props、实体类型）
- `type` 用于联合类型、工具类型
- 不用 `enum`，用 string literal union

```ts
// GOOD
interface TagCreateInput {
  name: string;
  color: string;
  sort_order?: number;
}

// GOOD — 小的本地类型用 type
type PopupState = "loading" | "error" | "empty" | "tooLong" | "resolved";
```

## 禁止模式

- **禁止 `any`**：用 `unknown` + 类型守卫
- **禁止 `as` 类型断言**：除非有明确的类型收窄逻辑
- **禁止 `@ts-ignore`**：修类型错误，不要绕过
- **禁止 `React.FC`**：用显式 Props 解构

## Tauri IPC 类型安全

`invoke<T>()` 的泛型参数需要与 Rust 端返回类型一致：

```ts
export async function getSettings(): Promise<Record<string, string>> {
  return invoke("get_settings");
}
```

如果类型不匹配，运行时会抛反序列化错误。

## 防御式默认值

从 settings (Record<string, string>) 读取时，始终给 fallback：

```ts
const enableSelection = settings.enableSelection !== "false";
const dailyLimit = settings.dailyLimit || "20";
const maxSelectionChars = Number(settings.maxSelectionChars || 200);
```
