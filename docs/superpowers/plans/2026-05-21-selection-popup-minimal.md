# 选中文字弹窗最小实现计划

> **给 agentic workers：** 必须使用子技能：推荐 `superpowers:subagent-driven-development`，也可以使用 `superpowers:executing-plans`。请按任务逐项执行，步骤使用 checkbox（`- [ ]`）跟踪。

**目标：** 实现第一条端到端链路：用户在其他桌面应用中选中文字后，Bugoo 打开一个小弹窗，弹窗只显示选中的原文。

**架构：** 新增 Rust `selection` 模块，专门负责系统选区捕获、过滤、平台选区读取、剪贴板兜底和全局鼠标释放监听。窗口创建仍放在 `commands/window.rs`，前端 UI 放在独立的 `SelectionPopup` 页面，不接入翻译和生词本逻辑。

**技术栈：** Tauri 2、Rust、rdev 全局鼠标/键盘事件、macOS Accessibility API、Windows UI Automation、Tauri clipboard plugin、React 19、React Router、HeroUI、Vitest。

---

## 文件结构

- 修改 `src-tauri/Cargo.toml`
  - 增加选区捕获相关依赖。
- 修改 `src-tauri/src/lib.rs`
  - 注册 `open_selection_popup`。
  - 在 setup 阶段启动 selection listener。
- 创建 `src-tauri/src/selection/mod.rs`
  - selection 模块入口。
- 创建 `src-tauri/src/selection/types.rs`
  - 选区读取共享类型和错误类型。
- 创建 `src-tauri/src/selection/filter.rs`
  - 纯逻辑：空文本过滤、长度过滤、节流、重复文本抑制。
- 创建 `src-tauri/src/selection/reader.rs`
  - “平台读取优先，剪贴板兜底”的编排逻辑。
- 创建 `src-tauri/src/selection/clipboard.rs`
  - 使用 rdev 模拟复制，并通过 Tauri clipboard plugin 读取/恢复剪贴板。
- 创建 `src-tauri/src/selection/listener.rs`
  - 全局鼠标释放监听，把系统事件转为选区读取任务。
- 创建 `src-tauri/src/selection/platform/mod.rs`
  - 平台读取器分发。
- 创建 `src-tauri/src/selection/platform/macos.rs`
  - macOS Accessibility 选中文本读取。
- 创建 `src-tauri/src/selection/platform/windows.rs`
  - Windows UI Automation 选中文本读取。
- 创建 `src-tauri/src/selection/platform/unsupported.rs`
  - 非 macOS/Windows 平台返回 unsupported 错误。
- 修改 `src-tauri/src/commands/window.rs`
  - 新增 `open_selection_popup` 和窗口复用逻辑。
- 修改 `src/App.tsx`
  - 新增 `/selection-popup` 路由。
- 创建 `src/pages/SelectionPopup/index.tsx`
  - 浮窗页面：读取 query text，并监听后续更新事件。
- 创建 `src/pages/SelectionPopup/SelectionText.tsx`
  - 只负责展示选中文本。
- 创建 `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`
  - 前端测试：query 文本、空态、事件更新。
- 修改 `vite.config.ts`
  - 增加 Vitest `jsdom` 测试环境。

---

### 任务 1：新增 Rust 选区过滤核心

**文件：**
- 创建：`src-tauri/src/selection/mod.rs`
- 创建：`src-tauri/src/selection/types.rs`
- 创建：`src-tauri/src/selection/filter.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：编写失败的过滤测试**

创建 `src-tauri/src/selection/mod.rs`：

```rust
pub mod filter;
pub mod types;
```

创建 `src-tauri/src/selection/types.rs`：

```rust
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionCandidate {
    pub text: String,
    pub captured_at: Instant,
}
```

创建 `src-tauri/src/selection/filter.rs`：

```rust
use std::time::{Duration, Instant};

use super::types::SelectionCandidate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text_after_trim() {
        let mut state = SelectionFilter::new();
        let result = state.accept("   \n\t  ", Instant::now());
        assert_eq!(result, None);
    }

    #[test]
    fn rejects_text_longer_than_50_chars() {
        let mut state = SelectionFilter::new();
        let text = "a".repeat(51);
        let result = state.accept(&text, Instant::now());
        assert_eq!(result, None);
    }

    #[test]
    fn accepts_trimmed_text_within_limit() {
        let mut state = SelectionFilter::new();
        let now = Instant::now();
        let result = state.accept("  hello  ", now);
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: now,
            }),
        );
    }

    #[test]
    fn throttles_triggers_inside_500ms_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("world", start + Duration::from_millis(499));
        assert_eq!(result, None);
    }

    #[test]
    fn allows_new_text_after_throttle_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("world", start + Duration::from_millis(500));
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "world".to_string(),
                captured_at: start + Duration::from_millis(500),
            }),
        );
    }

    #[test]
    fn suppresses_duplicate_text_after_throttle_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("hello", start + Duration::from_millis(800));
        assert_eq!(result, None);
    }
}
```

修改 `src-tauri/src/lib.rs` 顶部模块声明，加入：

```rust
mod selection;
```

- [ ] **步骤 2：运行测试，确认失败**

运行：

```bash
cd src-tauri && cargo test selection::filter
```

预期：失败，错误中出现缺少 `SelectionFilter`。

- [ ] **步骤 3：实现最小过滤逻辑**

替换 `src-tauri/src/selection/filter.rs`：

```rust
use std::time::{Duration, Instant};

