# Code Reuse Thinking Guide

> 停止重复造轮子。先搜索，再编码。

## 项目内搜索模式

```bash
# 搜索 Tauri command 名称
grep -r "command_name" src-tauri/src/

# 搜索前端 API 函数
grep -r "export async function" src/lib/api/

# 搜索类型定义
grep -r "interface\|type" src/types/

# 搜索 Rust trait 定义
grep -r "^pub trait" src-tauri/src/ports/

# 搜索已有 settings key
grep -r "getSetting\|set_setting\|settings\." src/ src-tauri/
```

## Bugoo 项目特有关注点

### Settings Key 去重

在添加新的 settings key 前：
- [ ] `grep -r "key_name" src/ src-tauri/resources/default-settings.json`
- [ ] 确认 `src-tauri/resources/default-settings.json` 中有默认值
- [ ] 确认前端 `settingsStore` 中有对应的读取方式

### API 函数去重

- [ ] 搜索 `lib/api/` 目录：是否已有类似的 Tauri invoke 封装
- [ ] 不要直接在组件中 `invoke()` — 所有 IPC 走 `lib/api/`
- [ ] 检查类型文件：`src/types/` 是否已有对应类型

### Rust 层去重

- [ ] ports 中是否已有对应 trait
- [ ] adapters 中是否已有对应实现
- [ ] 不重复定义已存在的 domain model

## 何时抽象

**抽象时机**：
- 同一模式在 3+ 个文件出现
- 逻辑复杂度高（可能有 bug）
- 多个路径需要相同的行为

**不要抽象**：
- 只出现一次的代码
- 一次性的简单映射
- 抽象比重复更复杂的场景

## 批量修改后检查

```bash
# 搜索遗漏的引用
grep -r "old_name" src/ src-tauri/src/

# 类型检查
pnpm tsc --noEmit
cargo check

# 运行测试
pnpm test --run
cd src-tauri && cargo test --lib
```

## 反模式（本项目特有）

- **不要**跨层复制类型：前后端类型需一一对应，不能各自定义不同结构
- **不要**在组件中直接 `invoke()`：必须封装在 `lib/api/`
- **不要**在多个文件中定义同一个 settings key 的字符串字面量
