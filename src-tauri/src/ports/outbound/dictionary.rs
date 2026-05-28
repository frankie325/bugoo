use crate::ports::outbound::translation::TranslationExample;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryLookupRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryLookupResult {
    pub word: String,
    pub translation: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<TranslationExample>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DictionaryError {
    #[error("词典查询文本不能为空")]
    EmptyText,
    #[error("词典资源不存在：{0}")]
    ResourceMissing(String),
    #[error("词典查询失败：{0}")]
    QueryFailed(String),
}

pub trait DictionaryProvider: Send + Sync {
    fn lookup(
        &self,
        request: DictionaryLookupRequest,
    ) -> Result<Option<DictionaryLookupResult>, DictionaryError>;

    fn supports_language_pair(&self, source_lang: &str, target_lang: &str) -> bool;
}

pub fn normalize_dictionary_text(text: &str) -> String {
    text.trim().to_lowercase()
}

pub fn should_lookup_dictionary(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.chars().count() > 48 {
        return false;
    }

    if trimmed.ends_with('.')
        || trimmed.ends_with('?')
        || trimmed.ends_with('!')
        || trimmed.ends_with('。')
        || trimmed.ends_with('？')
        || trimmed.ends_with('！')
    {
        return false;
    }

    trimmed.split_whitespace().count() <= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_dictionary_text_trims_and_lowercases() {
        assert_eq!(normalize_dictionary_text(" Hello "), "hello");
    }

    #[test]
    fn should_lookup_dictionary_accepts_word_and_short_phrase() {
        assert!(should_lookup_dictionary("hello"));
        assert!(should_lookup_dictionary("look up"));
    }

    #[test]
    fn should_lookup_dictionary_rejects_sentence_like_text() {
        assert!(!should_lookup_dictionary("I like this app."));
        assert!(!should_lookup_dictionary("one two three four five"));
        assert!(!should_lookup_dictionary(
            "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"
        ));
    }
}