use super::types::SelectionCandidate;

const MAX_SELECTION_CHARS: usize = 50;
const TRIGGER_THROTTLE: Duration = Duration::from_millis(500);

#[derive(Debug, Default)]
pub struct SelectionFilter {
    last_text: Option<String>,
    last_triggered_at: Option<Instant>,
}

impl SelectionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept(&mut self, raw_text: &str, captured_at: Instant) -> Option<SelectionCandidate> {
        let text = raw_text.trim();

        if text.is_empty() || text.chars().count() > MAX_SELECTION_CHARS {
            return None;
        }

        if let Some(last_triggered_at) = self.last_triggered_at {
            if captured_at.duration_since(last_triggered_at) < TRIGGER_THROTTLE {
                return None;
            }
        }

        if self.last_text.as_deref() == Some(text) {
            return None;
        }

        let text = text.to_string();
        self.last_text = Some(text.clone());
        self.last_triggered_at = Some(captured_at);

        Some(SelectionCandidate { text, captured_at })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text_after_trim() {
        let mut state = SelectionFilter::new();
        let result = state.accept("   \n\t  ", Instant::now());
        assert_eq!(result, None);
    }

    #[test]
    fn rejects_text_longer_than_50_chars() {
        let mut state = SelectionFilter::new();
        let text = "a".repeat(51);
        let result = state.accept(&text, Instant::now());
        assert_eq!(result, None);
    }

    #[test]
    fn accepts_trimmed_text_within_limit() {
        let mut state = SelectionFilter::new();
        let now = Instant::now();
        let result = state.accept("  hello  ", now);
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "hello".to_string(),
                captured_at: now,
            }),
        );
    }

    #[test]
    fn throttles_triggers_inside_500ms_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("world", start + Duration::from_millis(499));
        assert_eq!(result, None);
    }

    #[test]
    fn allows_new_text_after_throttle_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("world", start + Duration::from_millis(500));
        assert_eq!(
            result,
            Some(SelectionCandidate {
                text: "world".to_string(),
                captured_at: start + Duration::from_millis(500),
            }),
        );
    }

    #[test]
    fn suppresses_duplicate_text_after_throttle_window() {
        let mut state = SelectionFilter::new();
        let start = Instant::now();
        assert!(state.accept("hello", start).is_some());
        let result = state.accept("hello", start + Duration::from_millis(800));
        assert_eq!(result, None);
    }
}
```

- [ ] **步骤 4：运行过滤测试，确认通过**

运行：

```bash
cd src-tauri && cargo test selection::filter
```

预期：`selection::filter` 全部通过。

- [ ] **步骤 5：提交**

运行：

```bash
git add src-tauri/src/lib.rs src-tauri/src/selection/mod.rs src-tauri/src/selection/types.rs src-tauri/src/selection/filter.rs
git commit -m "feat: add selection filtering core"
```

---

### 任务 2：新增选区读取编排逻辑

**文件：**
- 修改：`src-tauri/src/selection/mod.rs`
- 修改：`src-tauri/src/selection/types.rs`
- 创建：`src-tauri/src/selection/reader.rs`

- [ ] **步骤 1：编写失败的读取编排测试**

修改 `src-tauri/src/selection/mod.rs`：

```rust
pub mod filter;
pub mod reader;
pub mod types;
```

替换 `src-tauri/src/selection/types.rs`：

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
    PermissionDenied(String),
    PlatformUnavailable(String),
    PlatformReadFailed(String),
    ClipboardReadFailed(String),
    ClipboardWriteFailed(String),
    SimulateCopyFailed(String),
}

impl fmt::Display for SelectionReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionReadError::PermissionDenied(message)
            | SelectionReadError::PlatformUnavailable(message)
            | SelectionReadError::PlatformReadFailed(message)
            | SelectionReadError::ClipboardReadFailed(message)
            | SelectionReadError::ClipboardWriteFailed(message)
            | SelectionReadError::SimulateCopyFailed(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SelectionReadError {}
```

创建 `src-tauri/src/selection/reader.rs`：

