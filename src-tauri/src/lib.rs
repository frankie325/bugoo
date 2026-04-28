mod commands;
mod db;
mod scheduler;
mod tts;

use crate::db::Database;
use crate::scheduler::notification::start_notification_scheduler;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, Manager, async_runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Bugoo application");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
            let db_path: PathBuf = app_data_dir.join("bugoo.db");
            info!("Database path: {:?}", db_path);

            let database =
                Database::new(db_path).expect("Failed to initialize database");
            let db = Arc::new(database);
            let db_clone = db.clone();
            let app_handle = app.handle().clone();
            app.manage(db.clone());
            async_runtime::spawn(async move {
                start_notification_scheduler(db_clone, app_handle).await;
            });
            info!("Database initialized successfully");

            // Register global shortcut: CmdOrCtrl+Shift+T
            let app_handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut("CmdOrCtrl+Shift+T", move |_app, _shortcut, _event| {
                    info!("Global shortcut triggered");
                    if let Ok(clipboard_text) = app_handle.clipboard().read_text() {
                        if !clipboard_text.trim().is_empty() {
                            info!("Clipboard text: {}", clipboard_text);
                            let _ = app_handle.emit("trigger-translation", clipboard_text);
                        } else {
                            info!("Clipboard is empty");
                        }
                    } else {
                        info!("Failed to read clipboard");
                    }
                })
                .unwrap();

            info!("Tauri app setup complete");
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Bugoo").unwrap();
            } else {
                info!("Main window not found - this is normal in tray-only mode");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::translate::translate_text,
            commands::words::add_word,
            commands::words::get_words,
            commands::words::delete_word,
            commands::review::get_due_reviews,
            commands::review::submit_review,
            commands::tts::speak_text,
            commands::window::open_float_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
