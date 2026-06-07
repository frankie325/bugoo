use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionPopupAnchor {
    pub x: f64,
    pub y: f64,
}

pub trait SelectionUiPort: Send + Sync {
    fn initialize_selection_popup(&self) -> Result<(), String>;
    fn open_or_update_selection_popup(&self, text: &str) -> Result<(), String>;
    fn open_or_update_selection_popup_at(
        &self,
        text: &str,
        anchor: SelectionPopupAnchor,
    ) -> Result<(), String>;
    fn close_selection_popup(&self) -> Result<(), String>;
    fn selection_popup_content_ready(&self, text: &str) -> Result<(), String>;
    fn resize_selection_popup(&self, height: f64) -> Result<(), String>;
    fn latest_selection_popup_text(&self) -> Option<String>;
    fn is_cursor_inside_visible_selection_popup(&self) -> Result<bool, String>;
    fn focused_own_window_label(&self) -> Option<String>;
    fn open_accessibility_permission_prompt(&self) -> Result<(), String>;
    fn close_accessibility_permission_prompt(&self) -> Result<(), String>;
}

#[derive(Clone)]
pub struct SelectionUiHandle {
    inner: Arc<dyn SelectionUiPort>,
}

impl SelectionUiHandle {
    pub fn new(inner: Arc<dyn SelectionUiPort>) -> Self {
        Self { inner }
    }

    pub fn port(&self) -> Arc<dyn SelectionUiPort> {
        self.inner.clone()
    }
}
