use crate::adapters::outbound::sqlite::SqliteWordRepository;
use crate::db::Database;
use crate::domain::models::{EnglishDefinitionGroup, Word, WordFormItem, WordMeaning};
use crate::ports::outbound::repository::{WordDetailDraft, WordRepository};
use crate::ports::outbound::translation::TranslationExample;
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

    pub fn find_existing_word(
        &self,
        word: &str,
        target_lang: &str,
    ) -> Result<Option<Word>, String> {
        self.repository
            .find_by_text(word, target_lang)
            .map_err(|e| e.to_string())
    }

    pub fn get_words(&self, search: Option<String>) -> Result<Vec<Word>, String> {
        self.repository
            .find_all(search.as_deref())
            .map_err(|e| e.to_string())
    }

    pub fn add_word_with_details(&self, input: AddWordWithDetails) -> Result<Word, String> {
        let word_text = input.word.trim().to_string();
        if word_text.is_empty() {
            return Err("单词不能为空".to_string());
        }

        let source_lang = normalize_lang_or_default(&input.source_lang, "en");
        let target_lang = normalize_lang_or_default(&input.target_lang, "zh");
        let now = chrono::Utc::now().timestamp_millis();
        let existing = self.find_existing_word(&word_text, &target_lang)?;

        let mut word = existing.unwrap_or_else(|| {
            Word::new(
                uuid::Uuid::new_v4().to_string(),
                word_text.clone(),
                input.translation.clone(),
                source_lang.clone(),
                target_lang.clone(),
            )
        });

        word.word = word_text;
        word.translation = input.translation;
        word.phonetic = input.phonetic.or(word.phonetic);
        word.source_lang = source_lang;
        word.target_lang = target_lang;
        if !input.tags.trim().is_empty() {
            word.tags = input.tags;
        }
        if word.created_at == 0 {
            word.created_at = now;
        }
        word.updated_at = now;

        let detail = WordDetailDraft {
            meanings: input.meanings,
            english_definitions: input.english_definitions,
            examples: input.examples,
            word_forms: input.word_forms,
            memory_tip: input.memory_tip,
        };

        self.repository
            .save_with_details(&word, &detail)
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

#[derive(Debug)]
pub struct AddWordWithDetails {
    pub word: String,
    pub translation: String,
    pub source_lang: String,
    pub target_lang: String,
    pub phonetic: Option<String>,
    pub meanings: Vec<WordMeaning>,
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    pub examples: Vec<TranslationExample>,
    pub word_forms: Vec<WordFormItem>,
    pub memory_tip: String,
    pub tags: String,
}

fn normalize_lang_or_default(value: &str, default: &str) -> String {
    let normalized = value.trim().to_lowercase();
    if normalized.is_empty() {
        default.to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service() -> WordService {
        let path =
            std::env::temp_dir().join(format!("bugoo-word-service-{}.db", uuid::Uuid::new_v4()));
        WordService::new(Arc::new(Database::new(path).unwrap()))
    }

    fn add_input(word: &str, source_lang: &str, target_lang: &str) -> AddWordWithDetails {
        AddWordWithDetails {
            word: word.to_string(),
            translation: "面板".to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            phonetic: None,
            meanings: Vec::new(),
            english_definitions: Vec::new(),
            examples: Vec::new(),
            word_forms: Vec::new(),
            memory_tip: String::new(),
            tags: String::new(),
        }
    }

    #[test]
    fn add_word_with_details_reuses_existing_word_when_source_lang_differs() {
        let service = test_service();
        let first = service
            .add_word_with_details(add_input("panel", "en", "zh"))
            .unwrap();
        let second = service
            .add_word_with_details(add_input("panel", "fr", "zh"))
            .unwrap();

        assert_eq!(second.id, first.id);
        assert_eq!(service.get_words(None).unwrap().len(), 1);
    }
}
