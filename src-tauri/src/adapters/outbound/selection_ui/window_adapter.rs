#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;
use std::sync::mpsc;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

#[cfg(not(target_os = "macos"))]
use tauri::Position;

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, TrackingAreaOptions,
    WebviewWindowExt as WebviewPanelExt,
};

#[cfg(target_os = "macos")]
use super::geometry::macos_top_left_point_from_physical_position;
use super::geometry::{
    calculate_popup_position, is_cursor_inside_window_bounds, is_position_inside_monitor,
    POPUP_OFFSET_PX,
};
use super::state::update_selection_popup_text;

const SELECTION_POPUP_LABEL: &str = "float-selection-popup";
const ACCESSIBILITY_PERMISSION_LABEL: &str = "accessibility-permission";
const POPUP_DEFAULT_WIDTH: u32 = 320;
const POPUP_DEFAULT_HEIGHT: u32 = 140;

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(SelectionPopupPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
        with: {
            tracking_area: {
                options: TrackingAreaOptions::new()
                    .active_always()
                    .mouse_entered_and_exited()
                    .cursor_update(),
                auto_resize: true
            }
        }
    })

    panel_event!(SelectionPopupPanelEventHandler {
        window_did_become_key(notification: &NSNotification) -> (),
        window_did_resign_key(notification: &NSNotification) -> ()
    })
}

pub fn open_or_update_selection_popup(app: &AppHandle, text: &str) -> Result<(), String> {
    let cursor = app.cursor_position().map_err(|error| error.to_string())?;
    open_or_update_selection_popup_at(app, text, cursor)
}

