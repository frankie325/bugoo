use crate::adapters::outbound::translation::engine_languages::EngineLanguages;
use crate::commands::AppState;

#[tauri::command]
pub fn get_translation_languages(
    state: tauri::State<'_, AppState>,
    engine: String,
) -> Result<EngineLanguages, String> {
    state
        .translation_service
        .engine_languages()
        .get(&engine)
        .cloned()
        .ok_or_else(|| format!("No languages configured for engine: {}", engine))
}
