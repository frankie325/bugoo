use crate::ports::outbound::translation::{TranslationError, TranslationExample};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub type WordInsightFuture<'a> =
    Pin<Box<dyn Future<Output = Result<GeneratedWordDetail, TranslationError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct WordInsightRequest {
    pub word: String,
    pub translation: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedWordDetail {
    pub translation: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<TranslationExample>,
    pub memory_tip: String,
    pub detail: String,
}

pub trait WordInsightProvider: Send + Sync {
    fn generate_word_detail<'a>(&'a self, request: WordInsightRequest) -> WordInsightFuture<'a>;
}