```rust
use super::types::SelectionReadError;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FakePlatformReader {
        result: Result<Option<String>, SelectionReadError>,
    }

    impl PlatformSelectionReader for FakePlatformReader {
        fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError> {
            self.result.clone()
        }
    }

    #[derive(Debug)]
    struct FakeClipboardReader {
        result: Result<Option<String>, SelectionReadError>,
    }

    impl ClipboardSelectionReader for FakeClipboardReader {
        fn read_selected_text_via_copy(&self) -> Result<Option<String>, SelectionReadError> {
            self.result.clone()
        }
    }

    #[test]
    fn uses_platform_text_without_clipboard_fallback() {
        let platform = FakePlatformReader {
            result: Ok(Some("hello".to_string())),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("clipboard".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("hello".to_string())));
    }

    #[test]
    fn uses_clipboard_when_platform_returns_empty() {
        let platform = FakePlatformReader { result: Ok(None) };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("fallback".to_string())));
    }

    #[test]
    fn uses_clipboard_when_platform_read_fails() {
        let platform = FakePlatformReader {
            result: Err(SelectionReadError::PlatformReadFailed(
                "text pattern unavailable".to_string(),
            )),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("fallback".to_string())));
    }

    #[test]
    fn returns_permission_error_without_clipboard_attempt() {
        let platform = FakePlatformReader {
            result: Err(SelectionReadError::PermissionDenied(
                "accessibility permission required".to_string(),
            )),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(
            result,
            Err(SelectionReadError::PermissionDenied(
                "accessibility permission required".to_string(),
            )),
        );
    }
}
```

- [ ] **步骤 2：运行测试，确认失败**

运行：

```bash
cd src-tauri && cargo test selection::reader
```

预期：失败，错误中出现缺少 `PlatformSelectionReader`、`ClipboardSelectionReader` 或 `read_selected_text`。

- [ ] **步骤 3：实现读取编排逻辑**

替换 `src-tauri/src/selection/reader.rs`：

```rust
use super::types::SelectionReadError;

pub trait PlatformSelectionReader {
    fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError>;
}

pub trait ClipboardSelectionReader {
    fn read_selected_text_via_copy(&self) -> Result<Option<String>, SelectionReadError>;
}

pub fn read_selected_text<P, C>(
    platform: &P,
    clipboard: &C,
) -> Result<Option<String>, SelectionReadError>
where
    P: PlatformSelectionReader,
    C: ClipboardSelectionReader,
{
    match platform.read_selected_text() {
        Ok(Some(text)) if !text.trim().is_empty() => Ok(Some(text)),
        Ok(_) => clipboard.read_selected_text_via_copy(),
        Err(SelectionReadError::PermissionDenied(message)) => {
            Err(SelectionReadError::PermissionDenied(message))
        }
        Err(error) => {
            log::debug!("Platform selection read failed, using clipboard fallback: {error}");
            clipboard.read_selected_text_via_copy()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FakePlatformReader {
        result: Result<Option<String>, SelectionReadError>,
    }

    impl PlatformSelectionReader for FakePlatformReader {
        fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError> {
            self.result.clone()
        }
    }

    #[derive(Debug)]
    struct FakeClipboardReader {
        result: Result<Option<String>, SelectionReadError>,
    }

    impl ClipboardSelectionReader for FakeClipboardReader {
        fn read_selected_text_via_copy(&self) -> Result<Option<String>, SelectionReadError> {
            self.result.clone()
        }
    }

    #[test]
    fn uses_platform_text_without_clipboard_fallback() {
        let platform = FakePlatformReader {
            result: Ok(Some("hello".to_string())),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("clipboard".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("hello".to_string())));
    }

    #[test]
    fn uses_clipboard_when_platform_returns_empty() {
        let platform = FakePlatformReader { result: Ok(None) };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("fallback".to_string())));
    }

    #[test]
    fn uses_clipboard_when_platform_read_fails() {
        let platform = FakePlatformReader {
            result: Err(SelectionReadError::PlatformReadFailed(
                "text pattern unavailable".to_string(),
            )),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(result, Ok(Some("fallback".to_string())));
    }

    #[test]
    fn returns_permission_error_without_clipboard_attempt() {
        let platform = FakePlatformReader {
            result: Err(SelectionReadError::PermissionDenied(
                "accessibility permission required".to_string(),
            )),
        };
        let clipboard = FakeClipboardReader {
            result: Ok(Some("fallback".to_string())),
        };

        let result = read_selected_text(&platform, &clipboard);

        assert_eq!(
            result,
            Err(SelectionReadError::PermissionDenied(
                "accessibility permission required".to_string(),
            )),
        );
    }
}
```

- [ ] **步骤 4：运行 selection 测试**

运行：

```bash
cd src-tauri && cargo test selection
```

