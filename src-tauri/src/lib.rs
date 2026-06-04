mod adapters;
mod commands;
mod db;
mod domain;
mod ports;
mod scheduler;
mod selection;

use crate::db::Database;
use crate::domain::services::translation_service::TranslationService;
use crate::ports::outbound::dictionary::DictionaryProvider;
use crate::scheduler::notification::start_notification_scheduler;
use crate::selection::permission_prompt::initialize_selection;
use adapters::outbound::config::engine_endpoints::read_engine_endpoints;
use adapters::outbound::dictionary::stardict_ecdict::StarDictEcdictDictionaryProvider;
use adapters::outbound::language_detection::libretranslate_detector::LibreTranslateLanguageDetector;
use adapters::outbound::selection_ui::manage_selection_ui;
use adapters::outbound::translation::engine_languages::read_engine_languages;
use commands::AppState;
use log::info;
use ports::outbound::translation::EngineEndpoints;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{async_runtime, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Bugoo application");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init());

    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
            let db_path: PathBuf = app_data_dir.join("bugoo.db");
            info!("Database path: {:?}", db_path);

            let database = Database::new(db_path).expect("Failed to initialize database");
            let db = Arc::new(database);
            let db_clone = db.clone();
            let app_handle = app.handle().clone();

            let dictionary_dir = app
                .path()
                .resolve("resources/stardict-ecdict", tauri::path::BaseDirectory::Resource)
                .unwrap_or_else(|_| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("resources")
                        .join("stardict-ecdict")
                });

            let dictionary_provider = match StarDictEcdictDictionaryProvider::new(
                dictionary_dir.clone(),
                "stardict-ecdict-2.4.2",
            ) {
                Ok(provider) => Some(Arc::new(provider) as Arc<dyn DictionaryProvider>),
                Err(error) => {
                    log::warn!(
                        "StarDict ECDICT dictionary unavailable at {:?}, dictionary lookup disabled: {}",
                        dictionary_dir,
                        error
                    );
                    None
                }
            };

            let engine_endpoints_path = app
                .path()
                .resolve(
                    "resources/translation/engine-endpoints.json",
                    tauri::path::BaseDirectory::Resource,
                )
                .unwrap_or_else(|_| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("resources")
                        .join("translation")
                        .join("engine-endpoints.json")
                });

            let engine_endpoints = match read_engine_endpoints(&engine_endpoints_path) {
                Ok(endpoints) => endpoints,
                Err(error) => {
                    log::warn!(
                        "Engine endpoints config unavailable at {:?}, using defaults: {}",
                        engine_endpoints_path,
                        error
                    );
                    EngineEndpoints::default()
                }
            };

            let translation_dir = app
                .path()
                .resolve(
                    "resources/translation",
                    tauri::path::BaseDirectory::Resource,
                )
                .unwrap_or_else(|_| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("resources")
                        .join("translation")
                });

            let mut engine_languages: HashMap<String, _> = HashMap::new();

            // Load all *-languages.json files from translation directory
            if let Ok(entries) = std::fs::read_dir(&translation_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            if filename.ends_with("-languages.json") {
                                match read_engine_languages(&path) {
                                    Ok(langs) => {
                                        let engine_name = filename.trim_end_matches("-languages.json");
                                        let key = if engine_name == "libretranslate" {
                                            "local"
                                        } else {
                                            engine_name
                                        };
                                        engine_languages.insert(key.to_string(), langs);
                                        info!("Loaded {} languages for engine '{}'", engine_name, filename);
                                    }
                                    Err(e) => {
                                        log::warn!("Failed to load {}: {}", filename, e);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                log::warn!("Translation directory not found: {:?}", translation_dir);
            }

            let translation_service = TranslationService::new(
                dictionary_provider,
                engine_endpoints.clone(),
                Arc::new(LibreTranslateLanguageDetector::new(
                    engine_endpoints.endpoint_or_default("local"),
                    15_000,
                )),
                engine_languages,
            );

            // 创建并管理 AppState
            let app_state = AppState::new(db.clone(), translation_service);
            app.manage(app_state);
            let selection_ui = manage_selection_ui(app.handle());

            async_runtime::spawn(async move {
                start_notification_scheduler(db_clone, app_handle).await;
            });
            info!("Database initialized successfully");

            #[cfg(target_os = "macos")]
            if let Err(error) = selection_ui.initialize_selection_popup() {
                log::warn!("Failed to initialize selection popup NSPanel: {}", error);
            }

            initialize_selection(app.handle().clone());
            info!("Selection service initialized");

            // 注册全局快捷键: CmdOrCtrl+Shift+T
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
            commands::words::add_word,
            commands::words::get_words,
            commands::words::delete_word,
            commands::words::update_word,
            commands::translate::translate_text,
            commands::word_details::get_word_detail,
            commands::word_details::resolve_word,
            commands::review::get_due_reviews,
            commands::review::submit_review,
            commands::tts::speak_text,
            commands::tts::stop_speech,
            commands::tts::list_voices,
            commands::tts::set_voice,
            commands::window::open_float_window,
            commands::window::open_selection_popup,
            commands::window::close_selection_popup,
            commands::window::selection_popup_content_ready,
            commands::window::resize_selection_popup,
            commands::window::get_selection_popup_text,
            commands::window::is_cursor_inside_selection_popup,
            commands::window::open_accessibility_settings,
            commands::window::dismiss_accessibility_permission_prompt,
            commands::settings::get_settings,
            commands::settings::set_setting,
            commands::settings::seed_settings,
            commands::tags::get_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::reorder_tags,
            commands::translation_languages::get_translation_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
