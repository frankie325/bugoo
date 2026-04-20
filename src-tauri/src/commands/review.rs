use crate::db::{Database, Word};
use crate::scheduler::ebbinghaus::SpacedRepetitionState;
use chrono::Utc;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn submit_review(
    db: State<'_, Arc<Database>>,
    word_id: String,
    remembered: bool,
) -> Result<(), String> {
    let conn = db.conn.lock().await;

    let mut stmt = conn.prepare(
        "SELECT ease_factor, interval, repetitions FROM words WHERE id = ?1"
    ).map_err(|e| e.to_string())?;
    let (ease_factor, interval, repetitions): (f64, i32, i32) = stmt
        .query_row([&word_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|e| e.to_string())?;
    drop(stmt);

    let state = SpacedRepetitionState { ease_factor, interval, repetitions, next_review_at: 0 };
    let next = SpacedRepetitionState::after_review(&state, remembered);

    let now = Utc::now().timestamp();
    let new_status = if next.interval > 21 { "mastered" } else { "learning" };

    conn.execute(
        "UPDATE words SET ease_factor = ?1, interval = ?2, repetitions = ?3, next_review_at = ?4, status = ?5, updated_at = ?6 WHERE id = ?7",
        rusqlite::params![next.ease_factor, next.interval, next.repetitions, next.next_review_at, new_status, now, word_id],
    ).map_err(|e| e.to_string())?;

    let log_id = Uuid::new_v4().to_string();
    let result = if remembered { "remembered" } else { "forgotten" };
    conn.execute(
        "INSERT INTO review_log (id, word_id, result, reviewed_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![log_id, word_id, result, now],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_due_reviews(db: State<'_, Arc<Database>>) -> Result<Vec<Word>, String> {
    let now = chrono::Utc::now().timestamp();
    let conn = db.conn.lock().await;
    let mut stmt = conn.prepare(
        "SELECT id, word, translation, source_lang, target_lang, status, tags, notes, audio_url, ease_factor, interval, repetitions, next_review_at, created_at, updated_at FROM words WHERE status = 'learning' AND next_review_at <= ?1 ORDER BY next_review_at ASC LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let words = stmt.query_map([now], |row| Ok(Word {
        id: row.get(0)?, word: row.get(1)?, translation: row.get(2)?,
        source_lang: row.get(3)?, target_lang: row.get(4)?,
        status: row.get(5)?, tags: row.get(6)?, notes: row.get(7)?, audio_url: row.get(8)?,
        ease_factor: row.get(9)?, interval: row.get(10)?, repetitions: row.get(11)?,
        next_review_at: row.get(12)?, created_at: row.get(13)?, updated_at: row.get(14)?,
    })).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(words)
}
