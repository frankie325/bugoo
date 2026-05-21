# 划词弹窗最小实现计划

> **给 agentic workers：** 必须使用子技能：推荐 `superpowers:subagent-driven-development`，也可以使用 `superpowers:executing-plans`。请按任务逐项执行，步骤使用 checkbox（`- [ ]`）跟踪。

**目标：** 实现稳定的第一条端到端链路：应用启动时检查 macOS Accessibility 权限，权限通过后使用 `rdev` 监听全局鼠标左键释放事件，使用 `get_selected_text` 读取选中文字；只有读取到非空且不超过 50 个字符的文本时，才打开或更新划词弹窗。

**架构：** Bugoo 自己只保留应用编排代码。`rdev` 负责跨平台全局鼠标事件监听，`get_selected_text` 负责跨平台选中文字读取，Bugoo 负责权限提示、文本过滤和 Tauri 窗口管理。移除上一轮排障中自维护的 macOS `CGEventTap`、`AXSelectedText` 读取、Windows UI Automation 读取和剪贴板兜底链路。

**技术栈：** Tauri 2、Rust、`rdev`、`get-selected-text`、macOS `accessibility-sys`、React 19、React Router、HeroUI/Tailwind、Vitest。

---

## 文件结构

- 修改 `src-tauri/Cargo.toml`
  - 增加 `get-selected-text = "0.1.6"`。
  - 保留 `rdev = "0.5.3"`。
  - macOS 仅保留 `accessibility-sys = "0.2.0"` 用于权限检查。
  - 删除旧自研读取/监听链路依赖：`accessibility`、`core-foundation`、`core-graphics`、`uiautomation`。
- 修改 `src-tauri/src/lib.rs`
  - 注册权限窗口命令。
  - 将 setup 阶段的 `start_selection_listener(...)` 改为 `initialize_selection(...)`。
- 修改 `src-tauri/src/selection/mod.rs`
  - 只导出新方案需要的模块。
- 修改 `src-tauri/src/selection/types.rs`
  - 保留 `SelectionCandidate`。
  - 将读取错误收敛为 `SelectionReadError::ReadFailed(String)`。
- 修改 `src-tauri/src/selection/filter.rs`
  - 只保留空文本过滤和 50 字符长度过滤。
  - 不再做 500ms 节流，也不再做重复文本抑制。
- 修改 `src-tauri/src/selection/reader.rs`
  - 封装 `get_selected_text::get_selected_text()`。
  - trim 后为空则返回 `Ok(None)`。
- 修改 `src-tauri/src/selection/listener.rs`
  - 使用 `rdev::listen`。
  - 只在 `ButtonRelease(Button::Left)` 时触发读取。
  - 编排顺序：读取文本 -> 过滤 -> 打开/更新弹窗。
- 新建 `src-tauri/src/selection/permission.rs`
  - macOS 使用 `AXIsProcessTrusted()` 检查 Accessibility。
  - 非 macOS 默认已授权。
- 新建 `src-tauri/src/selection/permission_prompt.rs`
  - 应用启动时按权限状态决定启动 listener 或打开权限提示窗口。
  - 权限提示窗口每次运行最多主动打开一次。
  - 权限提示窗口打开时每 1 秒轮询一次权限变化。
  - 用户授权后自动关闭提示窗口并启动 listener，无需重启应用。
  - 用户点击“稍后”后关闭提示窗口，并停止本轮轮询。
- 删除旧平台适配文件
  - `src-tauri/src/selection/clipboard.rs`
  - `src-tauri/src/selection/platform/mod.rs`
  - `src-tauri/src/selection/platform/macos.rs`
  - `src-tauri/src/selection/platform/macos_events.rs`
  - `src-tauri/src/selection/platform/windows.rs`
  - `src-tauri/src/selection/platform/unsupported.rs`
- 修改 `src-tauri/src/commands/window.rs`
  - 保留已有 selection popup 创建/更新逻辑。
  - 增加 Accessibility 权限提示窗口。
  - 增加打开 macOS Accessibility 设置的命令。
  - 增加关闭权限提示窗口并停止本轮轮询的命令。
