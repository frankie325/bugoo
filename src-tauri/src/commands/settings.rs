use crate::commands::AppState;
use std::collections::HashMap;

#[tauri::command]
pub fn get_settings(state: tauri::State<AppState>) -> Result<HashMap<String, String>, String> {
    state.settings_cache_read()
}

#[tauri::command]
pub fn set_setting(
    state: tauri::State<AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.db.connection();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [&key, &value],
    )
    .map_err(|e| e.to_string())?;

    let mut cache = state.settings_cache.write().map_err(|e| e.to_string())?;
    cache.insert(key, value);

    Ok(())
}

#[tauri::command]
pub fn seed_settings(state: tauri::State<AppState>) -> Result<(), String> {
    // Defaults are loaded from default-settings.json during database initialization.
    // This command just reloads the cache to pick up any changes.
    state.settings_cache_reload()
}
