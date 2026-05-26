# macOS 划词监听崩溃修复实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 macOS 下划词监听 `EXC_BREAKPOINT/SIGTRAP` 崩溃，恢复“左键释放触发划词弹窗”的稳定行为。

**Architecture:** 在 `target_os = "macos"` 下不再直接使用 `rdev::listen` 全事件路径，而是恢复 macOS 专用鼠标事件监听适配层，仅上报左键按下/抬起事件到现有 selection 流程。`listener` 保持现有读取、过滤、弹窗编排不变，只替换事件来源；非 macOS 继续走 `rdev::listen`。

**Tech Stack:** Tauri 2、Rust、rdev 0.5.3、CoreGraphics(CFRunLoop/CGEventTap)、cargo test/check。

---

## 文件结构

- 新建：`src-tauri/src/selection/platform/mod.rs`
  - 平台监听适配层模块入口。
- 新建：`src-tauri/src/selection/platform/macos_events.rs`
  - macOS `CGEventTap` 鼠标监听实现，只转发左键事件。
- 修改：`src-tauri/src/selection/mod.rs`
  - 重新导出 `platform` 模块。
- 修改：`src-tauri/src/selection/listener.rs`
  - 增加平台分流：macOS 走 `listen_mouse_events`，其他平台走 `rdev::listen`。
  - 保持 `classify_selection_event`、`read_selected_text`、`filter_selection_text` 与弹窗更新链路不变。
- 测试：`src-tauri/src/selection/listener.rs`（已有单测扩展）
- 测试：`src-tauri/src/selection/platform/macos_events.rs`（新增纯映射单测）

---

### Task 1：恢复平台监听适配层骨架

**Files:**
- Create: `src-tauri/src/selection/platform/mod.rs`
- Create: `src-tauri/src/selection/platform/macos_events.rs`
- Modify: `src-tauri/src/selection/mod.rs`
- Test: `src-tauri/src/selection/platform/macos_events.rs`

- [ ] **Step 1: 先写 macOS 事件映射失败测试**

