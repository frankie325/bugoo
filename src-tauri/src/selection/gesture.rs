use std::time::{Duration, Instant};

use crate::selection::mouse_event::{MousePosition, SelectionMouseEvent, SelectionMouseEventKind};

pub(super) const DRAG_TRIGGER_DISTANCE_PX: f64 = 6.0;
pub(super) const DOUBLE_CLICK_WINDOW_MS: u64 = 500;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum SelectionGestureDecision {
    Ignore,
    ReadSelection {
        release_position: Option<MousePosition>,
    },
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct SelectionGestureState {
    left_down_position: Option<MousePosition>,
    last_position: Option<MousePosition>,
    dragged: bool,
    last_release: Option<ReleaseGestureSnapshot>,
}

#[derive(Debug, Clone, Copy)]
struct ReleaseGestureSnapshot {
    at: Instant,
    position: Option<MousePosition>,
}

pub(super) fn classify_selection_gesture(
    event: SelectionMouseEvent,
    now: Instant,
    state: &mut SelectionGestureState,
) -> SelectionGestureDecision {
    if let Some(position) = event.position {
        state.last_position = Some(position);
    }

    match event.kind {
        SelectionMouseEventKind::LeftDown => {
            state.left_down_position = event.position.or(state.last_position);
            state.dragged = false;
            SelectionGestureDecision::Ignore
        }
        SelectionMouseEventKind::LeftDragged | SelectionMouseEventKind::Move => {
            update_drag_state(event.position, state);
            SelectionGestureDecision::Ignore
        }
        SelectionMouseEventKind::LeftUp { click_count } => {
            let release_position = event.position.or(state.last_position);
            let is_drag_release = state.dragged;
            let is_native_multi_click = click_count.is_some_and(|count| count >= 2);
            let is_inferred_double_click =
                click_count.is_none() && is_rdev_double_click(now, release_position, state);

            state.last_release = Some(ReleaseGestureSnapshot {
                at: now,
                position: release_position,
            });
            state.left_down_position = None;
            state.dragged = false;

            if is_drag_release || is_native_multi_click || is_inferred_double_click {
                SelectionGestureDecision::ReadSelection { release_position }
            } else {
                SelectionGestureDecision::Ignore
            }
        }
        SelectionMouseEventKind::Other => SelectionGestureDecision::Ignore,
    }
}

fn update_drag_state(position: Option<MousePosition>, state: &mut SelectionGestureState) {
    if state.dragged {
        return;
    }

    let Some(start) = state.left_down_position else {
        return;
    };
    let Some(current) = position.or(state.last_position) else {
        return;
    };

    if start.distance_squared_to(current) >= DRAG_TRIGGER_DISTANCE_PX * DRAG_TRIGGER_DISTANCE_PX {
        state.dragged = true;
    }
}

fn is_rdev_double_click(
    now: Instant,
    release_position: Option<MousePosition>,
    state: &SelectionGestureState,
) -> bool {
    let Some(last_release) = state.last_release else {
        return false;
    };
    if now.saturating_duration_since(last_release.at)
        > Duration::from_millis(DOUBLE_CLICK_WINDOW_MS)
    {
        return false;
    }

    let Some(previous_position) = last_release.position else {
        return false;
    };
    let Some(current_position) = release_position else {
        return false;
    };

    previous_position.distance_squared_to(current_position)
        <= DRAG_TRIGGER_DISTANCE_PX * DRAG_TRIGGER_DISTANCE_PX
}
