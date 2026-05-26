use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[cfg(not(target_os = "macos"))]
use rdev::listen;
#[cfg(not(target_os = "macos"))]
use rdev::{Button, Event, EventType};
use tauri::async_runtime;

use crate::ports::outbound::selection_ui::SelectionUiPort;
use crate::selection::debounce::{
    decide_debounce_action, mark_processing_finished, register_left_release_event,
    with_schedule_state, DebounceAction, ReleaseScheduleState,
};
use crate::selection::gesture::{
    classify_selection_gesture, SelectionGestureDecision, SelectionGestureState,
};
#[cfg(any(not(target_os = "macos"), test))]
use crate::selection::mouse_event::SelectionMouseEventKind;
use crate::selection::mouse_event::{MousePosition, SelectionMouseEvent};
#[cfg(target_os = "macos")]
use crate::selection::platform::macos_events::listen_mouse_events;
use crate::selection::processor::process_left_release_event;

#[cfg(test)]
use crate::selection::debounce::SELECTION_RELEASE_DEBOUNCE_MS;
#[cfg(test)]
use crate::selection::gesture::DOUBLE_CLICK_WINDOW_MS;
#[cfg(test)]
use crate::selection::processor::{
    decide_own_app_event_behavior, evaluate_popup_update, OwnAppEventBehavior, PopupCloseReason,
    PopupUpdateDecision,
};
#[cfg(test)]
use std::time::Duration;

pub fn start_selection_listener(selection_ui: Arc<dyn SelectionUiPort>) {
    let schedule_state = Arc::new(Mutex::new(ReleaseScheduleState::default()));

    thread::spawn(move || {
        log::info!(
            "Selection listener thread started with backend: {}",
            listener_backend_name()
        );
        let callback_selection_ui = selection_ui.clone();
        let callback_schedule_state = schedule_state.clone();
        let mut gesture_state = SelectionGestureState::default();
        let result = listen_selection_events(move |event| {
            let event_kind = event.kind;
            let release_position =
                match classify_selection_gesture(event, Instant::now(), &mut gesture_state) {
                    SelectionGestureDecision::Ignore => return,
                    SelectionGestureDecision::ReadSelection { release_position } => {
                        release_position
                    }
                };

            let callback_selection_ui = callback_selection_ui.clone();
            let callback_schedule_state = callback_schedule_state.clone();
            let result = catch_unwind(AssertUnwindSafe(|| {
                on_left_release_event(
                    callback_selection_ui,
                    callback_schedule_state,
                    release_position,
                );
            }));

            if result.is_err() {
                log::error!("Selection listener callback panicked while handling {event_kind:?}");
            }
        });

        if let Err(error) = result {
            log::warn!("Failed to start global selection listener: {error:?}");
        } else {
            log::warn!("Selection listener stopped unexpectedly");
        }
    });
}

fn on_left_release_event(
    selection_ui: Arc<dyn SelectionUiPort>,
    schedule_state: Arc<Mutex<ReleaseScheduleState>>,
    release_position: Option<MousePosition>,
) {
    let should_schedule = with_schedule_state(&schedule_state, |state| {
        register_left_release_event(Instant::now(), release_position, state)
    });

    if should_schedule {
        log::debug!("Selection debounce scheduled");
        schedule_debounce_task(selection_ui, schedule_state);
    }
}

fn schedule_debounce_task(
    selection_ui: Arc<dyn SelectionUiPort>,
    schedule_state: Arc<Mutex<ReleaseScheduleState>>,
) {
    async_runtime::spawn(async move {
        loop {
            let action = with_schedule_state(&schedule_state, |state| {
                decide_debounce_action(Instant::now(), state)
            });

            match action {
                DebounceAction::Wait(wait_for) => {
                    tokio::time::sleep(wait_for).await;
                }
                DebounceAction::StartProcessing(release_position) => {
                    schedule_processing_task(
                        selection_ui.clone(),
                        schedule_state.clone(),
                        release_position,
                    );
                    return;
                }
                DebounceAction::Stop => return,
            }
        }
    });
}

fn schedule_processing_task(
    selection_ui: Arc<dyn SelectionUiPort>,
    schedule_state: Arc<Mutex<ReleaseScheduleState>>,
    release_position: Option<MousePosition>,
) {
    async_runtime::spawn(async move {
        process_left_release_event(selection_ui.clone(), release_position);

        let should_schedule = with_schedule_state(&schedule_state, mark_processing_finished);
        if should_schedule {
            log::debug!("Selection processing replay scheduled for latest pending event");
            schedule_debounce_task(selection_ui, schedule_state);
        }
    });
}

