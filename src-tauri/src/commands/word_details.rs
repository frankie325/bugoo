use crate::commands::translate::{create_word_insight_provider, load_translation_config};
use crate::commands::AppState;
use crate::ports::outbound::translation::{TranslationError, TranslationExample};
use crate::ports::outbound::word_insight::{GeneratedWordDetail, WordInsightRequest};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct WordDetail {
    pub word_id: String,
    pub word: String,
    pub translation: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<TranslationExample>,
    pub memory_tip: String,
    pub detail: String,
    pub provider: String,
    pub raw_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WordDetailInput {
    pub word_id: String,
    pub translation: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<TranslationExample>,
    pub memory_tip: String,
    pub detail: String,
    pub provider: String,
    pub raw_json: String,
}

#[derive(Debug, Clone)]
struct WordSummary {
    id: String,
    word: String,
    translation: String,
    source_lang: String,
    target_lang: String,
}

#[tauri::command]
pub fn get_word_detail(
    state: tauri::State<AppState>,
    word_id: String,
) -> Result<Option<WordDetail>, String> {
    get_word_detail_by_id(state.inner(), &word_id)
}

#[tauri::command]
pub async fn generate_word_detail(
    state: tauri::State<'_, AppState>,
    word_id: String,
) -> Result<WordDetail, String> {
    let app_state = state.inner();
    let word = read_word_summary(app_state, &word_id)?
        .ok_or_else(|| TranslationError::WordNotFound.to_string())?;
    let config = load_translation_config(app_state)?;
    let engine = config.engine.trim().to_lowercase();
    let provider = create_word_insight_provider(config)?;
    let request = WordInsightRequest {
        word: word.word.clone(),
        translation: word.translation.clone(),
        source_lang: word.source_lang,
        target_lang: word.target_lang,
    };

    let generated = provider
        .generate_word_detail(request)
        .await
        .map_err(|error| error.to_string())?;
    let raw_json = serde_json::to_string(&generated).map_err(|error| error.to_string())?;
    let input = generated_word_detail_to_input(word.id, engine, raw_json, generated);

    save_word_detail_for_state(app_state, input)
}

#[tauri::command]
pub fn save_word_detail(
    state: tauri::State<AppState>,
    detail: WordDetailInput,
) -> Result<WordDetail, String> {
    save_word_detail_for_state(state.inner(), detail)
}

fn get_word_detail_by_id(state: &AppState, word_id: &str) -> Result<Option<WordDetail>, String> {
    let conn = state.db.connection();
    let mut stmt = conn
        .prepare(
            "SELECT
                d.word_id,
                w.word,
                w.translation,
                w.phonetic,
                d.part_of_speech_json,
                d.definitions_json,
                d.examples_json,
                d.memory_tip,
                d.detail,
                d.provider,
                d.raw_json,
                d.created_at,
                d.updated_at
             FROM word_details d
             JOIN words w ON w.id = d.word_id
             WHERE d.word_id = ?1",
        )
        .map_err(|error| error.to_string())?;

    stmt.query_row(params![word_id], |row| {
        let part_of_speech_json: String = row.get(4)?;
        let definitions_json: String = row.get(5)?;
        let examples_json: String = row.get(6)?;

        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            part_of_speech_json,
            definitions_json,
            examples_json,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, i64>(12)?,
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
            part_of_speech_json,
            definitions_json,
            examples_json,
            memory_tip,
            detail,
            provider,
            raw_json,
            created_at,
            updated_at,
        )| {
            Ok(WordDetail {
                word_id,
                word,
                translation,
                phonetic,
                part_of_speech: parse_json_field(&part_of_speech_json, "part_of_speech_json")?,
                definitions: parse_json_field(&definitions_json, "definitions_json")?,
                examples: parse_json_field(&examples_json, "examples_json")?,
                memory_tip,
                detail,
                provider,
                raw_json,
                created_at,
                updated_at,
            })
        },
    )
    .transpose()
}

fn save_word_detail_for_state(
    state: &AppState,
    detail: WordDetailInput,
) -> Result<WordDetail, String> {
    let word = read_word_summary(state, &detail.word_id)?
        .ok_or_else(|| TranslationError::WordNotFound.to_string())?;
    let part_of_speech_json =
        serde_json::to_string(&detail.part_of_speech).map_err(|error| error.to_string())?;
    let definitions_json =
        serde_json::to_string(&detail.definitions).map_err(|error| error.to_string())?;
    let examples_json =
        serde_json::to_string(&detail.examples).map_err(|error| error.to_string())?;
    let raw_json = normalize_raw_json(&detail)?;
    let now = Utc::now().timestamp();

    {
        let conn = state.db.connection();
        let created_at = conn
            .query_row(
                "SELECT created_at FROM word_details WHERE word_id = ?1",
                params![detail.word_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .unwrap_or(now);

        conn.execute(
            "UPDATE words SET translation = ?2, phonetic = ?3, updated_at = ?4 WHERE id = ?1",
            params![word.id, detail.translation, detail.phonetic, now],
        )
        .map_err(|error| error.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO word_details
                (word_id, part_of_speech_json, definitions_json, examples_json, memory_tip,
                 detail, provider, raw_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                detail.word_id,
                part_of_speech_json,
                definitions_json,
                examples_json,
                detail.memory_tip,
                detail.detail,
                detail.provider,
                raw_json,
                created_at,
                now,
            ],
        )
        .map_err(|error| error.to_string())?;
    }

    get_word_detail_by_id(state, &detail.word_id)?
        .ok_or_else(|| TranslationError::WordNotFound.to_string())
}

fn read_word_summary(state: &AppState, word_id: &str) -> Result<Option<WordSummary>, String> {
    let conn = state.db.connection();
    conn.query_row(
        "SELECT id, word, translation, source_lang, target_lang FROM words WHERE id = ?1",
        params![word_id],
        |row| {
            Ok(WordSummary {
                id: row.get("id")?,
                word: row.get("word")?,
                translation: row.get("translation")?,
                source_lang: row.get("source_lang").unwrap_or_else(|_| "EN".to_string()),
                target_lang: row.get("target_lang").unwrap_or_else(|_| "ZH".to_string()),
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn generated_word_detail_to_input(
    word_id: String,
    provider: String,
    raw_json: String,
    generated: GeneratedWordDetail,
) -> WordDetailInput {
    WordDetailInput {
        word_id,
        translation: generated.translation,
        phonetic: generated.phonetic,
        part_of_speech: generated.part_of_speech,
        definitions: generated.definitions,
        examples: generated.examples,
        memory_tip: generated.memory_tip,
        detail: generated.detail,
        provider,
        raw_json,
    }
}

fn normalize_raw_json(detail: &WordDetailInput) -> Result<String, String> {
    let raw_json = detail.raw_json.trim();
    if raw_json.is_empty() {
        return serde_json::to_string(detail).map_err(|error| error.to_string());
    }

    serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|error| format!("raw_json 不是合法 JSON：{error}"))?;
    Ok(raw_json.to_string())
}

fn parse_json_field<T>(value: &str, field: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(value).map_err(|error| format!("{field} 解析失败：{error}"))
}
