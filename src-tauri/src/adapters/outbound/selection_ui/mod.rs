mod auto_close;
mod geometry;
mod state;
mod window_adapter;

use std::sync::Arc;

use tauri::{AppHandle, Manager, PhysicalPosition};

use crate::ports::outbound::selection_ui::{
    SelectionPopupAnchor, SelectionUiHandle, SelectionUiPort,
};

pub use auto_close::SelectionPopupAutoCloseState;
pub use state::SelectionPopupTextState;
pub use window_adapter::open_accessibility_settings;

#[derive(Clone)]
pub struct TauriSelectionUi {
    app: AppHandle,
}

impl TauriSelectionUi {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl SelectionUiPort for TauriSelectionUi {
    fn initialize_selection_popup(&self) -> Result<(), String> {
        window_adapter::initialize_selection_popup_panel(&self.app)
    }

    fn open_or_update_selection_popup(&self, text: &str) -> Result<(), String> {
        window_adapter::open_or_update_selection_popup(&self.app, text)
    }

    fn open_or_update_selection_popup_at(
        &self,
        text: &str,
        anchor: SelectionPopupAnchor,
    ) -> Result<(), String> {
        window_adapter::open_or_update_selection_popup_at(
            &self.app,
            text,
            PhysicalPosition::new(anchor.x, anchor.y),
        )
    }

    fn close_selection_popup(&self) -> Result<(), String> {
        window_adapter::close_selection_popup(&self.app)
    }

    fn selection_popup_content_ready(&self, text: &str) -> Result<(), String> {
        window_adapter::selection_popup_content_ready(&self.app, text)
    }

    fn latest_selection_popup_text(&self) -> Option<String> {
        state::latest_selection_popup_text(&self.app)
    }

    fn is_cursor_inside_visible_selection_popup(&self) -> Result<bool, String> {
        window_adapter::is_cursor_inside_visible_selection_popup(&self.app)
    }

    fn focused_own_window_label(&self) -> Option<String> {
        window_adapter::focused_own_window_label(&self.app)
    }

    fn open_accessibility_permission_prompt(&self) -> Result<(), String> {
        window_adapter::open_accessibility_permission_window(&self.app)
    }

    fn close_accessibility_permission_prompt(&self) -> Result<(), String> {
        window_adapter::close_accessibility_permission_window(&self.app)
    }
}

pub fn manage_selection_ui(app: &AppHandle) -> Arc<dyn SelectionUiPort> {
    let selection_ui: Arc<dyn SelectionUiPort> = Arc::new(TauriSelectionUi::new(app.clone()));
    app.manage(SelectionPopupTextState::default());
    app.manage(SelectionPopupAutoCloseState::default());
    app.manage(SelectionUiHandle::new(selection_ui.clone()));
    selection_ui
}