预期：`selection::filter` 和 `selection::reader` 测试通过。

- [ ] **步骤 5：提交**

运行：

```bash
git add src-tauri/src/selection/mod.rs src-tauri/src/selection/types.rs src-tauri/src/selection/reader.rs
git commit -m "feat: add selection reader orchestration"
```

---

### 任务 3：新增平台读取器和剪贴板兜底

**文件：**
- 修改：`src-tauri/Cargo.toml`
- 修改：`src-tauri/src/selection/mod.rs`
- 创建：`src-tauri/src/selection/clipboard.rs`
- 创建：`src-tauri/src/selection/platform/mod.rs`
- 创建：`src-tauri/src/selection/platform/macos.rs`
- 创建：`src-tauri/src/selection/platform/windows.rs`
- 创建：`src-tauri/src/selection/platform/unsupported.rs`

- [ ] **步骤 1：添加依赖**

运行：

```bash
cd src-tauri && cargo add rdev
cd src-tauri && cargo add --target 'cfg(target_os = "macos")' accessibility accessibility-sys core-foundation
cd src-tauri && cargo add --target 'cfg(windows)' uiautomation
```

预期：`src-tauri/Cargo.toml` 和 `src-tauri/Cargo.lock` 发生变化。

- [ ] **步骤 2：声明模块**

替换 `src-tauri/src/selection/mod.rs`：

```rust
pub mod clipboard;
pub mod filter;
pub mod platform;
pub mod reader;
pub mod types;
```

- [ ] **步骤 3：实现剪贴板兜底**

创建 `src-tauri/src/selection/clipboard.rs`：

```rust
use std::thread;
use std::time::Duration;

use rdev::{simulate, EventType, Key};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::reader::ClipboardSelectionReader;
use super::types::SelectionReadError;

#[derive(Clone)]
pub struct TauriClipboardSelectionReader {
    app: AppHandle,
}

impl TauriClipboardSelectionReader {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl ClipboardSelectionReader for TauriClipboardSelectionReader {
    fn read_selected_text_via_copy(&self) -> Result<Option<String>, SelectionReadError> {
        let previous_text = self.app.clipboard().read_text().ok();

        simulate_copy_shortcut()?;
        thread::sleep(Duration::from_millis(80));

        let selected_text = self
            .app
            .clipboard()
            .read_text()
            .map_err(|error| SelectionReadError::ClipboardReadFailed(error.to_string()))?;

        if let Some(previous_text) = previous_text {
            if let Err(error) = self.app.clipboard().write_text(previous_text) {
                log::warn!("Failed to restore clipboard text after selection read: {error}");
            }
        }

        let selected_text = selected_text.trim().to_string();
        if selected_text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(selected_text))
        }
    }
}

fn simulate_copy_shortcut() -> Result<(), SelectionReadError> {
    #[cfg(target_os = "macos")]
    let modifier = Key::MetaLeft;

    #[cfg(not(target_os = "macos"))]
    let modifier = Key::ControlLeft;

    send_event(EventType::KeyPress(modifier))?;
    send_event(EventType::KeyPress(Key::KeyC))?;
    send_event(EventType::KeyRelease(Key::KeyC))?;
    send_event(EventType::KeyRelease(modifier))?;
    Ok(())
}

fn send_event(event_type: EventType) -> Result<(), SelectionReadError> {
    simulate(&event_type).map_err(|error| {
        SelectionReadError::SimulateCopyFailed(format!(
            "failed to simulate {event_type:?}: {error:?}",
        ))
    })?;
    thread::sleep(Duration::from_millis(20));
    Ok(())
}
```

- [ ] **步骤 4：实现平台分发**

创建 `src-tauri/src/selection/platform/mod.rs`：

```rust
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::SystemSelectionReader;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use unsupported::SystemSelectionReader;
#[cfg(target_os = "windows")]
pub use windows::SystemSelectionReader;
```

- [ ] **步骤 5：实现 macOS Accessibility 读取器**

创建 `src-tauri/src/selection/platform/macos.rs`：

