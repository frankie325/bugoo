use tauri::{AppHandle, Manager};

use crate::adapters::outbound::selection_ui::open_accessibility_settings as open_accessibility_settings_impl;
use crate::ports::outbound::selection_ui::SelectionUiHandle;

#[tauri::command]
pub fn open_float_window() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn open_selection_popup(app: AppHandle, text: String) -> Result<(), String> {
    selection_ui(&app)?.open_or_update_selection_popup(&text)
}

#[tauri::command]
pub fn close_selection_popup(app: AppHandle) -> Result<(), String> {
    selection_ui(&app)?.close_selection_popup()
}

#[tauri::command]
pub fn selection_popup_content_ready(app: AppHandle, text: String) -> Result<(), String> {
    selection_ui(&app)?.selection_popup_content_ready(&text)
}

#[tauri::command]
pub fn get_selection_popup_text(app: AppHandle) -> Option<String> {
    selection_ui(&app)
        .ok()
        .and_then(|selection_ui| selection_ui.latest_selection_popup_text())
}

#[tauri::command]
pub fn is_cursor_inside_selection_popup(app: AppHandle) -> Result<bool, String> {
    selection_ui(&app)?.is_cursor_inside_visible_selection_popup()
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    open_accessibility_settings_impl()
}

#[tauri::command]
pub fn dismiss_accessibility_permission_prompt(app: AppHandle) -> Result<(), String> {
    crate::selection::permission_prompt::stop_accessibility_permission_polling(&app);
    selection_ui(&app)?.close_accessibility_permission_prompt()
}

fn selection_ui(
    app: &AppHandle,
) -> Result<std::sync::Arc<dyn crate::ports::outbound::selection_ui::SelectionUiPort>, String> {
    app.try_state::<SelectionUiHandle>()
        .map(|handle| handle.port())
        .ok_or_else(|| "Selection UI port is not managed".to_string())
}
