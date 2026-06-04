//! 非 macOS 平台上的鼠标事件监听。基于 `rdev` crate，
//! 把 rdev 的 `EventType` 转换为项目内部的 `SelectionMouseEvent`。

use rdev::{listen, Button, Event, EventType};

use crate::selection::mouse_event::{MousePosition, SelectionMouseEvent, SelectionMouseEventKind};

pub fn listen_selection_events<T>(callback: T) -> Result<(), String>
where
    T: FnMut(SelectionMouseEvent) + Send + 'static,
{
    let mut last_position = None;
    let mut callback = callback;
    listen(move |event| {
        callback(selection_mouse_event_from_rdev(event, &mut last_position));
    })
    .map_err(|error| format!("{error:?}"))
}

pub fn listener_backend_name() -> &'static str {
    "rdev"
}

fn selection_mouse_event_from_rdev(
    event: Event,
    last_position: &mut Option<MousePosition>,
) -> SelectionMouseEvent {
    match event.event_type {
        EventType::ButtonPress(Button::Left) => SelectionMouseEvent::new(
            SelectionMouseEventKind::LeftDown,
            *last_position,
            event.time,
        ),
        EventType::ButtonRelease(Button::Left) => SelectionMouseEvent::new(
            SelectionMouseEventKind::LeftUp { click_count: None },
            *last_position,
            event.time,
        ),
        EventType::MouseMove { x, y } => {
            let position = MousePosition { x, y };
            *last_position = Some(position);
            SelectionMouseEvent::new(SelectionMouseEventKind::Move, Some(position), event.time)
        }
        _ => SelectionMouseEvent::new(SelectionMouseEventKind::Other, None, event.time),
    }
}