- 修改 `src-tauri/capabilities/default.json`
  - 给权限提示窗口增加 capability。
- 修改 `src/App.tsx`
  - 增加 `/accessibility-permission` 路由。
- 新建 `src/pages/AccessibilityPermission/index.tsx`
  - 使用 HeroUI `Button`，提供“去系统设置”和“稍后”。
- 新建 `src/pages/AccessibilityPermission/__test__/AccessibilityPermission.test.tsx`
  - 测试说明文案和两个按钮的 invoke 行为。

---

### 任务 1：收敛依赖和 selection 模块边界

**文件：**
- 修改：`src-tauri/Cargo.toml`
- 修改：`src-tauri/src/selection/mod.rs`
- 删除：`src-tauri/src/selection/clipboard.rs`
- 删除：`src-tauri/src/selection/platform/mod.rs`
- 删除：`src-tauri/src/selection/platform/macos.rs`
- 删除：`src-tauri/src/selection/platform/macos_events.rs`
- 删除：`src-tauri/src/selection/platform/windows.rs`
- 删除：`src-tauri/src/selection/platform/unsupported.rs`

- [ ] **步骤 1：更新 Cargo 依赖**

在 `src-tauri/Cargo.toml` 中，selection 相关依赖收敛为：

```toml
rdev = "0.5.3"
get-selected-text = "0.1.6"

[target.'cfg(target_os = "macos")'.dependencies]
accessibility-sys = "0.2.0"
```

如果存在以下旧依赖，删除它们：

```toml
accessibility = "0.2.0"
core-foundation = "0.10.1"
core-graphics = "0.25.0"
uiautomation = "0.25.0"
```

- [ ] **步骤 2：删除旧平台适配文件**

删除：

```text
src-tauri/src/selection/clipboard.rs
src-tauri/src/selection/platform/mod.rs
src-tauri/src/selection/platform/macos.rs
src-tauri/src/selection/platform/macos_events.rs
src-tauri/src/selection/platform/windows.rs
src-tauri/src/selection/platform/unsupported.rs
```

- [ ] **步骤 3：更新模块导出**

将 `src-tauri/src/selection/mod.rs` 替换为：

```rust
pub mod filter;
pub mod listener;
pub mod permission;
pub mod permission_prompt;
pub mod reader;
pub mod types;
```

- [ ] **步骤 4：运行依赖检查**

运行：

```bash
cd src-tauri && cargo check
```

预期：这一阶段允许因为后续文件尚未实现而失败，但不能出现依赖解析失败。如果失败点是旧模块引用缺失，继续后续任务。

- [ ] **步骤 5：提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/selection
git commit -m "refactor: simplify selection dependencies"
```

---

### 任务 2：简化 Selection Filter

**文件：**
- 修改：`src-tauri/src/selection/types.rs`
- 修改：`src-tauri/src/selection/filter.rs`

- [ ] **步骤 1：写失败测试和共享类型**

将 `src-tauri/src/selection/types.rs` 替换为：

```rust
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionCandidate {
    pub text: String,
    pub captured_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionReadError {
    ReadFailed(String),
}

impl fmt::Display for SelectionReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionReadError::ReadFailed(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SelectionReadError {}
```

将 `src-tauri/src/selection/filter.rs` 先替换为只包含测试的失败版本：

```rust
use std::time::Instant;

use super::types::SelectionCandidate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text_after_trim() {
        let now = Instant::now();
        assert_eq!(filter_selection_text("   \n\t  ", now), None);
    }

    #[test]
    fn rejects_text_longer_than_50_chars() {
        let now = Instant::now();
        let text = "a".repeat(51);
        assert_eq!(filter_selection_text(&text, now), None);
    }

    #[test]
    fn accepts_trimmed_text_within_limit() {
        let now = Instant::now();
        assert_eq!(
            filter_selection_text("  hello  ", now),
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: now,
            }),
        );
    }

    #[test]
    fn accepts_same_text_repeatedly() {
        let now = Instant::now();
        assert!(filter_selection_text("hello", now).is_some());
        assert!(filter_selection_text("hello", now).is_some());
    }
}
```

- [ ] **步骤 2：验证测试失败**

运行：

```bash
cd src-tauri && cargo test selection::filter
```

预期：失败，错误指向 `filter_selection_text` 未实现。

- [ ] **步骤 3：实现最小过滤逻辑**

将 `src-tauri/src/selection/filter.rs` 替换为：

```rust
use std::time::Instant;