在 `src-tauri/src/selection/platform/macos_events.rs` 中先只写测试（不实现映射函数）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use core_graphics::event::CGEventType;
    use rdev::{Button, EventType};

    #[test]
    fn maps_left_down() {
        assert_eq!(
            map_mouse_event_type(CGEventType::LeftMouseDown),
            Some(EventType::ButtonPress(Button::Left))
        );
    }

    #[test]
    fn maps_left_up() {
        assert_eq!(
            map_mouse_event_type(CGEventType::LeftMouseUp),
            Some(EventType::ButtonRelease(Button::Left))
        );
    }

    #[test]
    fn ignores_non_left_mouse_events() {
        assert_eq!(map_mouse_event_type(CGEventType::MouseMoved), None);
        assert_eq!(map_mouse_event_type(CGEventType::KeyDown), None);
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cd src-tauri && cargo test selection::platform::macos_events
```

Expected: FAIL，提示 `map_mouse_event_type` 未定义。

- [ ] **Step 3: 实现最小平台模块骨架与映射函数**

实现：

1. `src-tauri/src/selection/platform/mod.rs`

```rust
#[cfg(target_os = "macos")]
pub mod macos_events;
```

2. `src-tauri/src/selection/platform/macos_events.rs` 增加：
   - `pub fn map_mouse_event_type(...) -> Option<rdev::EventType>`
   - 仅处理 `LeftMouseDown/LeftMouseUp`，其余返回 `None`

- [ ] **Step 4: 再次运行测试确认通过**

Run:

```bash
cd src-tauri && cargo test selection::platform::macos_events
```

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/selection/mod.rs src-tauri/src/selection/platform
git commit -m "feat: restore macos-specific mouse event adapter"
```

---

### Task 2：实现 macOS CGEventTap 鼠标监听（仅左键）

**Files:**
- Modify: `src-tauri/src/selection/platform/macos_events.rs`
- Test: `src-tauri/src/selection/platform/macos_events.rs`

- [ ] **Step 1: 先写失败测试（只验证事件构造辅助函数）**

在 `macos_events.rs` 中新增辅助函数并先写测试：

```rust
#[cfg(test)]
mod event_build_tests {
    use super::*;
    use rdev::{Button, EventType};

    #[test]
    fn builds_event_for_left_release() {
        let event = build_mouse_event(EventType::ButtonRelease(Button::Left));
        assert!(matches!(event.event_type, EventType::ButtonRelease(Button::Left)));
    }
}
```

先不实现 `build_mouse_event`。

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cd src-tauri && cargo test selection::platform::macos_events::event_build_tests
```

Expected: FAIL，提示 `build_mouse_event` 未定义。

- [ ] **Step 3: 实现监听函数**

在 `macos_events.rs` 中实现：

1. `pub fn listen_mouse_events<T>(callback: T) -> Result<(), String>`
   - 使用 `CGEventTapCreate + CFRunLoopAddSource + CFRunLoopRun`。
   - `eventsOfInterest` 只包含 `LeftMouseDown/LeftMouseUp`。
2. 回调中仅把 `map_mouse_event_type` 结果转为 `rdev::Event` 后传给上层 callback。
3. 新增 `build_mouse_event(event_type: EventType) -> Event`，用于统一填充 `time/name`。
4. 对 `CGEventTapCreate` 或 `CFMachPortCreateRunLoopSource` 空指针失败返回 `Err(String)`。

- [ ] **Step 4: 运行测试与编译检查**

Run:

```bash
cd src-tauri && cargo test selection::platform::macos_events
cd src-tauri && cargo check
```

Expected:
- 单测 PASS；
- `cargo check` 通过，无新增 FFI 警告/错误。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/selection/platform/macos_events.rs
git commit -m "feat: implement macos cgeventtap left-mouse listener"
```

---

### Task 3：listener 平台分流并保持业务编排不变

**Files:**
- Modify: `src-tauri/src/selection/listener.rs`
- Test: `src-tauri/src/selection/listener.rs`

- [ ] **Step 1: 先写失败测试（平台分流函数）**

在 `listener.rs` 增加测试（先不实现被测函数）：

```rust
#[cfg(test)]
mod routing_tests {
    use super::*;

    #[test]
    fn non_macos_prefers_rdev_listener() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(listener_backend_name(), "rdev");
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cd src-tauri && cargo test selection::listener::routing_tests
```

Expected: FAIL，提示 `listener_backend_name` 未定义。

- [ ] **Step 3: 实现平台分流**

在 `listener.rs`：

1. 恢复 `listen_selection_events` 分流函数：
   - `#[cfg(target_os = "macos")]` 使用 `platform::macos_events::listen_mouse_events`；
   - `#[cfg(not(target_os = "macos"))]` 使用 `rdev::listen`。
2. `start_selection_listener` 内将 `listen(...)` 替换为 `listen_selection_events(...)`。
3. 增加 `listener_backend_name()`（仅用于测试和日志）：
   - macOS 返回 `"macos-event-tap"`；
   - 非 macOS 返回 `"rdev"`。
4. 保持 `classify_selection_event`、`handle_global_event` 与当前逻辑一致，不改业务行为。

- [ ] **Step 4: 运行 selection 测试**

Run:

```bash
cd src-tauri && cargo test selection::listener
```

Expected: PASS，原有 `left_button_release_triggers_read` 等测试仍通过。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/selection/listener.rs
git commit -m "fix: route macos selection listener to event tap adapter"
```

---

### Task 4：回归验证与崩溃复现关闭

**Files:**
- Modify: none (验证任务)
- Test: runtime/manual verification + `cargo test`

- [ ] **Step 1: 运行完整后端测试与编译**

Run:

```bash
cd src-tauri && cargo test
cd src-tauri && cargo check
```

Expected: 全部通过。

- [ ] **Step 2: 启动应用做手工回归**

Run:

```bash
pnpm tauri dev
```

验证点：
- macOS 已授权 Accessibility 时，应用启动不崩溃；
- 连续划词 5-10 分钟，无 `EXC_BREAKPOINT`；
- 左键释放可稳定弹出/更新划词窗口；
- 空白或超长文本不会触发有效弹窗。

- [ ] **Step 3: 崩溃日志确认**

操作：
- 复测后检查系统崩溃报告目录（本地）是否出现新的 Bugoo 同类崩溃；
- 若无新增同类崩溃，记为通过。

- [ ] **Step 4: 记录结果并提交（仅文档/日志变更时）**

若有验证记录文档变更：

```bash
git add <验证记录文件路径>
git commit -m "test: verify macos selection listener crash is resolved"
```

若无文件变更，本步骤跳过提交。

---

## 自检清单（计划级）

- 覆盖性：已覆盖“止崩”“行为不变”“跨平台分流”三个核心目标。
- 无占位符：所有任务均给出明确文件、命令、预期。
- 一致性：`listener` 仅切换事件来源，不更改读取/过滤/弹窗主链路。

---

Plan complete and saved to `docs/superpowers/plans/2026-05-22-selection-listener-crash-fix.md`. Two execution options:

1. Subagent-Driven (recommended) - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. Inline Execution - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
