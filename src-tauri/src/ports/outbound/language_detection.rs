use std::future::Future;
use std::pin::Pin;

pub type LanguageDetectionFuture<'a> = Pin<Box<dyn Future<Output = DetectedLanguage> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedLanguage {
    Known(String),
    Unknown,
}

pub trait LanguageDetector: Send + Sync {
    fn detect<'a>(&'a self, text: &'a str) -> LanguageDetectionFuture<'a>;
}
