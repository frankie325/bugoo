use crate::domain::models::Word;
use crate::db::DbError;

pub trait WordRepository: Send + Sync {
    fn create(&self, word: Word) -> Result<Word, DbError>;
    fn find_all(&self, search: Option<&str>) -> Result<Vec<Word>, DbError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Word>, DbError>;
    fn update(&self, word: &Word) -> Result<Word, DbError>;
    fn delete(&self, id: &str) -> Result<(), DbError>;
}
