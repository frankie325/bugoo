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
    let conn = state.db.connection();

    let defaults = vec![
        ("theme", "light"),
        ("startup", "false"),
        ("closeBehavior", "minimize"),
        ("autoUpdate", "true"),
        ("language", "zh-CN"),
        ("dailyLimit", "20"),
        ("reviewPace", "normal"),
        ("hintStrategy", "progressive"),
        ("enableSelection", "true"),
        ("autoSpeak", "false"),
        ("autoClose", "true"),
        ("translationEngine", "local"),
        ("sourceLanguage", "auto"),
        ("targetLanguage", "zh"),
        ("apiEndpoint", ""),
        ("apiKey", ""),
        ("apiSecret", ""),
        ("apiRegion", ""),
        ("translationModel", ""),
        ("translationPrompt", ""),
        ("wordDetailPrompt", ""),
        ("translationTimeoutMs", "15000"),
        ("themeColor", "#10b981"),
        ("cardStyle", "rich"),
        ("fontSize", "medium"),
        ("reminderStartTime", "09:00"),
        ("reminderEndTime", "21:00"),
        ("notifyDailyReview", "true"),
        ("notifyForgetting", "true"),
        ("notifyStreak", "true"),
        ("notifyAchievement", "true"),
        ("shortcutStartReview", "Cmd+Enter"),
        ("shortcutTranslation", "Cmd+Shift+B"),
        ("shortcutNewWord", "Cmd+D"),
        ("shortcutOpenApp", "Cmd+K"),
    ];

    for (key, value) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            [key, value],
        )
        .map_err(|e| e.to_string())?;
    }

    drop(conn);
    state.settings_cache_reload()
}
