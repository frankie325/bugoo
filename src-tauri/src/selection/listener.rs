use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(not(target_os = "macos"))]
use rdev::listen;
use rdev::{Button, Event, EventType};
use tauri::{async_runtime, AppHandle};

use crate::commands::window::open_or_update_selection_popup;
use crate::selection::clipboard::TauriClipboardSelectionReader;
use crate::selection::filter::SelectionFilter;
#[cfg(target_os = "macos")]
use crate::selection::platform::macos_events::listen_mouse_events;
use crate::selection::platform::SystemSelectionReader;
use crate::selection::reader::read_selected_text;
use crate::selection::types::SelectionReadError;

const SELECTION_READ_TIMEOUT: Duration = Duration::from_millis(320);
const SELECTION_READ_POLL_INTERVAL: Duration = Duration::from_millis(40);

pub fn start_selection_listener(app: AppHandle) {
    let filter = Arc::new(Mutex::new(SelectionFilter::new()));
    let mouse_state = Arc::new(Mutex::new(MouseState::default()));

    thread::spawn(move || {
        log::info!("Selection listener thread started");
        let callback_app = app.clone();
        let callback_filter = filter.clone();
        let callback_mouse_state = mouse_state.clone();
        let result = listen_selection_events(move |event| {
            let event_type = event.event_type;
            log::debug!("Selection listener received event: {event_type:?}");

            let result = catch_unwind(AssertUnwindSafe(|| {
                handle_global_event(
                    event,
                    callback_app.clone(),
                    callback_filter.clone(),
                    callback_mouse_state.clone(),
                );
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

#[cfg(target_os = "macos")]
fn listen_selection_events<T>(callback: T) -> Result<(), String>
where
    T: FnMut(Event) + 'static,
{
    listen_mouse_events(callback)
}

#[cfg(not(target_os = "macos"))]
fn listen_selection_events<T>(callback: T) -> Result<(), String>
where
    T: FnMut(Event) + 'static,
{
    listen(callback).map_err(|error| format!("{error:?}"))
}

#[derive(Debug, Default)]
struct MouseState {
    left_pressed: bool,
}

fn handle_global_event(
    event: Event,
    app: AppHandle,
    filter: Arc<Mutex<SelectionFilter>>,
    mouse_state: Arc<Mutex<MouseState>>,
) {
    match event.event_type {
        EventType::ButtonPress(Button::Left) => {
            log::info!("Selection listener observed left mouse press");
            if let Ok(mut state) = mouse_state.lock() {
                state.left_pressed = true;
            }
            return;
        }
        EventType::ButtonRelease(Button::Left) => {
            log::info!("Selection listener observed left mouse release");
            let should_process = if let Ok(mut state) = mouse_state.lock() {
                let should_process = state.left_pressed;
                state.left_pressed = false;
                should_process
            } else {
                false
            };

            if !should_process {
                log::info!("Selection listener ignored release without prior press");
                return;
            }
        }
        _ => return,
    }
    log::info!("triggered selection read on left mouse release event: {event:?}");

    // Only run selection read after a complete left-button press/release cycle
    // observed by this listener. This avoids a startup-only release event from
    // triggering an unexpected read when the app just opened.
    async_runtime::spawn(async move {
        log::info!("Selection trigger detected on left mouse release");

        let platform_reader = SystemSelectionReader;
        let clipboard_reader = TauriClipboardSelectionReader::new(app.clone());

        let selected_text =
            match read_selected_text_with_polling(&platform_reader, &clipboard_reader).await {
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
            log::info!("Selection accepted, opening/updating popup");
            if let Err(error) = open_or_update_selection_popup(&app, &candidate.text) {
                log::warn!("Failed to open selection popup: {error}");
            }
        } else {
            log::info!("Selection filtered out before popup update");
        }
    });
}

async fn read_selected_text_with_polling(
    platform_reader: &SystemSelectionReader,
    clipboard_reader: &TauriClipboardSelectionReader,
) -> Result<Option<String>, SelectionReadError> {
    let start = Instant::now();
    let mut last_error: Option<SelectionReadError> = None;

    loop {
        match read_selected_text(platform_reader, clipboard_reader) {
            Ok(Some(text)) => return Ok(Some(text)),
            Ok(None) => {}
            Err(SelectionReadError::PermissionDenied(message)) => {
                return Err(SelectionReadError::PermissionDenied(message));
            }
            Err(error) => {
                last_error = Some(error);
            }
        }

        if start.elapsed() >= SELECTION_READ_TIMEOUT {
            break;
        }

        tokio::time::sleep(SELECTION_READ_POLL_INTERVAL).await;
    }

    if let Some(error) = last_error {
        log::debug!("Selection read polling timed out after transient errors: {error}");
    }

    Ok(None)
}