use super::types::SelectionCandidate;

const MAX_SELECTION_CHARS: usize = 50;

pub fn filter_selection_text(
    raw_text: &str,
    captured_at: Instant,
) -> Option<SelectionCandidate> {
    let text = raw_text.trim();

    if text.is_empty() || text.chars().count() > MAX_SELECTION_CHARS {
        return None;
    }

    Some(SelectionCandidate {
        text: text.to_string(),
        captured_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text_after_trim() {
        let now = Instant::now();
        assert_eq!(filter_selection_text("   \n\t  ", now), None);
    }

    #[test]
    fn rejects_text_longer_than_50_chars() {
        let now = Instant::now();
        let text = "a".repeat(51);
        assert_eq!(filter_selection_text(&text, now), None);
    }

    #[test]
    fn accepts_trimmed_text_within_limit() {
        let now = Instant::now();
        assert_eq!(
            filter_selection_text("  hello  ", now),
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: now,
            }),
        );
    }

    #[test]
    fn accepts_same_text_repeatedly() {
        let now = Instant::now();
        assert!(filter_selection_text("hello", now).is_some());
        assert!(filter_selection_text("hello", now).is_some());
    }
}
```

- [ ] **步骤 4：验证通过**

```bash
cd src-tauri && cargo test selection::filter
```

- [ ] **步骤 5：提交**

```bash
git add src-tauri/src/selection/types.rs src-tauri/src/selection/filter.rs
git commit -m "refactor: simplify selection filtering"
```

---

### 任务 3：用 get_selected_text 实现取词层

**文件：**
- 修改：`src-tauri/src/selection/reader.rs`

- [ ] **步骤 1：写失败测试**

将 `src-tauri/src/selection/reader.rs` 替换为：

```rust
use super::types::SelectionReadError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_non_empty_selected_text() {
        let result = read_selected_text_with(|| Ok("  hello  ".to_string()));
        assert_eq!(result, Ok(Some("hello".to_string())));
    }

    #[test]
    fn returns_none_for_empty_selected_text() {
        let result = read_selected_text_with(|| Ok("   \n\t  ".to_string()));
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn maps_reader_errors() {
        let result = read_selected_text_with(|| Err("boom".into()));
        assert_eq!(
            result,
            Err(SelectionReadError::ReadFailed(
                "failed to read selected text: boom".to_string(),
            )),
        );
    }
}
```

- [ ] **步骤 2：验证测试失败**

```bash
cd src-tauri && cargo test selection::reader
```

预期：失败，缺少 `read_selected_text_with`。

- [ ] **步骤 3：实现 reader wrapper**

将 `src-tauri/src/selection/reader.rs` 替换为：

```rust
use super::types::SelectionReadError;

pub fn read_selected_text() -> Result<Option<String>, SelectionReadError> {
    read_selected_text_with(get_selected_text::get_selected_text)
}

