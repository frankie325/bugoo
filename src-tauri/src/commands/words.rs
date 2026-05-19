use crate::domain::models::Word;
use crate::domain::services::word_service::WordUpdate;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn add_word(
    state: State<'_, AppState>,
    word: String,
    translation: String,
    source_lang: String,
    target_lang: String,
    tags: String,
) -> Result<Word, String> {
    state
        .word_service
        .add_word(word, translation, source_lang, target_lang, tags)
}

#[tauri::command]
pub fn get_words(state: State<'_, AppState>, search: Option<String>) -> Result<Vec<Word>, String> {
    state.word_service.get_words(search)
}

#[tauri::command]
pub fn delete_word(state: State<'_, AppState>, word_id: String) -> Result<(), String> {
    state.word_service.delete_word(&word_id)
}

#[tauri::command]
pub fn update_word(
    state: State<'_, AppState>,
    word_id: String,
    updates: WordUpdate,
) -> Result<Word, String> {
    state.word_service.update_word(&word_id, updates)
}
