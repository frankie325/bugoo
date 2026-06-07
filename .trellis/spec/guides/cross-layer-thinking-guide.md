# Cross-Layer Thinking Guide

> Bugoo 是典型的三层架构。绝大多数 bug 出在层间边界。

## Bugoo 层级

```
前端 (React/Zustand/React Query)
    ↕ Tauri IPC (invoke)
Commands (Rust #[tauri::command])
    ↕ 领域服务
Domain Services
    ↕ Repository trait
SQLite Adapter (rusqlite)
```

## 每层边界检查

### 前端 → Tauri IPC

- [ ] TypeScript 类型与 Rust `#[serde(rename_all)]` 字段名对齐
- [ ] `camelCase` vs `snake_case` 转换正确
- [ ] Rust 端 `#[serde(default)]` 覆盖了前端可能不传的字段
- [ ] 错误处理：`invoke` 抛出的错误在前端有 try/catch 或显示

### Tauri Command → Domain Service

- [ ] Command 只做参数映射 + 委托，不含业务逻辑
- [ ] 错误从 service 层正确传播（`?` 或 `.map_err(|e| e.to_string())`）
- [ ] `State<'_, AppState>` 正确注入

### Domain Service → Repository

- [ ] 输入验证在 service 层完成（空字符串、无效值等）
- [ ] 错误从 `DbError` 转为 `String` 通过 `.map_err(|e| e.to_string())`
- [ ] Repository trait 和实现之间没有隐式类型转换

### Repository → SQLite

- [ ] 所有查询使用参数化 `params![]` 防注入
- [ ] `serde_json` 字段序列化不丢失数据
- [ ] 事务：多表写入用 `transaction()`

## Settings 数据流

```
ReviewPanel.tsx
    → updateSetting(key, val) [Zustand 乐观更新]
    → setSetting(key, val) [Tauri IPC]
    → set_setting command
    → SQLite INSERT OR REPLACE
    → AppState.settings_cache.insert
```

Settings 从 DB 到前端的三段链路：
1. AppState 启动时 `seed_settings` 写入默认值
2. 前端 `getSettings()` 拉取全量
3. 每次 `setSetting` 即写 DB + 更新缓存

## Date/Time 边界

- Rust 端：`chrono::Utc::now().timestamp_millis()` → `i64`
- 前端接收：`number`（毫秒时间戳）
- 显示前检查：时间戳为 0 = 未设置

## 反模式

- **不要**在前端硬编码 Rust 常量（如 SM-2 参数）
- **不要**在不同层定义不同含义的同名字段
- **不要**假设 JSON 序列化是无损的（测试 round-trip）
- **不要**在 IPC 层做业务验证（应该在 domain service 做）
