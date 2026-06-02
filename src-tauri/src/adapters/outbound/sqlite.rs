use crate::db::{Database, DbError};
use crate::domain::models::Word;
use crate::ports::outbound::repository::{WordDetailDraft, WordRepository};
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

    fn find_by_text(
        &self,
        word: &str,
        source_lang: Option<&str>,
        target_lang: &str,
    ) -> Result<Option<Word>, DbError> {
        let conn = self.db.connection();
        let normalized = word.trim().to_lowercase();
        let normalized_target_lang = target_lang.trim().to_lowercase();
        let normalized_source_lang = source_lang.map(|lang| lang.trim().to_lowercase());
        if normalized.is_empty() {
            return Ok(None);
        }
        let mut stmt = conn
            .prepare(
                "SELECT * FROM words
	                 WHERE LOWER(word) = ?1
	                   AND LOWER(target_lang) = ?2
	                   AND (?3 IS NULL OR LOWER(source_lang) = ?3)
	                 ORDER BY created_at DESC
	                 LIMIT 1",
            )
            .map_err(DbError::Sqlite)?;

        let mut rows = stmt
            .query(params![
                normalized,
                normalized_target_lang,
                normalized_source_lang.as_deref()
            ])
            .map_err(DbError::Sqlite)?;

        match rows.next().map_err(DbError::Sqlite)? {
            Some(row) => Ok(Some(Self::row_to_word(row).map_err(DbError::Sqlite)?)),
            None => Ok(None),
        }
    }

    fn save_with_details(&self, word: &Word, detail: &WordDetailDraft) -> Result<Word, DbError> {
        let meanings_json = serde_json::to_string(&detail.meanings)?;
        let english_definitions_json = serde_json::to_string(&detail.english_definitions)?;
        let examples_json = serde_json::to_string(&detail.examples)?;
        let word_forms_json = serde_json::to_string(&detail.word_forms)?;

        let mut conn = self.db.connection();
        let tx = conn.transaction().map_err(DbError::Sqlite)?;
        let existing_count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM words WHERE id = ?1",
                params![&word.id],
                |row| row.get(0),
            )
            .map_err(DbError::Sqlite)?;

        if existing_count > 0 {
            tx.execute(
                "UPDATE words
                 SET word = ?2,
                     translation = ?3,
                     phonetic = ?4,
                     source_lang = ?5,
                     target_lang = ?6,
                     status = ?7,
                     tags = ?8,
                     notes = ?9,
                     audio_url = ?10,
                     ease_factor = ?11,
                     interval = ?12,
                     repetitions = ?13,
                     next_review_at = ?14,
                     updated_at = ?15
                 WHERE id = ?1",
                params![
                    &word.id,
                    &word.word,
                    &word.translation,
                    &word.phonetic,
                    &word.source_lang,
                    &word.target_lang,
                    &word.status,
                    &word.tags,
                    &word.notes,
                    &word.audio_url,
                    word.ease_factor,
                    word.interval,
                    word.repetitions,
                    &word.next_review_at,
                    word.updated_at,
                ],
            )
            .map_err(DbError::Sqlite)?;
        } else {
            tx.execute(
                "INSERT INTO words
                    (id, word, translation, phonetic, source_lang, target_lang,
                     status, tags, notes, audio_url, ease_factor, interval, repetitions,
                     next_review_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    &word.id,
                    &word.word,
                    &word.translation,
                    &word.phonetic,
                    &word.source_lang,
                    &word.target_lang,
                    &word.status,
                    &word.tags,
                    &word.notes,
                    &word.audio_url,
                    word.ease_factor,
                    word.interval,
                    word.repetitions,
                    &word.next_review_at,
                    word.created_at,
                    word.updated_at,
                ],
            )
            .map_err(DbError::Sqlite)?;
        }

        tx.execute(
            "INSERT INTO word_details
                (word_id, meanings_json, english_definitions_json, examples_json,
                 word_forms_json, memory_tip, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(word_id) DO UPDATE SET
                meanings_json = excluded.meanings_json,
                english_definitions_json = excluded.english_definitions_json,
                examples_json = excluded.examples_json,
                word_forms_json = excluded.word_forms_json,
                memory_tip = excluded.memory_tip,
                updated_at = excluded.updated_at",
            params![
                &word.id,
                &meanings_json,
                &english_definitions_json,
                &examples_json,
                &word_forms_json,
                &detail.memory_tip,
                word.created_at,
                word.updated_at,
            ],
        )
        .map_err(DbError::Sqlite)?;

        tx.commit().map_err(DbError::Sqlite)?;
        Ok(word.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::outbound::translation::TranslationExample;
    use std::sync::Arc;

    fn test_db() -> Arc<Database> {
        let path =
            std::env::temp_dir().join(format!("bugoo-sqlite-repo-{}.db", uuid::Uuid::new_v4()));
        Arc::new(Database::new(path).unwrap())
    }

    fn detail_draft() -> WordDetailDraft {
        WordDetailDraft {
            meanings: vec![crate::domain::models::WordMeaning {
                part_of_speech: "int".to_string(),
                translations: vec!["你好".to_string()],
            }],
            english_definitions: Vec::new(),
            examples: vec![TranslationExample {
                sentence: "Hello there.".to_string(),
                translation: "你好。".to_string(),
            }],
            word_forms: Vec::new(),
            memory_tip: "问候语".to_string(),
        }
    }

    #[test]
    fn find_by_text_matches_language_case_insensitively() {
        let db = test_db();
        let repo = SqliteWordRepository::new(Arc::clone(&db));
        let word = Word::new(
            "word-1".to_string(),
            "Hello".to_string(),
            "你好".to_string(),
            "EN".to_string(),
            "ZH".to_string(),
        );

        repo.save_with_details(&word, &detail_draft()).unwrap();

        let found = repo
            .find_by_text("hello", Some("en"), "zh")
            .unwrap()
            .expect("word should be found");
        assert_eq!(found.id, "word-1");
    }

    #[test]
    fn save_with_details_upserts_detail_for_existing_word_without_duplicate() {
        let db = test_db();
        {
            let conn = db.connection();
            conn.execute(
                "INSERT INTO words
                    (id, word, translation, source_lang, target_lang, created_at, updated_at)
                 VALUES ('word-1', 'hello', '你好', 'EN', 'ZH', 1, 1)",
                [],
            )
            .unwrap();
        }

        let repo = SqliteWordRepository::new(Arc::clone(&db));
        let mut word = repo
            .find_by_text("hello", Some("en"), "zh")
            .unwrap()
            .expect("existing word should be found");
        word.translation = "您好".to_string();
        word.source_lang = "en".to_string();
        word.target_lang = "zh".to_string();
        word.updated_at = 2;

        repo.save_with_details(&word, &detail_draft()).unwrap();

        let conn = db.connection();
        let word_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM words WHERE LOWER(word) = 'hello'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let detail_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM word_details WHERE word_id = 'word-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(word_count, 1);
        assert_eq!(detail_count, 1);
    }
}
