# Thinking Guides

> 编码前扩展思考，减少"没想到"类型的 bug。

## 为什么需要 Thinking Guides？

**大多数 bug 来自"没想到"**，不是技能不够：

- 没考虑边界条件 → 层间 bug
- 没搜索现有代码 → 重复实现
- 没检查 settings key → 前后端不一致

## 可用指南

| 指南 | 用途 | 何时使用 |
|------|------|---------|
| [Code Reuse](./code-reuse-thinking-guide.md) | 搜索现有代码、减少重复 | 新增函数/组件/API 前 |
| [Cross-Layer](./cross-layer-thinking-guide.md) | 跨层数据流和边界 | 涉及 2+ 层的新功能 |

## Bugoo 开发触发性问题

### 前端改动的自检

- [ ] 检查 `src/lib/api/` — 是否需要新的 IPC 函数？
- [ ] 检查 `src/types/` — 类型是否与 Rust 端对齐？
- [ ] 检查 `src/locales/` — 所有 15 个语言文件是否都更新？
- [ ] 检查 `src-tauri/resources/default-settings.json` — 新 settings key 是否有默认值？

### Rust 改动的自检

- [ ] `cargo check` 无错误
- [ ] `cargo clippy` 无 warning
- [ ] `cargo test --lib` 全部通过
- [ ] 所有 `unwrap()` 已替换为 `?` 或 `.expect("why")`
- [ ] 新增 command 已在 `lib.rs` 的 `generate_handler![]` 注册

### 跨层改动的自检

- [ ] 前端类型 camelCase ↔ Rust `#[serde(rename_all = "camelCase")]` 一致
- [ ] 新增字段在 Rust 端有 `#[serde(default)]` 兜底
- [ ] Settings 值双向同步：前端 `updateSetting` → IPC → DB → 缓存

---

**核心原则**: 30 分钟思考省 3 小时 debug。
