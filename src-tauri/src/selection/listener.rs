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
                log::info!("Selection read completed, but no text was available");
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
                log::info!("Selection filtered out before popup update");
                return;
            }
        };

        log::info!("Selection accepted, opening/updating popup");
        if let Err(error) = open_or_update_selection_popup(&app, &candidate.text) {
            log::warn!("Failed to open selection popup: {error}");
        }
    });
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
