use crate::commands::AppState;
use crate::ports::outbound::translation::LibreTranslateLanguages;

#[tauri::command]
pub fn get_translation_languages(
    state: tauri::State<'_, AppState>,
) -> Result<LibreTranslateLanguages, String> {
    Ok(state.translation_service.libretranslate_languages().clone())
}