pub fn read_selected_text_with<F>(read: F) -> Result<Option<String>, SelectionReadError>
where
    F: FnOnce() -> Result<String, Box<dyn std::error::Error>>,
{
    let text = read()
        .map_err(|error| SelectionReadError::ReadFailed(format!("failed to read selected text: {error}")))?;
    let text = text.trim().to_string();

    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_non_empty_selected_text() {
        let result = read_selected_text_with(|| Ok("  hello  ".to_string()));
        assert_eq!(result, Ok(Some("hello".to_string())));
    }

    #[test]
    fn returns_none_for_empty_selected_text() {
        let result = read_selected_text_with(|| Ok("   \n\t  ".to_string()));
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn maps_reader_errors() {
        let result = read_selected_text_with(|| Err("boom".into()));
        assert_eq!(
            result,
            Err(SelectionReadError::ReadFailed(
                "failed to read selected text: boom".to_string(),
            )),
        );
    }
}
```

- [ ] **步骤 4：验证通过**

```bash
cd src-tauri && cargo test selection::reader
```

- [ ] **步骤 5：提交**

```bash
git add src-tauri/src/selection/reader.rs
git commit -m "feat: read selected text with get-selected-text"
```

---

### 任务 4：实现 macOS Accessibility 权限检查

**文件：**
- 新建：`src-tauri/src/selection/permission.rs`

- [ ] **步骤 1：新增权限模块**

创建 `src-tauri/src/selection/permission.rs`：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibilityPermission {
    Granted,
    Missing,
}

pub fn accessibility_permission() -> AccessibilityPermission {
    if accessibility_permission_granted() {
        AccessibilityPermission::Granted
    } else {
        AccessibilityPermission::Missing
    }
}

#[cfg(target_os = "macos")]
pub fn accessibility_permission_granted() -> bool {
    unsafe { accessibility_sys::AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn accessibility_permission_granted() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_shape_is_stable() {
        let permission = accessibility_permission();
        assert!(matches!(
            permission,
            AccessibilityPermission::Granted | AccessibilityPermission::Missing
        ));
    }
}
```

- [ ] **步骤 2：验证通过**

```bash
cd src-tauri && cargo test selection::permission
```

- [ ] **步骤 3：提交**

```bash
git add src-tauri/src/selection/permission.rs src-tauri/src/selection/mod.rs
git commit -m "feat: check accessibility permission"
```

---

### 任务 5：用 rdev 编排全局鼠标释放监听

**文件：**
- 修改：`src-tauri/src/selection/listener.rs`

- [ ] **步骤 1：写事件分类测试**

将 `src-tauri/src/selection/listener.rs` 替换为失败测试骨架：

```rust
use rdev::{Button, Event, EventType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionEventDecision {
    Ignore,
    ReadSelection,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn event(event_type: EventType) -> Event {
        Event {
            event_type,
            time: SystemTime::UNIX_EPOCH,
            name: None,
        }
    }

    #[test]
    fn left_button_release_triggers_read() {
        assert_eq!(
            classify_selection_event(&event(EventType::ButtonRelease(Button::Left))),
            SelectionEventDecision::ReadSelection,
        );
    }

    #[test]
    fn left_button_press_is_ignored() {
        assert_eq!(
            classify_selection_event(&event(EventType::ButtonPress(Button::Left))),
            SelectionEventDecision::Ignore,
        );
    }

    #[test]
    fn right_button_release_is_ignored() {
        assert_eq!(
            classify_selection_event(&event(EventType::ButtonRelease(Button::Right))),
            SelectionEventDecision::Ignore,
        );
    }
}
```

- [ ] **步骤 2：验证测试失败**

```bash
cd src-tauri && cargo test selection::listener
```

预期：失败，缺少 `classify_selection_event`。

- [ ] **步骤 3：实现 listener 编排**

将 `src-tauri/src/selection/listener.rs` 替换为：

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;
use std::time::Instant;

use rdev::{listen, Button, Event, EventType};
use tauri::{async_runtime, AppHandle};

use crate::commands::window::open_or_update_selection_popup;
use crate::selection::filter::filter_selection_text;
use crate::selection::reader::read_selected_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionEventDecision {
    Ignore,
    ReadSelection,
}

pub fn start_selection_listener(app: AppHandle) {
    thread::spawn(move || {
        log::info!("Selection listener thread started");
        let callback_app = app.clone();
        let result = listen(move |event| {
            let callback_app = callback_app.clone();
            let event_type = event.event_type;
            let result = catch_unwind(AssertUnwindSafe(|| {
                handle_global_event(event, callback_app);
            }));

            if result.is_err() {
                log::error!("Selection listener callback panicked while handling {event_type:?}");
            }
        });

        if let Err(error) = result {
            log::warn!("Failed to start global selection listener: {error:?}");
        } else {
            log::warn!("Selection listener stopped unexpectedly");
        }
    });
}

pub fn classify_selection_event(event: &Event) -> SelectionEventDecision {
    match event.event_type {
        EventType::ButtonRelease(Button::Left) => SelectionEventDecision::ReadSelection,
        _ => SelectionEventDecision::Ignore,
    }
}

fn handle_global_event(event: Event, app: AppHandle) {
    if classify_selection_event(&event) == SelectionEventDecision::Ignore {
        return;
    }

    log::info!("Selection trigger detected on left mouse release");
    async_runtime::spawn(async move {
        let selected_text = match read_selected_text() {
            Ok(Some(text)) => text,
            Ok(None) => {
                log::debug!("Selection read completed with empty text");
                return;
            }
            Err(error) => {
                log::warn!("Failed to read selected text: {error}");
                return;
            }
        };

        let candidate = match filter_selection_text(&selected_text, Instant::now()) {
            Some(candidate) => candidate,
            None => {
                log::debug!("Selection filtered out before popup update");
                return;
            }
        };

        log::info!("Selection accepted, opening/updating popup");
        if let Err(error) = open_or_update_selection_popup(&app, &candidate.text) {
            log::warn!("Failed to open selection popup: {error}");
        }
    });
}
```

保留步骤 1 中的测试模块。

- [ ] **步骤 4：验证通过**

```bash
cd src-tauri && cargo test selection::listener
```

- [ ] **步骤 5：提交**

```bash
git add src-tauri/src/selection/listener.rs
git commit -m "refactor: listen for selection release with rdev"
```

---

### 任务 6：实现权限提示编排

**文件：**
- 新建：`src-tauri/src/selection/permission_prompt.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：写启动决策测试**

创建 `src-tauri/src/selection/permission_prompt.rs`：

```rust
use crate::selection::permission::AccessibilityPermission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStartupAction {
    StartListener,
    OpenPermissionPrompt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_listener_when_permission_is_granted() {
        assert_eq!(
            startup_action_for_permission(AccessibilityPermission::Granted),
            SelectionStartupAction::StartListener,
        );
    }

    #[test]
    fn opens_prompt_when_permission_is_missing() {
        assert_eq!(
            startup_action_for_permission(AccessibilityPermission::Missing),
            SelectionStartupAction::OpenPermissionPrompt,
        );
    }
}
```

- [ ] **步骤 2：验证测试失败**

```bash
cd src-tauri && cargo test selection::permission_prompt
```

预期：失败，缺少 `startup_action_for_permission`。

- [ ] **步骤 3：实现权限提示编排**

将 `src-tauri/src/selection/permission_prompt.rs` 替换为：

```rust
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use tauri::{async_runtime, AppHandle, Emitter, Manager};

use crate::commands::window::{
    close_accessibility_permission_window, open_accessibility_permission_window,
};
use crate::selection::listener::start_selection_listener;
use crate::selection::permission::{accessibility_permission, AccessibilityPermission};

const ACCESSIBILITY_PERMISSION_GRANTED_EVENT: &str = "accessibility-permission://granted";
const PERMISSION_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStartupAction {
    StartListener,
    OpenPermissionPrompt,
}

pub fn startup_action_for_permission(
    permission: AccessibilityPermission,
) -> SelectionStartupAction {
    match permission {
        AccessibilityPermission::Granted => SelectionStartupAction::StartListener,
        AccessibilityPermission::Missing => SelectionStartupAction::OpenPermissionPrompt,
    }
}

#[derive(Clone)]
pub struct SelectionRuntimeState {
    listener_started: Arc<AtomicBool>,
    permission_polling_cancelled: Arc<AtomicBool>,
}

impl Default for SelectionRuntimeState {
    fn default() -> Self {
        Self {
            listener_started: Arc::new(AtomicBool::new(false)),
            permission_polling_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl SelectionRuntimeState {
    pub fn cancel_permission_polling(&self) {
        self.permission_polling_cancelled
            .store(true, Ordering::SeqCst);
    }
}

pub fn initialize_selection(app: AppHandle) {
    let state = SelectionRuntimeState::default();
    app.manage(state.clone());

    match startup_action_for_permission(accessibility_permission()) {
        SelectionStartupAction::StartListener => start_listener_once(&app, &state),
        SelectionStartupAction::OpenPermissionPrompt => {
            if let Err(error) = open_accessibility_permission_window(&app) {
                log::warn!("Failed to open Accessibility permission window: {error}");
            }
            start_accessibility_permission_polling(app, state);
        }
    }
}

pub fn stop_accessibility_permission_polling(app: &AppHandle) {
    if let Some(state) = app.try_state::<SelectionRuntimeState>() {
        state.cancel_permission_polling();
    }
}

fn start_listener_once(app: &AppHandle, state: &SelectionRuntimeState) {
    if state
        .listener_started
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        start_selection_listener(app.clone());
    }
}

fn start_accessibility_permission_polling(app: AppHandle, state: SelectionRuntimeState) {
    async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(PERMISSION_POLL_INTERVAL).await;

            if state.permission_polling_cancelled.load(Ordering::SeqCst) {
                return;
            }

            if accessibility_permission() == AccessibilityPermission::Granted {
                state.cancel_permission_polling();
                if let Err(error) = close_accessibility_permission_window(&app) {
                    log::warn!("Failed to close Accessibility permission window: {error}");
                }
                start_listener_once(&app, &state);
                if let Err(error) = app.emit(ACCESSIBILITY_PERMISSION_GRANTED_EVENT, ()) {
                    log::warn!("Failed to emit Accessibility permission granted event: {error}");
                }
                return;
            }
        }
    });
}
```

补回步骤 1 中的测试，并额外增加：

```rust
#[test]
fn can_cancel_permission_polling() {
    let state = SelectionRuntimeState::default();
    assert!(!state.permission_polling_cancelled.load(Ordering::SeqCst));
    state.cancel_permission_polling();
    assert!(state.permission_polling_cancelled.load(Ordering::SeqCst));
}
```

- [ ] **步骤 4：接入 lib.rs**

在 `src-tauri/src/lib.rs` 中，将：

```rust
use crate::selection::listener::start_selection_listener;
```

替换为：

```rust
use crate::selection::permission_prompt::initialize_selection;
```

将 setup 中的：

```rust
start_selection_listener(app.handle().clone());
info!("Selection listener started");
```

替换为：

```rust
initialize_selection(app.handle().clone());
info!("Selection service initialized");
```

- [ ] **步骤 5：运行测试**

```bash
cd src-tauri && cargo test selection::permission_prompt
```

预期：此时可能因为任务 7 的窗口函数尚未存在而失败。如果失败点是缺少 `open_accessibility_permission_window` 或 `close_accessibility_permission_window`，继续任务 7。

- [ ] **步骤 6：暂不提交**

如果任务 6 尚不能编译，不单独提交。等任务 7 补齐窗口命令后一起提交。

---

### 任务 7：新增 Accessibility 权限窗口命令

**文件：**
- 修改：`src-tauri/src/commands/window.rs`
- 修改：`src-tauri/src/lib.rs`
- 修改：`src-tauri/capabilities/default.json`

- [ ] **步骤 1：写 URL 测试**

在 `src-tauri/src/commands/window.rs` 的 `tests` 模块中增加：

```rust
#[test]
fn builds_accessibility_permission_url() {
    assert_eq!(accessibility_permission_url(), "/accessibility-permission");
}
```

- [ ] **步骤 2：验证测试失败**

```bash
cd src-tauri && cargo test commands::window
```

预期：失败，缺少 `accessibility_permission_url`。

- [ ] **步骤 3：实现权限窗口命令**

在 `src-tauri/src/commands/window.rs` 中保留现有 selection popup 代码，并新增：

```rust
const ACCESSIBILITY_PERMISSION_LABEL: &str = "accessibility-permission";

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    open_accessibility_settings_impl()
}

#[tauri::command]
pub fn dismiss_accessibility_permission_prompt(app: AppHandle) -> Result<(), String> {
    crate::selection::permission_prompt::stop_accessibility_permission_polling(&app);
    close_accessibility_permission_window(&app)
}

pub fn open_accessibility_permission_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ACCESSIBILITY_PERMISSION_LABEL) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        ACCESSIBILITY_PERMISSION_LABEL,
        WebviewUrl::App(accessibility_permission_url().into()),
    )
    .title("Bugoo Accessibility Permission")
    .inner_size(420.0, 240.0)
    .min_inner_size(360.0, 220.0)
    .decorations(true)
    .always_on_top(true)
    .resizable(false)
    .visible(true)
    .build()
    .map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn close_accessibility_permission_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ACCESSIBILITY_PERMISSION_LABEL) {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn accessibility_permission_url() -> String {
    "/accessibility-permission".to_string()
}

#[cfg(target_os = "macos")]
fn open_accessibility_settings_impl() -> Result<(), String> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn open_accessibility_settings_impl() -> Result<(), String> {
    Ok(())
}
```

- [ ] **步骤 4：注册命令**

在 `src-tauri/src/lib.rs` 的 `tauri::generate_handler![...]` 中增加：

```rust
commands::window::open_accessibility_settings,
commands::window::dismiss_accessibility_permission_prompt,
```

- [ ] **步骤 5：增加 capability**

将 `src-tauri/capabilities/default.json` 中的：

```json
"windows": ["main", "float-*"],
```

改为：

```json
"windows": ["main", "float-*", "accessibility-permission"],
```

- [ ] **步骤 6：验证 Rust**

```bash
cd src-tauri && cargo test commands::window
cd src-tauri && cargo test selection::permission_prompt
cd src-tauri && cargo check
```

预期：通过。已有无关 warning 可以接受。

- [ ] **步骤 7：提交任务 6 和任务 7**

```bash
git add src-tauri/src/selection/permission_prompt.rs src-tauri/src/lib.rs src-tauri/src/commands/window.rs src-tauri/capabilities/default.json
git commit -m "feat: prompt for accessibility permission"
```

---

### 任务 8：新增 Accessibility 权限提示页面

**文件：**
- 新建：`src/pages/AccessibilityPermission/index.tsx`
- 新建：`src/pages/AccessibilityPermission/__test__/AccessibilityPermission.test.tsx`
- 修改：`src/App.tsx`

- [ ] **步骤 1：写失败测试**

创建 `src/pages/AccessibilityPermission/__test__/AccessibilityPermission.test.tsx`：

```tsx
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AccessibilityPermissionPage } from "../index";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("AccessibilityPermissionPage", () => {
  it("renders permission instructions", () => {
    render(<AccessibilityPermissionPage />);

    expect(screen.getByText("开启辅助功能权限")).toBeTruthy();
    expect(screen.getByText(/划词弹窗需要 macOS 辅助功能权限/)).toBeTruthy();
  });

  it("opens system settings", () => {
    invokeMock.mockResolvedValueOnce(undefined);
    render(<AccessibilityPermissionPage />);

    fireEvent.click(screen.getByRole("button", { name: "去系统设置" }));

    expect(invokeMock).toHaveBeenCalledWith("open_accessibility_settings");
  });

  it("dismisses the prompt", () => {
    invokeMock.mockResolvedValueOnce(undefined);
    render(<AccessibilityPermissionPage />);

    fireEvent.click(screen.getByRole("button", { name: "稍后" }));

    expect(invokeMock).toHaveBeenCalledWith("dismiss_accessibility_permission_prompt");
  });
});
```

- [ ] **步骤 2：验证测试失败**

```bash
pnpm test -- AccessibilityPermission
```

预期：失败，因为页面尚未实现。

- [ ] **步骤 3：实现页面**

创建 `src/pages/AccessibilityPermission/index.tsx`：

```tsx
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@heroui/react";

export function AccessibilityPermissionPage() {
  const openSettings = () => {
    invoke("open_accessibility_settings").catch((error) => {
      console.warn("Failed to open Accessibility settings", error);
    });
  };

  const dismiss = () => {
    invoke("dismiss_accessibility_permission_prompt").catch((error) => {
      console.warn("Failed to dismiss Accessibility permission prompt", error);
    });
  };

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-5">
      <section className="w-full max-w-sm rounded-lg border border-default-200 bg-content1 p-5 shadow-sm">
        <div className="space-y-3">
          <div>
            <h1 className="text-lg font-semibold text-foreground">开启辅助功能权限</h1>
            <p className="mt-2 text-sm leading-6 text-default-600">
              划词弹窗需要 macOS 辅助功能权限，授权后 Bugoo 才能读取你在其他应用中选中的文本。
            </p>
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onPress={dismiss}>
              稍后
            </Button>
            <Button onPress={openSettings}>
              去系统设置
            </Button>
          </div>
        </div>
      </section>
    </main>
  );
}
```

- [ ] **步骤 4：注册路由**

在 `src/App.tsx` 增加 import：

```tsx
import { AccessibilityPermissionPage } from "./pages/AccessibilityPermission";
```

在 `<Routes>` 中增加：

```tsx
<Route path="/accessibility-permission" element={<AccessibilityPermissionPage />} />
```

- [ ] **步骤 5：验证前端测试**

```bash
pnpm test -- AccessibilityPermission
pnpm test -- SelectionPopup
```

- [ ] **步骤 6：提交**

```bash
git add src/pages/AccessibilityPermission src/App.tsx
git commit -m "feat: add accessibility permission page"
```

---

### 任务 9：最终验证和手工回归

**文件：**
- 任务 1 到任务 8 修改过的全部文件。

- [ ] **步骤 1：运行 Rust 测试**

```bash
cd src-tauri && cargo test selection
cd src-tauri && cargo test commands::window
```

预期：通过。已有无关 warning 可以接受。

- [ ] **步骤 2：运行 Rust 检查**

```bash
cd src-tauri && cargo check
```

预期：通过。

- [ ] **步骤 3：运行前端测试**

```bash
pnpm test -- SelectionPopup
pnpm test -- AccessibilityPermission
```

预期：通过。

- [ ] **步骤 4：检查旧实现引用是否已删除**

运行：

```bash
rg -n "CGEventTap|UI Automation|uiautomation|read_selected_text_via_copy|core_graphics|core_foundation|AXSelectedText|SelectionFilter::new|TRIGGER_THROTTLE|DUPLICATE" src-tauri/src src-tauri/Cargo.toml
```

预期：活跃实现代码中没有匹配。docs 中出现旧方案名可以接受。

- [ ] **步骤 5：macOS 手工回归**

运行：

```bash
pnpm tauri dev
```

手工验证：

- Accessibility 权限缺失时，打开 `accessibility-permission` 权限提示窗口。
- 点击“去系统设置”后，打开 macOS Accessibility 设置页。
- 点击“稍后”后，关闭提示窗口，并且本轮运行不再轮询权限。
- 提示窗口保持打开时，在系统设置里授权 Accessibility 后，提示窗口自动关闭，listener 自动启动，不需要重启应用。
- Accessibility 权限已授权时，在非 Bugoo 应用中选中非空且 50 字符以内文本，松开左键后打开或更新划词弹窗。
- 触控板三指拖选、鼠标按住拖选、双击选词，只要最终产生左键释放事件并能被 `get_selected_text` 读到文本，都应触发同一条流程。
- 选中文字为空时，不打开也不更新弹窗。
- 选中文字超过 50 字符时，不打开也不更新弹窗。
- 连续多次选中相同文本也应该重复触发弹窗更新；不再有 500ms 节流和重复文本抑制。

- [ ] **步骤 6：如有修复则提交**

如果最终验证过程中需要修复代码：

```bash
git add <fixed-files>
git commit -m "fix: complete selection popup verification"
```

如果没有产生修复，不创建空提交。
