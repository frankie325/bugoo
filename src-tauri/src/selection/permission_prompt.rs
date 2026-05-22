use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use tauri::{async_runtime, AppHandle, Emitter, Manager};

use crate::commands::window::{
    close_accessibility_permission_window, open_accessibility_permission_window,
};
use crate::selection::listener::start_selection_listener;
use crate::selection::permission::{accessibility_permission, AccessibilityPermission};

const ACCESSIBILITY_PERMISSION_GRANTED_EVENT: &str = "accessibility-permission://granted";
const PERMISSION_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStartupAction {
    StartListener,
    OpenPermissionPrompt,
}

pub fn startup_action_for_permission(permission: AccessibilityPermission) -> SelectionStartupAction {
    match permission {
        AccessibilityPermission::Granted => SelectionStartupAction::StartListener,
        AccessibilityPermission::Missing => SelectionStartupAction::OpenPermissionPrompt,
    }
}

#[derive(Clone)]
pub struct SelectionRuntimeState {
    listener_started: Arc<AtomicBool>,
    permission_polling_cancelled: Arc<AtomicBool>,
}

impl Default for SelectionRuntimeState {
    fn default() -> Self {
        Self {
            listener_started: Arc::new(AtomicBool::new(false)),
            permission_polling_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl SelectionRuntimeState {
    pub fn cancel_permission_polling(&self) {
        self.permission_polling_cancelled.store(true, Ordering::SeqCst);
    }
}

pub fn initialize_selection(app: AppHandle) {
    let state = SelectionRuntimeState::default();
    app.manage(state.clone());

    match startup_action_for_permission(accessibility_permission()) {
        SelectionStartupAction::StartListener => start_listener_once(&app, &state),
        SelectionStartupAction::OpenPermissionPrompt => {
            if let Err(error) = open_accessibility_permission_window(&app) {
                log::warn!("Failed to open Accessibility permission window: {error}");
            }
            start_accessibility_permission_polling(app, state);
        }
    }
}

pub fn stop_accessibility_permission_polling(app: &AppHandle) {
    if let Some(state) = app.try_state::<SelectionRuntimeState>() {
        state.cancel_permission_polling();
    }
}

fn start_listener_once(app: &AppHandle, state: &SelectionRuntimeState) {
    if state
        .listener_started
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        start_selection_listener(app.clone());
    }
}

fn start_accessibility_permission_polling(app: AppHandle, state: SelectionRuntimeState) {
    async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(PERMISSION_POLL_INTERVAL).await;

            if state.permission_polling_cancelled.load(Ordering::SeqCst) {
                return;
            }

            if accessibility_permission() == AccessibilityPermission::Granted {
                state.cancel_permission_polling();
                if let Err(error) = close_accessibility_permission_window(&app) {
                    log::warn!("Failed to close Accessibility permission window: {error}");
                }
                start_listener_once(&app, &state);
                if let Err(error) = app.emit(ACCESSIBILITY_PERMISSION_GRANTED_EVENT, ()) {
                    log::warn!("Failed to emit Accessibility permission granted event: {error}");
                }
                return;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_listener_when_permission_is_granted() {
        assert_eq!(
            startup_action_for_permission(AccessibilityPermission::Granted),
            SelectionStartupAction::StartListener,
        );
    }

    #[test]
    fn opens_prompt_when_permission_is_missing() {
        assert_eq!(
            startup_action_for_permission(AccessibilityPermission::Missing),
            SelectionStartupAction::OpenPermissionPrompt,
        );
    }

    #[test]
    fn can_cancel_permission_polling() {
        let state = SelectionRuntimeState::default();
        assert!(!state.permission_polling_cancelled.load(Ordering::SeqCst));
        state.cancel_permission_polling();
        assert!(state.permission_polling_cancelled.load(Ordering::SeqCst));
    }
}
