use crate::commands::AppState;
use std::collections::HashMap;

#[tauri::command]
pub fn get_settings(state: tauri::State<AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.connection();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut settings = HashMap::new();
    for (key, value) in rows.flatten() {
        settings.insert(key, value);
    }
    Ok(settings)
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
    Ok(())
}

#[tauri::command]
pub fn seed_settings(state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.connection();

    let defaults = vec![
        // 通用设置
        ("theme", "light"),
        ("startup", "false"),
        ("closeBehavior", "minimize"),
        ("autoUpdate", "true"),
        ("language", "zh-CN"),
        // 学习设置
        ("dailyLimit", "20"),
        ("reviewPace", "normal"),
        ("hintStrategy", "progressive"),
        ("enableSelection", "true"),
        ("autoSpeak", "false"),
        ("autoClose", "true"),
        // 翻译设置
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
        // 外观设置
        ("themeColor", "#10b981"),
        ("cardStyle", "rich"),
        ("fontSize", "medium"),
        // 通知设置
        ("reminderStartTime", "09:00"),
        ("reminderEndTime", "21:00"),
        ("notifyDailyReview", "true"),
        ("notifyForgetting", "true"),
        ("notifyStreak", "true"),
        ("notifyAchievement", "true"),
        // 快捷键设置
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

    Ok(())
}
