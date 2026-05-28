#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedLanguage {
    Known(String),
    Unknown,
}

pub trait LanguageDetector: Send + Sync {
    fn detect(&self, text: &str) -> DetectedLanguage;
}