pub fn open_or_update_selection_popup_at(
    app: &AppHandle,
    text: &str,
    anchor: PhysicalPosition<f64>,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let text = text.to_string();
        run_on_main_thread_sync(app, move |app| {
            open_or_update_selection_popup_panel_at(&app, &text, anchor)
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        open_or_update_selection_popup_window_at(app, text, anchor)
    }
}

#[cfg(target_os = "macos")]
pub fn initialize_selection_popup_panel(app: &AppHandle) -> Result<(), String> {
    run_on_main_thread_sync(app, |app| {
        ensure_selection_popup_panel(&app)?;
        log::info!("Selection popup NSPanel initialized");
        Ok(())
    })
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_selection_popup_panel(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn open_or_update_selection_popup_window_at(
    app: &AppHandle,
    text: &str,
    anchor: PhysicalPosition<f64>,
) -> Result<(), String> {
    update_selection_popup_text(app, text);

    if let Some(window) = app.get_webview_window(SELECTION_POPUP_LABEL) {
        log::info!("Updating existing selection popup window");
        if let Err(error) = window.emit("selection-popup://text-updated", text) {
            log::warn!("Failed to emit selection popup text update: {error}");
        }
        position_selection_popup(&window, anchor)?;
        show_selection_popup_window(app, &window)?;
        return Ok(());
    }

    log::info!("Creating new selection popup window");
    let window = create_selection_popup_window(app, text)?;

    position_selection_popup(&window, anchor)?;
    show_selection_popup_window(app, &window)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_or_update_selection_popup_panel_at(
    app: &AppHandle,
    text: &str,
    anchor: PhysicalPosition<f64>,
) -> Result<(), String> {
    ensure_selection_popup_panel(app)?;
    update_selection_popup_text(app, text);

    let window = app
        .get_webview_window(SELECTION_POPUP_LABEL)
        .ok_or_else(|| "Selection popup window was not created".to_string())?;

    log::info!("Updating existing selection popup panel");
    if let Err(error) = window.emit("selection-popup://text-updated", text) {
        log::warn!("Failed to emit selection popup text update: {error}");
    }

    let target_position = selection_popup_target_position(&window, anchor)?;
    position_selection_popup_panel(app, target_position)?;
    show_selection_popup_window(app, &window)?;
    Ok(())
}

fn create_selection_popup_window(app: &AppHandle, text: &str) -> Result<WebviewWindow, String> {
    let url = selection_popup_url(text);

    WebviewWindowBuilder::new(app, SELECTION_POPUP_LABEL, WebviewUrl::App(url.into()))
        .title("Bugoo Selection")
        .inner_size(POPUP_DEFAULT_WIDTH as f64, POPUP_DEFAULT_HEIGHT as f64)
        .min_inner_size(220.0, 96.0)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())
}

fn show_selection_popup_window(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if let Ok(panel) = app.get_webview_panel(SELECTION_POPUP_LABEL) {
        panel.show();
        return Ok(());
    }

    window.show().map_err(|error| error.to_string())?;
    Ok(())
}

pub fn close_selection_popup(app: &AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let app = app.clone();
        run_on_main_thread_sync(&app, close_selection_popup_on_main_thread)
    }

    #[cfg(not(target_os = "macos"))]
    {
        close_selection_popup_window(app)
    }
}

fn close_selection_popup_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SELECTION_POPUP_LABEL) {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn close_selection_popup_on_main_thread(app: AppHandle) -> Result<(), String> {
    if let Ok(panel) = app.get_webview_panel(SELECTION_POPUP_LABEL) {
        panel.hide();
        return Ok(());
    }

    close_selection_popup_window(&app)
}

#[cfg(target_os = "macos")]
fn ensure_selection_popup_panel(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_panel(SELECTION_POPUP_LABEL).is_ok() {
        return Ok(());
    }

    let window = if let Some(window) = app.get_webview_window(SELECTION_POPUP_LABEL) {
        window
    } else {
        log::info!("Creating hidden selection popup window for NSPanel conversion");
        create_selection_popup_window(app, "")?
    };

    convert_selection_popup_window_to_panel(&window)
}

#[cfg(target_os = "macos")]
fn convert_selection_popup_window_to_panel(window: &WebviewWindow) -> Result<(), String> {
    let panel = window
        .to_panel::<SelectionPopupPanel>()
        .map_err(|error| error.to_string())?;

    let handler = SelectionPopupPanelEventHandler::new();
    let app_for_enter = window.app_handle().clone();
    handler.on_mouse_entered(move |_| {
        if let Ok(panel) = app_for_enter.get_webview_panel(SELECTION_POPUP_LABEL) {
            panel.make_key_window();
        }
    });

    let app_for_exit = window.app_handle().clone();
    handler.on_mouse_exited(move |_| {
        if let Ok(panel) = app_for_exit.get_webview_panel(SELECTION_POPUP_LABEL) {
            panel.resign_key_window();
        }
    });

    panel.set_level(PanelLevel::Floating.value());
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .full_screen_auxiliary()
            .can_join_all_spaces()
            .into(),
    );
    panel.set_hides_on_deactivate(false);
    panel.set_works_when_modal(true);
    panel.set_event_handler(Some(handler.as_ref()));
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_on_main_thread_sync<T, F>(app: &AppHandle, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(AppHandle) -> Result<T, String> + Send + 'static,
{
    if tauri_nspanel::objc2_foundation::MainThreadMarker::new().is_some() {
        return task(app.clone());
    }

    let app_for_task = app.clone();
    let (sender, receiver) = mpsc::channel();

    app.run_on_main_thread(move || {
        let result = task(app_for_task);
        let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;

    receiver.recv().map_err(|error| error.to_string())?
}

pub fn is_cursor_inside_visible_selection_popup(app: &AppHandle) -> Result<bool, String> {
    let Some(window) = app.get_webview_window(SELECTION_POPUP_LABEL) else {
        return Ok(false);
    };
    if !window.is_visible().map_err(|error| error.to_string())? {
        return Ok(false);
    }

    let cursor = app.cursor_position().map_err(|error| error.to_string())?;
    let window_position = window.outer_position().map_err(|error| error.to_string())?;
    let window_size = window.outer_size().map_err(|error| error.to_string())?;

    Ok(is_cursor_inside_window_bounds(
        cursor,
        window_position,
        window_size,
    ))
}

pub fn focused_own_window_label(app: &AppHandle) -> Option<String> {
    for (label, window) in app.webview_windows() {
        match window.is_focused() {
            Ok(true) => return Some(label),
            Ok(false) => {}
            Err(error) => {
                log::warn!("Failed to read focus state for window {label}: {error}");
            }
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn position_selection_popup(
    window: &WebviewWindow,
    anchor: PhysicalPosition<f64>,
) -> Result<(), String> {
    let target_position = selection_popup_target_position(window, anchor)?;

    window
        .set_position(Position::Physical(target_position))
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn position_selection_popup_panel(
    app: &AppHandle,
    target_position: PhysicalPosition<i32>,
) -> Result<(), String> {
    let panel = app
        .get_webview_panel(SELECTION_POPUP_LABEL)
        .map_err(|_| "Selection popup panel was not registered".to_string())?;
    let panel = panel.as_panel();
    let scale_factor = panel.backingScaleFactor();
    let point = macos_top_left_point_from_physical_position(
        target_position,
        scale_factor,
        CGDisplay::main().pixels_high() as f64,
    );

    panel.setFrameTopLeftPoint(tauri_nspanel::NSPoint::new(point.0, point.1));
    let frame = panel.frame();
    log::debug!(
        "Selection popup panel positioned: target_physical=({}, {}), appkit_top_left=({}, {}), frame_origin=({}, {}), frame_size=({}, {}), scale_factor={}",
        target_position.x,
        target_position.y,
        point.0,
        point.1,
        frame.origin.x,
        frame.origin.y,
        frame.size.width,
        frame.size.height,
        scale_factor,
    );
    Ok(())
}

fn selection_popup_target_position(
    window: &WebviewWindow,
    anchor: PhysicalPosition<f64>,
) -> Result<PhysicalPosition<i32>, String> {
    let popup_size = window
        .outer_size()
        .unwrap_or(PhysicalSize::new(POPUP_DEFAULT_WIDTH, POPUP_DEFAULT_HEIGHT));

    let target_position = if let Some(monitor) = window
        .available_monitors()
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|monitor| is_position_inside_monitor(anchor, *monitor.position(), *monitor.size()))
        .or(window
            .current_monitor()
            .map_err(|error| error.to_string())?)
        .or(window
            .primary_monitor()
            .map_err(|error| error.to_string())?)
    {
        calculate_popup_position(
            anchor,
            popup_size,
            *monitor.position(),
            *monitor.size(),
            POPUP_OFFSET_PX,
        )
    } else {
        let x = (anchor.x.round() as i32 + POPUP_OFFSET_PX).max(0);
        let y = (anchor.y.round() as i32 + POPUP_OFFSET_PX).max(0);
        PhysicalPosition::new(x, y)
    };

    Ok(target_position)
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

pub fn open_accessibility_settings() -> Result<(), String> {
    open_accessibility_settings_impl()
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