#[cfg(target_os = "macos")]
fn listen_selection_events<T>(callback: T) -> Result<(), String>
where
    T: FnMut(SelectionMouseEvent) + Send + 'static,
{
    listen_mouse_events(callback)
}

#[cfg(not(target_os = "macos"))]
fn listen_selection_events<T>(callback: T) -> Result<(), String>
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

fn listener_backend_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macos-event-tap"
    }

    #[cfg(not(target_os = "macos"))]
    {
        "rdev"
    }
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn selection_event(
        kind: SelectionMouseEventKind,
        position: Option<MousePosition>,
    ) -> SelectionMouseEvent {
        SelectionMouseEvent {
            kind,
            position,
            time: SystemTime::UNIX_EPOCH,
        }
    }

    fn pos(x: f64, y: f64) -> MousePosition {
        MousePosition { x, y }
    }

    fn read_selection_position(decision: SelectionGestureDecision) -> Option<MousePosition> {
        match decision {
            SelectionGestureDecision::ReadSelection { release_position } => release_position,
            SelectionGestureDecision::Ignore => panic!("expected read selection decision"),
        }
    }

    #[test]
    fn ordinary_single_click_is_ignored() {
        let now = Instant::now();
        let mut state = SelectionGestureState::default();

        assert_eq!(
            classify_selection_gesture(
                selection_event(SelectionMouseEventKind::LeftDown, Some(pos(10.0, 10.0))),
                now,
                &mut state,
            ),
            SelectionGestureDecision::Ignore
        );
        assert_eq!(
            classify_selection_gesture(
                selection_event(
                    SelectionMouseEventKind::LeftUp {
                        click_count: Some(1),
                    },
                    Some(pos(10.0, 10.0)),
                ),
                now + Duration::from_millis(80),
                &mut state,
            ),
            SelectionGestureDecision::Ignore
        );
    }

    #[test]
    fn movement_below_drag_threshold_is_ignored() {
        let now = Instant::now();
        let mut state = SelectionGestureState::default();

        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDown, Some(pos(0.0, 0.0))),
            now,
            &mut state,
        );
        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::Move, Some(pos(5.0, 0.0))),
            now + Duration::from_millis(10),
            &mut state,
        );

        assert_eq!(
            classify_selection_gesture(
                selection_event(
                    SelectionMouseEventKind::LeftUp {
                        click_count: Some(1),
                    },
                    Some(pos(5.0, 0.0)),
                ),
                now + Duration::from_millis(20),
                &mut state,
            ),
            SelectionGestureDecision::Ignore
        );
    }

    #[test]
    fn drag_release_triggers_read() {
        let now = Instant::now();
        let mut state = SelectionGestureState::default();

        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDown, Some(pos(0.0, 0.0))),
            now,
            &mut state,
        );
        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDragged, Some(pos(6.0, 0.0))),
            now + Duration::from_millis(10),
            &mut state,
        );

        let decision = classify_selection_gesture(
            selection_event(
                SelectionMouseEventKind::LeftUp {
                    click_count: Some(1),
                },
                Some(pos(6.0, 0.0)),
            ),
            now + Duration::from_millis(20),
            &mut state,
        );

        assert_eq!(read_selection_position(decision), Some(pos(6.0, 0.0)));
    }

    #[test]
    fn drag_release_without_position_uses_latest_position() {
        let now = Instant::now();
        let mut state = SelectionGestureState::default();

        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDown, Some(pos(0.0, 0.0))),
            now,
            &mut state,
        );
        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDragged, Some(pos(8.0, 0.0))),
            now + Duration::from_millis(10),
            &mut state,
        );

        let decision = classify_selection_gesture(
            selection_event(
                SelectionMouseEventKind::LeftUp {
                    click_count: Some(1),
                },
                None,
            ),
            now + Duration::from_millis(20),
            &mut state,
        );

        assert_eq!(read_selection_position(decision), Some(pos(8.0, 0.0)));
    }

    #[test]
    fn native_double_click_release_triggers_read() {
        let mut state = SelectionGestureState::default();
        let decision = classify_selection_gesture(
            selection_event(
                SelectionMouseEventKind::LeftUp {
                    click_count: Some(2),
                },
                Some(pos(12.0, 12.0)),
            ),
            Instant::now(),
            &mut state,
        );

        assert_eq!(read_selection_position(decision), Some(pos(12.0, 12.0)));
    }

    #[test]
    fn native_triple_click_release_triggers_read() {
        let mut state = SelectionGestureState::default();
        let decision = classify_selection_gesture(
            selection_event(
                SelectionMouseEventKind::LeftUp {
                    click_count: Some(3),
                },
                Some(pos(12.0, 12.0)),
            ),
            Instant::now(),
            &mut state,
        );

        assert_eq!(read_selection_position(decision), Some(pos(12.0, 12.0)));
    }

    #[test]
    fn rdev_inferred_double_click_release_triggers_read() {
        let now = Instant::now();
        let mut state = SelectionGestureState::default();

        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDown, Some(pos(10.0, 10.0))),
            now,
            &mut state,
        );
        assert_eq!(
            classify_selection_gesture(
                selection_event(
                    SelectionMouseEventKind::LeftUp { click_count: None },
                    Some(pos(10.0, 10.0))
                ),
                now + Duration::from_millis(20),
                &mut state,
            ),
            SelectionGestureDecision::Ignore
        );
        classify_selection_gesture(
            selection_event(SelectionMouseEventKind::LeftDown, Some(pos(12.0, 12.0))),
            now + Duration::from_millis(320),
            &mut state,
        );

        let decision = classify_selection_gesture(
            selection_event(
                SelectionMouseEventKind::LeftUp { click_count: None },
                Some(pos(12.0, 12.0)),
            ),
            now + Duration::from_millis(340),
            &mut state,
        );

        assert_eq!(read_selection_position(decision), Some(pos(12.0, 12.0)));
    }

    #[test]
    fn rdev_slow_second_click_is_ignored() {
        let now = Instant::now();
        let mut state = SelectionGestureState::default();

        classify_selection_gesture(
            selection_event(
                SelectionMouseEventKind::LeftUp { click_count: None },
                Some(pos(10.0, 10.0)),
            ),
            now,
            &mut state,
        );

        assert_eq!(
            classify_selection_gesture(
                selection_event(
                    SelectionMouseEventKind::LeftUp { click_count: None },
                    Some(pos(10.0, 10.0))
                ),
                now + Duration::from_millis(DOUBLE_CLICK_WINDOW_MS + 1),
                &mut state,
            ),
            SelectionGestureDecision::Ignore
        );
    }

    #[test]
    fn non_left_or_keyboard_events_are_ignored() {
        let mut state = SelectionGestureState::default();

        assert_eq!(
            classify_selection_gesture(
                selection_event(SelectionMouseEventKind::Other, None),
                Instant::now(),
                &mut state,
            ),
            SelectionGestureDecision::Ignore
        );
    }

    #[test]
    fn evaluate_popup_update_closes_on_empty_selection() {
        let decision = evaluate_popup_update(None, Instant::now());
        assert_eq!(
            decision,
            PopupUpdateDecision::Close(PopupCloseReason::EmptySelection)
        );
    }

    #[test]
    fn evaluate_popup_update_closes_when_filtered_out() {
        let decision = evaluate_popup_update(Some("   "), Instant::now());
        assert_eq!(
            decision,
            PopupUpdateDecision::Close(PopupCloseReason::FilteredOut)
        );
    }

    #[test]
    fn own_popup_focus_ignores_without_closing() {
        assert_eq!(
            decide_own_app_event_behavior(Some("float-selection-popup"), false),
            OwnAppEventBehavior::IgnoreKeepPopup
        );
    }

    #[test]
    fn cursor_inside_popup_ignores_without_closing_even_without_focus() {
        assert_eq!(
            decide_own_app_event_behavior(None, true),
            OwnAppEventBehavior::IgnoreKeepPopup
        );
    }

    #[test]
    fn main_window_focus_ignores_and_closes() {
        assert_eq!(
            decide_own_app_event_behavior(Some("main"), false),
            OwnAppEventBehavior::IgnoreAndClosePopup
        );
    }

    #[test]
    fn settings_or_permission_focus_ignores_and_closes() {
        assert_eq!(
            decide_own_app_event_behavior(Some("settings"), false),
            OwnAppEventBehavior::IgnoreAndClosePopup
        );
        assert_eq!(
            decide_own_app_event_behavior(Some("accessibility-permission"), false),
            OwnAppEventBehavior::IgnoreAndClosePopup
        );
    }

    #[test]
    fn no_focused_own_window_continues_flow() {
        assert_eq!(
            decide_own_app_event_behavior(None, false),
            OwnAppEventBehavior::ContinueSelectionFlow
        );
    }

    #[test]
    fn register_release_schedules_first_debounce() {
        let mut state = ReleaseScheduleState::default();
        assert!(register_left_release_event(
            Instant::now(),
            Some(pos(20.0, 30.0)),
            &mut state
        ));
        assert!(state.debounce_scheduled);
        assert_eq!(state.latest_release_position, Some(pos(20.0, 30.0)));
    }

    #[test]
    fn register_release_does_not_reschedule_when_debounce_exists() {
        let now = Instant::now();
        let mut state = ReleaseScheduleState {
            last_release_at: Some(now),
            debounce_scheduled: true,
            in_flight: false,
            pending_latest: false,
            latest_release_position: Some(pos(1.0, 1.0)),
        };
        assert!(!register_left_release_event(
            now,
            Some(pos(2.0, 2.0)),
            &mut state
        ));
        assert!(state.debounce_scheduled);
        assert_eq!(state.latest_release_position, Some(pos(2.0, 2.0)));
    }

    #[test]
    fn register_release_marks_pending_and_updates_position_when_in_flight() {
        let now = Instant::now();
        let mut state = ReleaseScheduleState {
            last_release_at: Some(now),
            debounce_scheduled: false,
            in_flight: true,
            pending_latest: false,
            latest_release_position: Some(pos(1.0, 1.0)),
        };

        assert!(register_left_release_event(
            now,
            Some(pos(9.0, 9.0)),
            &mut state
        ));
        assert!(state.pending_latest);
        assert_eq!(state.latest_release_position, Some(pos(9.0, 9.0)));
    }

    #[test]
    fn debounce_action_waits_until_window_elapsed() {
        let now = Instant::now();
        let mut state = ReleaseScheduleState {
            last_release_at: Some(now),
            debounce_scheduled: true,
            in_flight: false,
            pending_latest: false,
            latest_release_position: Some(pos(3.0, 4.0)),
        };

        let action = decide_debounce_action(
            now + Duration::from_millis(SELECTION_RELEASE_DEBOUNCE_MS - 1),
            &mut state,
        );
        assert!(matches!(action, DebounceAction::Wait(_)));
        assert!(state.debounce_scheduled);
    }

    #[test]
    fn debounce_action_starts_processing_after_window_elapsed() {
        let now = Instant::now();
        let mut state = ReleaseScheduleState {
            last_release_at: Some(now),
            debounce_scheduled: true,
            in_flight: false,
            pending_latest: false,
            latest_release_position: Some(pos(3.0, 4.0)),
        };

        let action = decide_debounce_action(
            now + Duration::from_millis(SELECTION_RELEASE_DEBOUNCE_MS),
            &mut state,
        );
        assert_eq!(action, DebounceAction::StartProcessing(Some(pos(3.0, 4.0))));
        assert!(state.in_flight);
        assert!(!state.debounce_scheduled);
    }

    #[test]
    fn debounce_action_marks_pending_when_processing_in_flight() {
        let now = Instant::now();
        let mut state = ReleaseScheduleState {
            last_release_at: Some(now),
            debounce_scheduled: true,
            in_flight: true,
            pending_latest: false,
            latest_release_position: Some(pos(3.0, 4.0)),
        };

        let action = decide_debounce_action(
            now + Duration::from_millis(SELECTION_RELEASE_DEBOUNCE_MS),
            &mut state,
        );
        assert_eq!(action, DebounceAction::Stop);
        assert!(state.pending_latest);
        assert!(!state.debounce_scheduled);
    }

    #[test]
    fn mark_processing_finished_replays_pending_latest() {
        let mut state = ReleaseScheduleState {
            last_release_at: Some(Instant::now()),
            debounce_scheduled: false,
            in_flight: true,
            pending_latest: true,
            latest_release_position: Some(pos(3.0, 4.0)),
        };

        assert!(mark_processing_finished(&mut state));
        assert!(!state.in_flight);
        assert!(!state.pending_latest);
        assert!(state.debounce_scheduled);
    }
}

#[cfg(test)]
mod routing_tests {
    #[test]
    fn non_macos_prefers_rdev_listener() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(super::listener_backend_name(), "rdev");

        #[cfg(target_os = "macos")]
        assert_eq!(super::listener_backend_name(), "macos-event-tap");
    }
}
