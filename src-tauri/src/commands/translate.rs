use crate::commands::AppState;
use crate::ports::outbound::translation::TranslationResult;

#[tauri::command]
pub async fn translate_text(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<TranslationResult, String> {
    let settings = state.settings_cache_read()?;
    state.translation_service.translate(settings, text).await
}
