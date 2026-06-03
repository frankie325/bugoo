use crate::db::DbError;
use crate::domain::models::{EnglishDefinitionGroup, Word, WordFormItem, WordMeaning};
use crate::ports::outbound::translation::TranslationExample;

#[derive(Debug, Clone)]
pub struct WordDetailDraft {
    pub meanings: Vec<WordMeaning>,
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    pub examples: Vec<TranslationExample>,
    pub word_forms: Vec<WordFormItem>,
    pub memory_tip: String,
}

pub trait WordRepository: Send + Sync {
    fn find_all(&self, search: Option<&str>) -> Result<Vec<Word>, DbError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Word>, DbError>;
    fn find_by_text(&self, word: &str, target_lang: &str) -> Result<Option<Word>, DbError>;
    fn save_with_details(&self, word: &Word, detail: &WordDetailDraft) -> Result<Word, DbError>;
    fn update(&self, word: &Word) -> Result<Word, DbError>;
    fn delete(&self, id: &str) -> Result<(), DbError>;
}
