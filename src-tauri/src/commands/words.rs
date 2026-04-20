use crate::db::{Database, Word};
use crate::scheduler::ebbinghaus::SpacedRepetitionState;
use chrono::Utc;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn add_word(
    db: State<'_, Arc<Database>>,
    word: String,
    translation: String,
    source_lang: String,
    target_lang: String,
    tags: String,
) -> Result<Word, String> {
    let now = Utc::now().timestamp();
    let id = Uuid::new_v4().to_string();
    let state = SpacedRepetitionState::default();

    let w = Word {
        id: id.clone(),
        word,
        translation,
        source_lang,
        target_lang,
        status: "learning".to_string(),
        tags,
        notes: String::new(),
        audio_url: String::new(),
        ease_factor: state.ease_factor,
        interval: state.interval,
        repetitions: state.repetitions,
        next_review_at: now + 86400,
        created_at: now,
        updated_at: now,
    };

    let conn = db.conn.lock().await;
    conn.execute(
        "INSERT INTO words (id, word, translation, source_lang, target_lang, status, tags, notes, audio_url, ease_factor, interval, repetitions, next_review_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![w.id, w.word, w.translation, w.source_lang, w.target_lang, w.status, w.tags, w.notes, w.audio_url, w.ease_factor, w.interval, w.repetitions, w.next_review_at, w.created_at, w.updated_at],
    ).map_err(|e| e.to_string())?;

    Ok(w)
}

#[tauri::command]
pub async fn get_words(
    db: State<'_, Arc<Database>>,
    search: Option<String>,
) -> Result<Vec<Word>, String> {
    let conn = db.conn.lock().await;

    let words = if let Some(s) = search {
        let pattern = format!("%{}%", s);
        let mut stmt = conn.prepare(
            "SELECT id, word, translation, source_lang, target_lang, status, tags, notes, audio_url, ease_factor, interval, repetitions, next_review_at, created_at, updated_at FROM words WHERE word LIKE ?1 OR translation LIKE ?1 ORDER BY next_review_at ASC"
        ).map_err(|e| e.to_string())?;
        let result: Vec<Word> = stmt.query_map([&pattern], |row| Ok(Word {
            id: row.get(0)?, word: row.get(1)?, translation: row.get(2)?,
            source_lang: row.get(3)?, target_lang: row.get(4)?,
            status: row.get(5)?, tags: row.get(6)?, notes: row.get(7)?, audio_url: row.get(8)?,
            ease_factor: row.get(9)?, interval: row.get(10)?, repetitions: row.get(11)?,
            next_review_at: row.get(12)?, created_at: row.get(13)?, updated_at: row.get(14)?,
        })).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        result
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, word, translation, source_lang, target_lang, status, tags, notes, audio_url, ease_factor, interval, repetitions, next_review_at, created_at, updated_at FROM words ORDER BY next_review_at ASC"
        ).map_err(|e| e.to_string())?;
        let result: Vec<Word> = stmt.query_map([], |row| Ok(Word {
            id: row.get(0)?, word: row.get(1)?, translation: row.get(2)?,
            source_lang: row.get(3)?, target_lang: row.get(4)?,
            status: row.get(5)?, tags: row.get(6)?, notes: row.get(7)?, audio_url: row.get(8)?,
            ease_factor: row.get(9)?, interval: row.get(10)?, repetitions: row.get(11)?,
            next_review_at: row.get(12)?, created_at: row.get(13)?, updated_at: row.get(14)?,
        })).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        result
    };

    Ok(words)
}

#[tauri::command]
pub async fn delete_word(db: State<'_, Arc<Database>>, word_id: String) -> Result<(), String> {
    let conn = db.conn.lock().await;
    conn.execute("DELETE FROM words WHERE id = ?1", rusqlite::params![word_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
