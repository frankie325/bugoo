use std::sync::Arc;
use std::time::Instant;

use crate::ports::outbound::selection_ui::{SelectionPopupAnchor, SelectionUiPort};
use crate::selection::filter::filter_selection_text;
use crate::selection::mouse_event::MousePosition;
use crate::selection::reader::read_selected_text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PopupCloseReason {
    EmptySelection,
    FilteredOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PopupUpdateDecision {
    Open(String),
    Close(PopupCloseReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum OwnAppEventBehavior {
    ContinueSelectionFlow,
    IgnoreKeepPopup,
    IgnoreAndClosePopup,
}

pub(super) fn process_left_release_event(
    selection_ui: Arc<dyn SelectionUiPort>,
    release_position: Option<MousePosition>,
) {
    let cursor_inside_popup = match selection_ui.is_cursor_inside_visible_selection_popup() {
        Ok(is_inside) => is_inside,
        Err(error) => {
            log::warn!("Failed to check whether cursor is inside selection popup: {error}");
            false
        }
    };
    let own_app_behavior = decide_own_app_event_behavior(
        selection_ui.focused_own_window_label().as_deref(),
        cursor_inside_popup,
    );
    match own_app_behavior {
        OwnAppEventBehavior::IgnoreKeepPopup => {
            log::info!("Selection event ignored in selection popup window");
            return;
        }
        OwnAppEventBehavior::IgnoreAndClosePopup => {
            log::info!("Selection event ignored in Bugoo window; closing selection popup");
            if let Err(error) = selection_ui.close_selection_popup() {
                log::warn!("Failed to close selection popup for own-app event: {error}");
            }
            return;
        }
        OwnAppEventBehavior::ContinueSelectionFlow => {}
    }
    log::info!("Selection trigger detected from drag or multi-click gesture");
    let update_decision = evaluate_popup_update(read_selected_text().as_deref(), Instant::now());

    match update_decision {
        PopupUpdateDecision::Open(text) => {
            log::info!("Selection accepted, opening/updating popup");
            let result = if let Some(position) = release_position {
                selection_ui.open_or_update_selection_popup_at(
                    &text,
                    SelectionPopupAnchor {
                        x: position.x,
                        y: position.y,
                    },
                )
            } else {
                selection_ui.open_or_update_selection_popup(&text)
            };

            if let Err(error) = result {
                log::warn!("Failed to open selection popup: {error}");
            }
        }
        PopupUpdateDecision::Close(PopupCloseReason::EmptySelection) => {
            log::info!("Selection read completed, but no text was available");
            if let Err(error) = selection_ui.close_selection_popup() {
                log::warn!("Failed to close selection popup on empty selection: {error}");
            };
        }
        PopupUpdateDecision::Close(PopupCloseReason::FilteredOut) => {
            log::info!("Selection filtered out before popup update");
            if let Err(error) = selection_ui.close_selection_popup() {
                log::warn!("Failed to close selection popup after filtering: {error}");
            };
        }
    }
}

pub(super) fn decide_own_app_event_behavior(
    focused_label: Option<&str>,
    cursor_inside_popup: bool,
) -> OwnAppEventBehavior {
    if cursor_inside_popup {
        return OwnAppEventBehavior::IgnoreKeepPopup;
    }

    match focused_label {
        Some("float-selection-popup") => OwnAppEventBehavior::IgnoreKeepPopup,
        Some(_) => OwnAppEventBehavior::IgnoreAndClosePopup,
        None => OwnAppEventBehavior::ContinueSelectionFlow,
    }
}

pub(super) fn evaluate_popup_update(
    selected_text: Option<&str>,
    now: Instant,
) -> PopupUpdateDecision {
    let Some(text) = selected_text else {
        return PopupUpdateDecision::Close(PopupCloseReason::EmptySelection);
    };

    match filter_selection_text(text, now) {
        Some(candidate) => PopupUpdateDecision::Open(candidate.text),
        None => PopupUpdateDecision::Close(PopupCloseReason::FilteredOut),
    }
}
