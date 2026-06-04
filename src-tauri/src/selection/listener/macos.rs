//! macOS 上的鼠标事件监听。桥接到 `selection::platform::macos_events`，
//! 后者基于 CoreGraphics event tap 暴露统一的 `SelectionMouseEvent` 序列。

use crate::selection::mouse_event::SelectionMouseEvent;
use crate::selection::platform::macos_events::listen_mouse_events;

pub fn listen_selection_events<T>(callback: T) -> Result<(), String>
where
    T: FnMut(SelectionMouseEvent) + Send + 'static,
{
    listen_mouse_events(callback)
}

pub fn listener_backend_name() -> &'static str {
    "macos-event-tap"
}
