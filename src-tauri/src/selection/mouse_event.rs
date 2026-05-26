use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MousePosition {
    pub x: f64,
    pub y: f64,
}

impl MousePosition {
    pub(crate) fn distance_squared_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectionMouseEventKind {
    LeftDown,
    LeftDragged,
    LeftUp {
        click_count: Option<u8>,
    },
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    Move,
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SelectionMouseEvent {
    pub kind: SelectionMouseEventKind,
    pub position: Option<MousePosition>,
    pub time: SystemTime,
}

impl SelectionMouseEvent {
    pub(crate) fn new(
        kind: SelectionMouseEventKind,
        position: Option<MousePosition>,
        time: SystemTime,
    ) -> Self {
        Self {
            kind,
            position,
            time,
        }
    }
}
