use crate::commands::AppState;
use crate::domain::models::{EnglishDefinitionGroup, Word, WordFormItem, WordMeaning};
use crate::ports::outbound::translation::{
    TranslationError, TranslationExample, TranslationResult,
};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct WordDetail {
    pub word_id: String,
    pub word: String,
    pub translation: String,
    pub phonetic: Option<String>,
    pub meanings: Vec<WordMeaning>,
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    pub examples: Vec<TranslationExample>,
    pub word_forms: Vec<WordFormItem>,
    pub memory_tip: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedWord {
    pub word_id: Option<String>,
    pub word: String,
    pub translation: String,
    pub detected_source_lang: Option<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub phonetic: Option<String>,
    pub meanings: Vec<WordMeaning>,
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    pub examples: Vec<TranslationExample>,
    pub word_forms: Vec<WordFormItem>,
    pub memory_tip: String,
}

#[tauri::command]
pub fn get_word_detail(
    state: tauri::State<AppState>,
    word_id: String,
) -> Result<Option<WordDetail>, String> {
    get_word_detail_by_id(state.inner(), &word_id)
}

#[tauri::command]
pub async fn resolve_word(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<ResolvedWord, String> {
    let app_state = state.inner();
    let query = text.trim().to_string();
    if query.is_empty() {
        return Err(TranslationError::EmptyText.to_string());
    }

    let settings = app_state.settings_cache_read()?;
    let source_setting = settings_value(&settings, "sourceLanguage", "auto");
    let target_lang = settings_value(&settings, "targetLanguage", "zh");

    if let Some(existing) = app_state
        .word_service
        .find_existing_word(&query, &target_lang)?
    {
        if let Some(detail) = get_word_detail_by_id(app_state, &existing.id)? {
            return Ok(word_detail_to_resolved(
                detail,
                existing.source_lang,
                existing.target_lang,
            ));
        }
        return Ok(word_to_resolved(existing));
    }

    let result = app_state
        .translation_service
        .translate(settings, query.clone())
        .await?;
    let resolved_source_lang = result
        .detected_source_lang
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if source_setting == "auto" {
                "en".to_string()
            } else {
                source_setting.clone()
            }
        });

    Ok(translation_result_to_resolved(
        query,
        resolved_source_lang,
        target_lang,
        result,
    ))
}

pub fn get_word_detail_by_id(
    state: &AppState,
    word_id: &str,
) -> Result<Option<WordDetail>, String> {
    let conn = state.db.connection();
    let mut stmt = conn
        .prepare(
            "SELECT
                d.word_id,
                w.word,
                w.translation,
                w.phonetic,
                d.meanings_json,
                d.english_definitions_json,
                d.examples_json,
                d.word_forms_json,
                d.memory_tip,
                d.created_at,
                d.updated_at
             FROM word_details d
             JOIN words w ON w.id = d.word_id
             WHERE d.word_id = ?1",
        )
        .map_err(|error| error.to_string())?;

    stmt.query_row(params![word_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)?,
            row.get::<_, i64>(10)?,
        ))
    })
    .optional()
    .map_err(|error| error.to_string())?
    .map(
        |(
            word_id,
            word,
            translation,
            phonetic,
            meanings_json,
            english_definitions_json,
            examples_json,
            word_forms_json,
            memory_tip,
            created_at,
            updated_at,
        )| {
            Ok(WordDetail {
                word_id,
                word,
                translation,
                phonetic,
                meanings: parse_json_field(&meanings_json, "meanings_json")?,
                english_definitions: parse_json_field(
                    &english_definitions_json,
                    "english_definitions_json",
                )?,
                examples: parse_json_field(&examples_json, "examples_json")?,
                word_forms: parse_json_field(&word_forms_json, "word_forms_json")?,
                memory_tip,
                created_at,
                updated_at,
            })
        },
    )
    .transpose()
}

fn word_detail_to_resolved(
    detail: WordDetail,
    source_lang: String,
    target_lang: String,
) -> ResolvedWord {
    ResolvedWord {
        word_id: Some(detail.word_id),
        word: detail.word,
        translation: detail.translation,
        detected_source_lang: Some(source_lang.clone()),
        source_lang,
        target_lang,
        phonetic: detail.phonetic,
        meanings: detail.meanings,
        english_definitions: detail.english_definitions,
        examples: detail.examples,
        word_forms: detail.word_forms,
        memory_tip: detail.memory_tip,
    }
}

fn word_to_resolved(word: Word) -> ResolvedWord {
    ResolvedWord {
        word_id: Some(word.id),
        word: word.word,
        translation: word.translation,
        detected_source_lang: Some(word.source_lang.clone()),
        source_lang: word.source_lang,
        target_lang: word.target_lang,
        phonetic: word.phonetic,
        meanings: Vec::new(),
        english_definitions: Vec::new(),
        examples: Vec::new(),
        word_forms: Vec::new(),
        memory_tip: String::new(),
    }
}

fn translation_result_to_resolved(
    word: String,
    source_lang: String,
    target_lang: String,
    result: TranslationResult,
) -> ResolvedWord {
    ResolvedWord {
        word_id: None,
        word,
        translation: result.translation,
        detected_source_lang: result.detected_source_lang,
        source_lang,
        target_lang,
        phonetic: result.phonetic,
        meanings: result.meanings,
        english_definitions: result.english_definitions,
        examples: result.examples,
        word_forms: result.word_forms,
        memory_tip: result.memory_tip,
    }
}

fn settings_value(settings: &HashMap<String, String>, key: &str, default: &str) -> String {
    settings
        .get(key)
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn parse_json_field<T>(value: &str, field: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(value).map_err(|error| format!("{field} 解析失败：{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_to_resolved_returns_saved_word_without_details() {
        let mut word = Word::new(
            "word-1".to_string(),
            "panel".to_string(),
            "面板".to_string(),
            "en".to_string(),
            "zh".to_string(),
        );
        word.phonetic = Some("/ˈpænəl/".to_string());

        let resolved = word_to_resolved(word);

        assert_eq!(resolved.word_id, Some("word-1".to_string()));
        assert_eq!(resolved.word, "panel");
        assert_eq!(resolved.translation, "面板");
        assert_eq!(resolved.source_lang, "en");
        assert_eq!(resolved.target_lang, "zh");
        assert_eq!(resolved.phonetic, Some("/ˈpænəl/".to_string()));
        assert!(resolved.meanings.is_empty());
        assert!(resolved.examples.is_empty());
    }
}