```rust
use std::ptr;

use accessibility_sys::{
    kAXErrorSuccess, AXIsProcessTrusted, AXUIElementCopyAttributeValue,
    AXUIElementCreateSystemWide, AXUIElementRef,
};
use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::string::{CFString, CFStringRef};

use crate::selection::reader::PlatformSelectionReader;
use crate::selection::types::SelectionReadError;

#[derive(Debug, Default)]
pub struct SystemSelectionReader;

impl PlatformSelectionReader for SystemSelectionReader {
    fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError> {
        if !unsafe { AXIsProcessTrusted() } {
            return Err(SelectionReadError::PermissionDenied(
                "macOS accessibility permission is required. Open System Settings -> Privacy & Security -> Accessibility and enable Bugoo.".to_string(),
            ));
        }

        let system = unsafe { AXUIElementCreateSystemWide() };
        if system.is_null() {
            return Err(SelectionReadError::PlatformReadFailed(
                "failed to create macOS system-wide accessibility element".to_string(),
            ));
        }

        let focused = copy_ax_element_attribute(system, "AXFocusedUIElement")?;
        let selected_text = copy_ax_string_attribute(focused, "AXSelectedText")?;
        unsafe {
            CFRelease(focused);
            CFRelease(system as CFTypeRef);
        }

        let selected_text = selected_text.trim().to_string();
        if selected_text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(selected_text))
        }
    }
}

fn copy_ax_element_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
) -> Result<AXUIElementRef, SelectionReadError> {
    let attribute = CFString::new(attribute_name);
    let mut value: CFTypeRef = ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };

    if error != kAXErrorSuccess || value.is_null() {
        return Err(SelectionReadError::PlatformReadFailed(format!(
            "macOS accessibility attribute {attribute_name} unavailable: {error:?}",
        )));
    }

    Ok(value as AXUIElementRef)
}

fn copy_ax_string_attribute(
    element: AXUIElementRef,
    attribute_name: &str,
) -> Result<String, SelectionReadError> {
    let attribute = CFString::new(attribute_name);
    let mut value: CFTypeRef = ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
    };

    if error != kAXErrorSuccess || value.is_null() {
        return Err(SelectionReadError::PlatformReadFailed(format!(
            "macOS accessibility attribute {attribute_name} unavailable: {error:?}",
        )));
    }

    let text = unsafe { CFString::wrap_under_create_rule(value as CFStringRef).to_string() };
    Ok(text)
}
```

- [ ] **步骤 6：实现 Windows UI Automation 读取器**

创建 `src-tauri/src/selection/platform/windows.rs`：

```rust
use uiautomation::actions::Text;
use uiautomation::patterns::UITextPattern;
use uiautomation::UIAutomation;

use crate::selection::reader::PlatformSelectionReader;
use crate::selection::types::SelectionReadError;

#[derive(Debug, Default)]
pub struct SystemSelectionReader;

impl PlatformSelectionReader for SystemSelectionReader {
    fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError> {
        let automation = UIAutomation::new().map_err(|error| {
            SelectionReadError::PlatformReadFailed(format!(
                "failed to initialize Windows UI Automation: {error}",
            ))
        })?;

        let element = automation.get_focused_element().map_err(|error| {
            SelectionReadError::PlatformReadFailed(format!(
                "failed to read focused UI Automation element: {error}",
            ))
        })?;

        let text_pattern = element.get_pattern::<UITextPattern>().map_err(|error| {
            SelectionReadError::PlatformReadFailed(format!(
                "focused element does not expose TextPattern: {error}",
            ))
        })?;

        let ranges = text_pattern.get_selection().map_err(|error| {
            SelectionReadError::PlatformReadFailed(format!(
                "failed to read UI Automation text selection: {error}",
            ))
        })?;

        let selected_text = ranges
            .into_iter()
            .filter_map(|range| range.get_text(-1).ok())
            .collect::<Vec<_>>()
            .join("");

        let selected_text = selected_text.trim().to_string();
        if selected_text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(selected_text))
        }
    }
}
```

- [ ] **步骤 7：实现 unsupported 平台读取器**

创建 `src-tauri/src/selection/platform/unsupported.rs`：

```rust
use crate::selection::reader::PlatformSelectionReader;
use crate::selection::types::SelectionReadError;

#[derive(Debug, Default)]
pub struct SystemSelectionReader;

impl PlatformSelectionReader for SystemSelectionReader {
    fn read_selected_text(&self) -> Result<Option<String>, SelectionReadError> {
        Err(SelectionReadError::PlatformUnavailable(
            "system selection reader is only enabled on macOS and Windows".to_string(),
        ))
    }
}
```

- [ ] **步骤 8：运行 selection 测试和平台构建检查**

运行：

```bash
cd src-tauri && cargo test selection
cd src-tauri && cargo check
```

预期：当前平台上测试通过，`cargo check` 成功。

- [ ] **步骤 9：提交**

运行：

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/selection
git commit -m "feat: add system selection readers"
```

---

### 任务 4：实现 selection popup 窗口命令

**文件：**
- 修改：`src-tauri/src/commands/window.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：编写失败的窗口 URL 测试**

替换 `src-tauri/src/commands/window.rs`：

```rust
#[tauri::command]
pub fn open_float_window() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_selection_popup_url_with_encoded_text() {
        let url = selection_popup_url("hello world");
        assert_eq!(url, "/selection-popup?text=hello%20world");
    }

    #[test]
    fn builds_selection_popup_url_with_unicode_text() {
        let url = selection_popup_url("你好");
        assert_eq!(url, "/selection-popup?text=%E4%BD%A0%E5%A5%BD");
    }
}
```

