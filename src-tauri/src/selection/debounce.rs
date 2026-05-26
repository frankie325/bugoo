use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::selection::mouse_event::MousePosition;

pub(super) const SELECTION_RELEASE_DEBOUNCE_MS: u64 = 200;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct ReleaseScheduleState {
    pub(super) last_release_at: Option<Instant>,
    pub(super) debounce_scheduled: bool,
    pub(super) in_flight: bool,
    pub(super) pending_latest: bool,
    pub(super) latest_release_position: Option<MousePosition>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum DebounceAction {
    Wait(Duration),
    StartProcessing(Option<MousePosition>),
    Stop,
}

pub(super) fn register_left_release_event(
    now: Instant,
    release_position: Option<MousePosition>,
    state: &mut ReleaseScheduleState,
) -> bool {
    state.last_release_at = Some(now);
    state.latest_release_position = release_position;
    if state.in_flight {
        state.pending_latest = true;
    }
    if state.debounce_scheduled {
        return false;
    }

    state.debounce_scheduled = true;
    true
}

pub(super) fn decide_debounce_action(
    now: Instant,
    state: &mut ReleaseScheduleState,
) -> DebounceAction {
    if !state.debounce_scheduled {
        return DebounceAction::Stop;
    }

    let Some(last_release_at) = state.last_release_at else {
        state.debounce_scheduled = false;
        return DebounceAction::Stop;
    };

    let debounce_window = Duration::from_millis(SELECTION_RELEASE_DEBOUNCE_MS);
    let elapsed = now.saturating_duration_since(last_release_at);
    if elapsed < debounce_window {
        return DebounceAction::Wait(debounce_window - elapsed);
    }

    state.debounce_scheduled = false;
    if state.in_flight {
        state.pending_latest = true;
        return DebounceAction::Stop;
    }

    state.in_flight = true;
    DebounceAction::StartProcessing(state.latest_release_position)
}

pub(super) fn mark_processing_finished(state: &mut ReleaseScheduleState) -> bool {
    state.in_flight = false;
    if !state.pending_latest {
        return false;
    }

    state.pending_latest = false;
    if state.debounce_scheduled {
        return false;
    }

    state.debounce_scheduled = true;
    true
}

pub(super) fn with_schedule_state<T>(
    schedule_state: &Arc<Mutex<ReleaseScheduleState>>,
    f: impl FnOnce(&mut ReleaseScheduleState) -> T,
) -> T {
    let mut guard = schedule_state
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    f(&mut guard)
}
