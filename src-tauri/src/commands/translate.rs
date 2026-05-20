use crate::commands::AppState;
use crate::domain::services::translation_service::TranslationService;
use crate::ports::outbound::translation::TranslationResult;
use std::collections::HashMap;

#[tauri::command]
pub async fn translate_text(
    state: tauri::State<'_, AppState>,
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<TranslationResult, String> {
    let settings = read_settings_map(state.inner())?;
    TranslationService::translate(settings, text, source_lang, target_lang).await
}

pub(crate) fn read_settings_map(state: &AppState) -> Result<HashMap<String, String>, String> {
    let conn = state.db.connection();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?;

    let mut settings = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(|error| error.to_string())?;
        settings.insert(key, value);
    }

    Ok(settings)
}