- [ ] **步骤 2：运行测试，确认失败**

运行：

```bash
cd src-tauri && cargo test commands::window
```

预期：失败，错误中出现缺少 `selection_popup_url`。

- [ ] **步骤 3：实现窗口命令**

替换 `src-tauri/src/commands/window.rs`：

```rust
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const SELECTION_POPUP_LABEL: &str = "selection-popup";

#[tauri::command]
pub fn open_float_window() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn open_selection_popup(app: AppHandle, text: String) -> Result<(), String> {
    open_or_update_selection_popup(&app, &text)
}

pub fn open_or_update_selection_popup(app: &AppHandle, text: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SELECTION_POPUP_LABEL) {
        window
            .emit("selection-popup://text-updated", text)
            .map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let url = selection_popup_url(text);
    let window = WebviewWindowBuilder::new(
        app,
        SELECTION_POPUP_LABEL,
        WebviewUrl::App(url.into()),
    )
    .title("Bugoo Selection")
    .inner_size(320.0, 140.0)
    .min_inner_size(220.0, 96.0)
    .decorations(false)
    .always_on_top(true)
    .resizable(false)
    .visible(true)
    .build()
    .map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn selection_popup_url(text: &str) -> String {
    format!("/selection-popup?text={}", urlencoding::encode(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_selection_popup_url_with_encoded_text() {
        let url = selection_popup_url("hello world");
        assert_eq!(url, "/selection-popup?text=hello%20world");
    }

    #[test]
    fn builds_selection_popup_url_with_unicode_text() {
        let url = selection_popup_url("你好");
        assert_eq!(url, "/selection-popup?text=%E4%BD%A0%E5%A5%BD");
    }
}
```

修改 `src-tauri/src/lib.rs` 的 `invoke_handler` 列表，在 `open_float_window` 后加入：

```rust
commands::window::open_float_window,
commands::window::open_selection_popup,
```

- [ ] **步骤 4：运行测试和构建检查**

运行：

```bash
cd src-tauri && cargo test commands::window
cd src-tauri && cargo check
```

预期：窗口测试通过，`cargo check` 成功。

- [ ] **步骤 5：提交**

运行：

```bash
git add src-tauri/src/commands/window.rs src-tauri/src/lib.rs
git commit -m "feat: add selection popup window command"
```

---

### 任务 5：启动全局选区监听

**文件：**
- 修改：`src-tauri/src/selection/mod.rs`
- 创建：`src-tauri/src/selection/listener.rs`
- 修改：`src-tauri/src/lib.rs`

- [ ] **步骤 1：声明 listener 模块**

替换 `src-tauri/src/selection/mod.rs`：

```rust
pub mod clipboard;
pub mod filter;
pub mod listener;
pub mod platform;
pub mod reader;
pub mod types;
```

- [ ] **步骤 2：实现 listener**

创建 `src-tauri/src/selection/listener.rs`：

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rdev::{listen, Button, Event, EventType};
use tauri::{async_runtime, AppHandle};

use crate::commands::window::open_or_update_selection_popup;
use crate::selection::clipboard::TauriClipboardSelectionReader;
use crate::selection::filter::SelectionFilter;
use crate::selection::platform::SystemSelectionReader;
use crate::selection::reader::read_selected_text;

pub fn start_selection_listener(app: AppHandle) {
    let filter = Arc::new(Mutex::new(SelectionFilter::new()));

    thread::spawn(move || {
        let callback_app = app.clone();
        let callback_filter = filter.clone();
        let result = listen(move |event| {
            handle_global_event(event, callback_app.clone(), callback_filter.clone());
        });

        if let Err(error) = result {
            log::warn!("Failed to start global selection listener: {error:?}");
        }
    });
}

