use std::sync::Mutex;

use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct SelectionPopupTextState {
    latest_text: Mutex<Option<String>>,
}

pub(super) fn update_selection_popup_text(app: &AppHandle, text: &str) {
    let Some(state) = app.try_state::<SelectionPopupTextState>() else {
        log::warn!("Selection popup text state is not managed");
        return;
    };

    match state.latest_text.lock() {
        Ok(mut latest_text) => {
            *latest_text = Some(text.to_string());
        }
        Err(error) => {
            log::warn!("Failed to store latest selection popup text: {error}");
        }
    };
}

pub(super) fn latest_selection_popup_text(app: &AppHandle) -> Option<String> {
    app.try_state::<SelectionPopupTextState>()
        .and_then(|state| state.latest_text.lock().ok().and_then(|text| text.clone()))
}
