use crate::adapters::outbound::sqlite::SqliteWordRepository;
use crate::db::Database;
use crate::domain::models::Word;
use crate::ports::outbound::repository::WordRepository;
use std::sync::Arc;

pub struct WordService {
    repository: Arc<SqliteWordRepository>,
}

impl WordService {
    pub fn new(db: Arc<Database>) -> Self {
        WordService {
            repository: Arc::new(SqliteWordRepository::new(db)),
        }
    }

    pub fn add_word(
        &self,
        word: String,
        translation: String,
        source_lang: String,
        target_lang: String,
        tags: String,
    ) -> Result<Word, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut w = Word::new(id, word, translation, source_lang, target_lang);
        w.tags = tags;
        self.repository.create(w).map_err(|e| e.to_string())
    }

    pub fn get_words(&self, search: Option<String>) -> Result<Vec<Word>, String> {
        self.repository
            .find_all(search.as_deref())
            .map_err(|e| e.to_string())
    }

    pub fn delete_word(&self, word_id: &str) -> Result<(), String> {
        self.repository.delete(word_id).map_err(|e| e.to_string())
    }

    pub fn update_word(&self, word_id: &str, updates: WordUpdate) -> Result<Word, String> {
        let mut word = self
            .repository
            .find_by_id(word_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Word not found".to_string())?;

        if let Some(translation) = updates.translation {
            word.translation = translation;
        }
        if let Some(tags) = updates.tags {
            word.tags = tags;
        }
        if let Some(notes) = updates.notes {
            word.notes = notes;
        }
        if let Some(status) = updates.status {
            word.status = status;
        }

        word.updated_at = chrono::Utc::now().timestamp_millis();
        self.repository.update(&word).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct WordUpdate {
    pub translation: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
}