fn handle_global_event(event: Event, app: AppHandle, filter: Arc<Mutex<SelectionFilter>>) {
    if !matches!(event.event_type, EventType::ButtonRelease(Button::Left)) {
        return;
    }

    async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;

        let platform_reader = SystemSelectionReader::default();
        let clipboard_reader = TauriClipboardSelectionReader::new(app.clone());

        let selected_text = match read_selected_text(&platform_reader, &clipboard_reader) {
            Ok(Some(text)) => text,
            Ok(None) => return,
            Err(error) => {
                log::warn!("Failed to read selected text: {error}");
                return;
            }
        };

        let candidate = {
            let mut filter = match filter.lock() {
                Ok(filter) => filter,
                Err(error) => {
                    log::warn!("Selection filter lock poisoned: {error}");
                    return;
                }
            };
            filter.accept(&selected_text, Instant::now())
        };

        if let Some(candidate) = candidate {
            if let Err(error) = open_or_update_selection_popup(&app, &candidate.text) {
                log::warn!("Failed to open selection popup: {error}");
            }
        }
    });
}
```

- [ ] **步骤 3：在 Tauri setup 中启动 listener**

修改 `src-tauri/src/lib.rs`。

在 scheduler import 附近加入：

```rust
use crate::selection::listener::start_selection_listener;
```

在 `.setup(|app| { ... })` 内，`info!("Database initialized successfully");` 后加入：

```rust
start_selection_listener(app.handle().clone());
info!("Selection listener started");
```

- [ ] **步骤 4：运行构建检查和重点测试**

运行：

```bash
cd src-tauri && cargo test selection commands::window
cd src-tauri && cargo check
```

预期：重点测试通过，`cargo check` 成功。

- [ ] **步骤 5：提交**

运行：

```bash
git add src-tauri/src/lib.rs src-tauri/src/selection/mod.rs src-tauri/src/selection/listener.rs
git commit -m "feat: start selection listener"
```

---

### 任务 6：新增 selection popup 前端页面

**文件：**
- 修改：`vite.config.ts`
- 修改：`src/App.tsx`
- 创建：`src/pages/SelectionPopup/index.tsx`
- 创建：`src/pages/SelectionPopup/SelectionText.tsx`
- 创建：`src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`

- [ ] **步骤 1：配置 Vitest jsdom 环境**

修改 `vite.config.ts`：

```ts
/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { createSvgIconsPlugin } from "vite-plugin-svg-icons";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [
    react(),
    tailwindcss(),
    createSvgIconsPlugin({
      iconDirs: ["./src/assets/svg"],
      symbolId: "icon-[name]",
    }),
  ],
  resolve: {
    alias: {
      "@src": path.resolve(__dirname, "./src"),
      "@components": path.resolve(__dirname, "./src/components"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    environment: "jsdom",
  },
}));
```

- [ ] **步骤 2：编写失败的前端测试**

创建 `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`：

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { SelectionPopupPage } from "../index";

const listenMock = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

function renderPopup(initialEntry: string) {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <Routes>
        <Route path="/selection-popup" element={<SelectionPopupPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("SelectionPopupPage", () => {
  it("renders selected text from the query string", () => {
    listenMock.mockResolvedValueOnce(() => undefined);

    renderPopup("/selection-popup?text=hello%20world");

    expect(screen.getByText("hello world")).toBeInTheDocument();
  });

  it("renders a safe empty state when text is missing", () => {
    listenMock.mockResolvedValueOnce(() => undefined);

    renderPopup("/selection-popup");

    expect(screen.getByText("未读取到选中文本")).toBeInTheDocument();
  });

  it("updates displayed text from the Tauri event", async () => {
    let eventHandler: ((event: { payload: string }) => void) | undefined;
    listenMock.mockImplementationOnce((_eventName, handler) => {
      eventHandler = handler as (event: { payload: string }) => void;
      return Promise.resolve(() => undefined);
    });

    renderPopup("/selection-popup?text=old");
    eventHandler?.({ payload: "new text" });

    await waitFor(() => {
      expect(screen.getByText("new text")).toBeInTheDocument();
    });
  });
});
```

- [ ] **步骤 3：运行前端测试，确认失败**

运行：

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

预期：失败，因为 `SelectionPopupPage` 尚不存在；下一步会把 matcher 换成不依赖 jest-dom 的写法。

- [ ] **步骤 4：新增 SelectionText 组件**

创建 `src/pages/SelectionPopup/SelectionText.tsx`：

```tsx
import { Card } from "@heroui/react";

type SelectionTextProps = {
  text: string;
};

export function SelectionText({ text }: SelectionTextProps) {
  const displayText = text.trim();

  return (
    <Card className="h-full w-full border border-divider bg-background shadow-lg">
      <div className="flex h-full min-h-24 max-h-40 w-80 max-w-80 items-center p-4">
        {displayText ? (
          <p className="max-h-32 w-full overflow-hidden break-words text-sm leading-6 text-foreground">
            {displayText}
          </p>
        ) : (
          <p className="w-full text-center text-sm text-foreground-500">
            未读取到选中文本
          </p>
        )}
      </div>
    </Card>
  );
}
```

- [ ] **步骤 5：新增 SelectionPopup 页面**

创建 `src/pages/SelectionPopup/index.tsx`：

