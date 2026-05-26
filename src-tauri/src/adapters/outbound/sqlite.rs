use crate::db::{Database, DbError};
use crate::domain::models::Word;
use crate::ports::outbound::repository::WordRepository;
use rusqlite::params;
use std::sync::Arc;

pub struct SqliteWordRepository {
    db: Arc<Database>,
}

impl SqliteWordRepository {
    pub fn new(db: Arc<Database>) -> Self {
        SqliteWordRepository { db }
    }

    fn row_to_word(row: &rusqlite::Row) -> Result<Word, rusqlite::Error> {
        Ok(Word {
            id: row.get("id")?,
            word: row.get("word")?,
            translation: row.get("translation")?,
            phonetic: row.get("phonetic").ok(),
            source_lang: row.get("source_lang").unwrap_or_else(|_| "EN".to_string()),
            target_lang: row.get("target_lang").unwrap_or_else(|_| "ZH".to_string()),
            status: row.get("status").unwrap_or_else(|_| "new".to_string()),
            tags: row.get("tags").unwrap_or_else(|_| String::new()),
            notes: row.get("notes").unwrap_or_else(|_| String::new()),
            audio_url: row.get("audio_url").ok(),
            ease_factor: row.get("ease_factor").unwrap_or(2.5),
            interval: row.get("interval").unwrap_or(0),
            repetitions: row.get("repetitions").unwrap_or(0),
            next_review_at: row.get("next_review_at").ok(),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl WordRepository for SqliteWordRepository {
    fn create(&self, word: Word) -> Result<Word, DbError> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO words (id, word, translation, phonetic, source_lang, target_lang, status, tags, notes, audio_url, ease_factor, interval, repetitions, next_review_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                word.id,
                word.word,
                word.translation,
                word.phonetic,
                word.source_lang,
                word.target_lang,
                word.status,
                word.tags,
                word.notes,
                word.audio_url,
                word.ease_factor,
                word.interval,
                word.repetitions,
                word.next_review_at,
                word.created_at,
                word.updated_at,
            ],
        ).map_err(DbError::Sqlite)?;
        Ok(word)
    }

    fn find_all(&self, search: Option<&str>) -> Result<Vec<Word>, DbError> {
        let conn = self.db.connection();
        let mut stmt = match search {
            Some(s) if !s.is_empty() => conn.prepare(
                "SELECT * FROM words WHERE word LIKE ?1 OR translation LIKE ?1 ORDER BY created_at DESC"
            ).map_err(DbError::Sqlite)?,
            _ => conn
                .prepare("SELECT * FROM words ORDER BY created_at DESC")
                .map_err(DbError::Sqlite)?,
        };

        let word_iter = match search {
            Some(s) if !s.is_empty() => {
                let pattern = format!("%{}%", s);
                stmt.query(params![pattern]).map_err(DbError::Sqlite)?
            }
            _ => stmt.query([]).map_err(DbError::Sqlite)?,
        };

        let words = word_iter
            .mapped(Self::row_to_word)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::Sqlite)?;

        Ok(words)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Word>, DbError> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT * FROM words WHERE id = ?1")
            .map_err(DbError::Sqlite)?;

        let mut rows = stmt.query(params![id]).map_err(DbError::Sqlite)?;

        match rows.next().map_err(DbError::Sqlite)? {
            Some(row) => Ok(Some(Self::row_to_word(row).map_err(DbError::Sqlite)?)),
            None => Ok(None),
        }
    }

    fn update(&self, word: &Word) -> Result<Word, DbError> {
        let conn = self.db.connection();
        conn.execute(
            "UPDATE words SET word=?2, translation=?3, phonetic=?4, source_lang=?5, target_lang=?6, status=?7, tags=?8, notes=?9, audio_url=?10, ease_factor=?11, interval=?12, repetitions=?13, next_review_at=?14, updated_at=?15 WHERE id=?1",
            params![
                word.id,
                word.word,
                word.translation,
                word.phonetic,
                word.source_lang,
                word.target_lang,
                word.status,
                word.tags,
                word.notes,
                word.audio_url,
                word.ease_factor,
                word.interval,
                word.repetitions,
                word.next_review_at,
                word.updated_at,
            ],
        ).map_err(DbError::Sqlite)?;
        Ok(word.clone())
    }

    fn delete(&self, id: &str) -> Result<(), DbError> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM word_details WHERE word_id = ?1", params![id])
            .map_err(DbError::Sqlite)?;
        conn.execute("DELETE FROM review_records WHERE word_id = ?1", params![id])
            .map_err(DbError::Sqlite)?;
        conn.execute("DELETE FROM words WHERE id = ?1", params![id])
            .map_err(DbError::Sqlite)?;
        Ok(())
    }
}
