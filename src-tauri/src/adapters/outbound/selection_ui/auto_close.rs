use std::sync::Mutex;
use std::time::Duration;

use tauri::{async_runtime, AppHandle, Manager};

const POPUP_AUTO_CLOSE_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug, Default)]
pub struct SelectionPopupAutoCloseState {
    inner: Mutex<AutoCloseInner>,
}

#[derive(Debug, Default)]
struct AutoCloseInner {
    token: u64,
    latest_text: Option<String>,
    content_ready: bool,
    cursor_inside: bool,
}

impl SelectionPopupAutoCloseState {
    fn mark_loading(&self, text: &str) -> Result<Option<u64>, String> {
        self.with_inner(|inner| {
            inner.token = inner.token.wrapping_add(1);
            let can_reuse_ready_content =
                inner.latest_text.as_deref() == Some(text) && inner.content_ready;
            inner.latest_text = Some(text.to_string());
            inner.content_ready = can_reuse_ready_content;
            inner.cursor_inside = false;
            (inner.content_ready && !inner.cursor_inside).then_some(inner.token)
        })
    }

    fn mark_content_ready(&self) -> Result<Option<u64>, String> {
        self.with_inner(|inner| {
            inner.token = inner.token.wrapping_add(1);
            inner.content_ready = true;
            (!inner.cursor_inside).then_some(inner.token)
        })
    }

    fn mark_mouse_entered(&self) -> Result<(), String> {
        self.with_inner(|inner| {
            inner.token = inner.token.wrapping_add(1);
            inner.cursor_inside = true;
        })
    }

    fn mark_mouse_exited(&self) -> Result<Option<u64>, String> {
        self.with_inner(|inner| {
            inner.token = inner.token.wrapping_add(1);
            inner.cursor_inside = false;
            inner.content_ready.then_some(inner.token)
        })
    }

    fn cancel(&self) -> Result<(), String> {
        self.with_inner(|inner| {
            inner.token = inner.token.wrapping_add(1);
        })
    }

    fn should_close(&self, token: u64) -> Result<bool, String> {
        self.with_inner(|inner| inner.token == token && inner.content_ready && !inner.cursor_inside)
    }

    fn with_inner<T>(&self, f: impl FnOnce(&mut AutoCloseInner) -> T) -> Result<T, String> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|error| format!("Selection popup auto-close state is poisoned: {error}"))?;
        Ok(f(&mut inner))
    }
}

pub(super) fn mark_popup_loading(app: &AppHandle, text: &str) -> Result<(), String> {
    if let Some(token) = auto_close_state(app)?.mark_loading(text)? {
        spawn_auto_close_timer(app.clone(), token);
    }
    Ok(())
}

pub(super) fn mark_popup_content_ready(app: &AppHandle, text: &str) -> Result<(), String> {
    let _ = text;
    if let Some(token) = auto_close_state(app)?.mark_content_ready()? {
        spawn_auto_close_timer(app.clone(), token);
    }
    Ok(())
}

pub(super) fn mark_popup_mouse_entered(app: &AppHandle) {
    if let Err(error) = auto_close_state(app).and_then(|state| state.mark_mouse_entered()) {
        log::warn!("Failed to mark selection popup mouse entered: {error}");
    }
}

pub(super) fn mark_popup_mouse_exited(app: &AppHandle) {
    match auto_close_state(app).and_then(|state| state.mark_mouse_exited()) {
        Ok(Some(token)) => spawn_auto_close_timer(app.clone(), token),
        Ok(None) => {}
        Err(error) => log::warn!("Failed to mark selection popup mouse exited: {error}"),
    }
}

pub(super) fn cancel_popup_auto_close(app: &AppHandle) {
    if let Err(error) = auto_close_state(app).and_then(|state| state.cancel()) {
        log::warn!("Failed to cancel selection popup auto-close: {error}");
    }
}

fn spawn_auto_close_timer(app: AppHandle, token: u64) {
    async_runtime::spawn(async move {
        tokio::time::sleep(POPUP_AUTO_CLOSE_DELAY).await;

        let should_close = match auto_close_state(&app).and_then(|state| state.should_close(token))
        {
            Ok(should_close) => should_close,
            Err(error) => {
                log::warn!("Failed to read selection popup auto-close state: {error}");
                false
            }
        };

        if !should_close {
            return;
        }

        if let Err(error) = super::window_adapter::close_selection_popup(&app) {
            log::warn!("Failed to auto close selection popup: {error}");
        }
    });
}

fn auto_close_state(
    app: &AppHandle,
) -> Result<tauri::State<'_, SelectionPopupAutoCloseState>, String> {
    app.try_state::<SelectionPopupAutoCloseState>()
        .ok_or_else(|| "Selection popup auto-close state is not managed".to_string())
}

#[cfg(test)]
mod tests {
    use super::SelectionPopupAutoCloseState;

    #[test]
    fn loading_state_does_not_schedule_close_until_content_is_ready() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();

        assert_eq!(state.mark_mouse_exited().unwrap(), None);
    }

    #[test]
    fn content_ready_schedules_close_when_cursor_is_outside() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();
        let token = state.mark_content_ready().unwrap();

        assert!(token.is_some());
        assert!(state.should_close(token.unwrap()).unwrap());
    }

    #[test]
    fn mouse_entered_cancels_pending_close() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();
        let token = state.mark_content_ready().unwrap().unwrap();
        state.mark_mouse_entered().unwrap();

        assert!(!state.should_close(token).unwrap());
    }

    #[test]
    fn mouse_exited_schedules_close_when_content_is_ready() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();
        state.mark_mouse_entered().unwrap();
        state.mark_content_ready().unwrap();
        let token = state.mark_mouse_exited().unwrap();

        assert!(token.is_some());
        assert!(state.should_close(token.unwrap()).unwrap());
    }

    #[test]
    fn content_ready_schedules_close_after_mouse_already_exited() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();
        state.mark_mouse_entered().unwrap();
        assert_eq!(state.mark_mouse_exited().unwrap(), None);
        let close_token = state.mark_content_ready().unwrap();

        assert!(close_token.is_some());
        assert!(state.should_close(close_token.unwrap()).unwrap());
    }

    #[test]
    fn content_ready_does_not_require_text_matching() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();

        assert!(state.mark_content_ready().unwrap().is_some());
    }

    #[test]
    fn reopening_same_ready_text_schedules_close_without_another_ready_event() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();
        state.mark_content_ready().unwrap();
        state.cancel().unwrap();
        let token = state.mark_loading("panel").unwrap();

        assert!(token.is_some());
        assert!(state.should_close(token.unwrap()).unwrap());
    }

    #[test]
    fn opening_different_text_waits_for_new_content_ready() {
        let state = SelectionPopupAutoCloseState::default();

        state.mark_loading("panel").unwrap();
        state.mark_content_ready().unwrap();
        state.cancel().unwrap();
        let token = state.mark_loading("window").unwrap();

        assert_eq!(token, None);
        assert_eq!(state.mark_mouse_exited().unwrap(), None);
    }
}