```tsx
import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useLocation } from "react-router-dom";
import { SelectionText } from "./SelectionText";

const TEXT_UPDATED_EVENT = "selection-popup://text-updated";

export function SelectionPopupPage() {
  const location = useLocation();
  const initialText = useMemo(() => {
    const params = new URLSearchParams(location.search);
    return params.get("text") ?? "";
  }, [location.search]);
  const [text, setText] = useState(initialText);

  useEffect(() => {
    setText(initialText);
  }, [initialText]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    listen<string>(TEXT_UPDATED_EVENT, (event) => {
      setText(event.payload);
    })
      .then((dispose) => {
        if (disposed) {
          dispose();
        } else {
          unlisten = dispose;
        }
      })
      .catch((error) => {
        console.warn("Failed to listen for selection popup updates", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return (
    <main className="flex min-h-screen items-center justify-center bg-transparent p-2">
      <SelectionText text={text} />
    </main>
  );
}
```

- [ ] **步骤 6：在 App 中增加路由**

修改 `src/App.tsx` imports：

```tsx
import { SelectionPopupPage } from "./pages/SelectionPopup";
```

修改 routes：

```tsx
<Routes>
  <Route path="/" element={<HomePage />} />
  <Route path="/settings" element={<SettingsPage />} />
  <Route path="/selection-popup" element={<SelectionPopupPage />} />
</Routes>
```

- [ ] **步骤 7：替换测试中的 jest-dom matcher**

替换 `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`：

```tsx
import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { SelectionPopupPage } from "../index";

const listenMock = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

function renderPopup(initialEntry: string) {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <Routes>
        <Route path="/selection-popup" element={<SelectionPopupPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("SelectionPopupPage", () => {
  it("renders selected text from the query string", () => {
    listenMock.mockResolvedValueOnce(() => undefined);

    renderPopup("/selection-popup?text=hello%20world");

    expect(screen.getByText("hello world")).toBeTruthy();
  });

  it("renders a safe empty state when text is missing", () => {
    listenMock.mockResolvedValueOnce(() => undefined);

    renderPopup("/selection-popup");

    expect(screen.getByText("未读取到选中文本")).toBeTruthy();
  });

  it("updates displayed text from the Tauri event", async () => {
    let eventHandler: ((event: { payload: string }) => void) | undefined;
    listenMock.mockImplementationOnce((_eventName, handler) => {
      eventHandler = handler as (event: { payload: string }) => void;
      return Promise.resolve(() => undefined);
    });

    renderPopup("/selection-popup?text=old");
    eventHandler?.({ payload: "new text" });

    await waitFor(() => {
      expect(screen.getByText("new text")).toBeTruthy();
    });
  });
});
```

- [ ] **步骤 8：运行前端测试和类型构建**

运行：

```bash
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
pnpm build
```

预期：popup 测试通过，TypeScript/Vite 构建成功。

- [ ] **步骤 9：提交**

运行：

```bash
git add vite.config.ts src/App.tsx src/pages/SelectionPopup
git commit -m "feat: add selection popup page"
```

---

### 任务 7：端到端验证和平台手测

**文件：**
- 只有验证发现明确编译或测试问题时才修改代码。

- [ ] **步骤 1：运行 Rust 验证**

运行：

```bash
cd src-tauri && cargo test
cd src-tauri && cargo check
```

预期：Rust 测试全部通过，`cargo check` 成功。

- [ ] **步骤 2：运行前端验证**

运行：

```bash
pnpm test
pnpm build
```

预期：前端测试全部通过，生产构建成功。

- [ ] **步骤 3：运行 Tauri 应用**

运行：

```bash
pnpm tauri dev
```

预期：Bugoo 正常启动，不 panic；日志中出现 `Selection listener started`。

- [ ] **步骤 4：macOS 手动验证**

在 macOS：

1. 打开 System Settings -> Privacy & Security -> Accessibility。
2. 给 Bugoo 或 `pnpm tauri dev` 使用的开发二进制授权。
3. 在 TextEdit、Safari 或其他普通应用里选中 50 个字符以内的文本。
4. 松开鼠标。
5. 确认 Bugoo 小弹窗出现，并且只显示选中的文本。
6. 触发前先复制一个已知文本；触发后粘贴到别处，确认进入剪贴板兜底时原剪贴板尽量恢复。

预期：不崩溃、不弹空窗；权限缺失时日志能读懂。

- [ ] **步骤 5：Windows 手动验证**

在 Windows：

1. 普通权限运行 Bugoo。
2. 在 Notepad 或浏览器普通文本页面中选中 50 个字符以内的文本。
3. 松开鼠标。
4. 确认 Bugoo 小弹窗出现，并且只显示选中的文本。
5. 如有条件，尝试管理员权限应用。

预期：普通应用能显示弹窗；管理员权限或受保护应用可能失败，但只记录日志，不弹空窗。

- [ ] **步骤 6：提交验证修复**

如果步骤 1 或步骤 2 需要代码修复，运行：

```bash
git add src-tauri src vite.config.ts
git commit -m "fix: stabilize selection popup verification"
```

预期：如果验证没有引发代码修改，就不创建提交。
