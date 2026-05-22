use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const SELECTION_POPUP_LABEL: &str = "float-selection-popup";
const ACCESSIBILITY_PERMISSION_LABEL: &str = "accessibility-permission";

#[tauri::command]
pub fn open_float_window() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn open_selection_popup(app: AppHandle, text: String) -> Result<(), String> {
    open_or_update_selection_popup(&app, &text)
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    open_accessibility_settings_impl()
}

#[tauri::command]
pub fn dismiss_accessibility_permission_prompt(app: AppHandle) -> Result<(), String> {
    crate::selection::permission_prompt::stop_accessibility_permission_polling(&app);
    close_accessibility_permission_window(&app)
}

pub fn open_or_update_selection_popup(app: &AppHandle, text: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SELECTION_POPUP_LABEL) {
        log::info!("Updating existing selection popup window");
        window
            .emit("selection-popup://text-updated", text)
            .map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
        return Ok(());
    }

    log::info!("Creating new selection popup window");
    let url = selection_popup_url(text);
    let window = WebviewWindowBuilder::new(app, SELECTION_POPUP_LABEL, WebviewUrl::App(url.into()))
        .title("Bugoo Selection")
        .inner_size(320.0, 140.0)
        .min_inner_size(220.0, 96.0)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .visible(true)
        .build()
        .map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn selection_popup_url(text: &str) -> String {
    format!("/selection-popup?text={}", urlencoding::encode(text))
}

pub fn open_accessibility_permission_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ACCESSIBILITY_PERMISSION_LABEL) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        ACCESSIBILITY_PERMISSION_LABEL,
        WebviewUrl::App(accessibility_permission_url().into()),
    )
    .title("Bugoo Accessibility Permission")
    .inner_size(420.0, 240.0)
    .min_inner_size(360.0, 220.0)
    .decorations(true)
    .always_on_top(true)
    .resizable(false)
    .visible(true)
    .build()
    .map_err(|error| error.to_string())?;

    window.show().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn close_accessibility_permission_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ACCESSIBILITY_PERMISSION_LABEL) {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn accessibility_permission_url() -> String {
    "/accessibility-permission".to_string()
}

#[cfg(target_os = "macos")]
fn open_accessibility_settings_impl() -> Result<(), String> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn open_accessibility_settings_impl() -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_selection_popup_url_with_encoded_text() {
        let url = selection_popup_url("hello world");
        assert_eq!(url, "/selection-popup?text=hello%20world");
    }

    #[test]
    fn builds_selection_popup_url_with_unicode_text() {
        let url = selection_popup_url("你好");
        assert_eq!(url, "/selection-popup?text=%E4%BD%A0%E5%A5%BD");
    }

    #[test]
    fn builds_accessibility_permission_url() {
        assert_eq!(accessibility_permission_url(), "/accessibility-permission");
    }
}
