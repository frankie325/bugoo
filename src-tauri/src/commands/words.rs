use crate::commands::word_details::{get_word_detail_by_id, WordDetail};
use crate::commands::AppState;
use crate::domain::models::{EnglishDefinitionGroup, Word, WordFormItem, WordMeaning};
use crate::domain::services::word_service::{AddWordWithDetails, WordUpdate};
use crate::ports::outbound::translation::TranslationExample;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWordInput {
    pub word: String,
    pub translation: String,
    pub source_lang: String,
    pub target_lang: String,
    #[serde(default)]
    pub phonetic: Option<String>,
    #[serde(default)]
    pub meanings: Vec<WordMeaning>,
    #[serde(default)]
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    #[serde(default)]
    pub examples: Vec<TranslationExample>,
    #[serde(default)]
    pub word_forms: Vec<WordFormItem>,
    #[serde(default)]
    pub memory_tip: String,
    #[serde(default)]
    pub tags: String,
}

#[tauri::command]
pub fn add_word(
    state: tauri::State<'_, AppState>,
    input: AddWordInput,
) -> Result<WordDetail, String> {
    let word = state
        .word_service
        .add_word_with_details(AddWordWithDetails {
            word: input.word,
            translation: input.translation,
            source_lang: input.source_lang,
            target_lang: input.target_lang,
            phonetic: input.phonetic,
            meanings: input.meanings,
            english_definitions: input.english_definitions,
            examples: input.examples,
            word_forms: input.word_forms,
            memory_tip: input.memory_tip,
            tags: input.tags,
        })?;

    get_word_detail_by_id(state.inner(), &word.id)?.ok_or_else(|| "单词保存失败".to_string())
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
