use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};
use whichlang::{detect_language, Lang};

#[derive(Clone)]
pub struct WhichlangLanguageDetector;

impl WhichlangLanguageDetector {
    pub fn new() -> Self {
        Self
    }
}

impl LanguageDetector for WhichlangLanguageDetector {
    fn detect(&self, text: &str) -> DetectedLanguage {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return DetectedLanguage::Unknown;
        }

        lang_to_libretranslate_code(detect_language(trimmed))
            .map(|code| DetectedLanguage::Known(code.to_string()))
            .unwrap_or(DetectedLanguage::Unknown)
    }
}

impl Default for WhichlangLanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

fn lang_to_libretranslate_code(lang: Lang) -> Option<&'static str> {
    match lang {
        Lang::Ara => Some("ar"),
        Lang::Cmn => Some("zh"),
        Lang::Deu => Some("de"),
        Lang::Eng => Some("en"),
        Lang::Fra => Some("fr"),
        Lang::Hin => Some("hi"),
        Lang::Ita => Some("it"),
        Lang::Jpn => Some("ja"),
        Lang::Kor => Some("ko"),
        Lang::Nld => Some("nl"),
        Lang::Por => Some("pt"),
        Lang::Rus => Some("ru"),
        Lang::Spa => Some("es"),
        Lang::Swe => Some("sv"),
        Lang::Tur => Some("tr"),
        Lang::Vie => Some("vi"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_language_code() {
        let detector = WhichlangLanguageDetector::new();

        assert_eq!(
            detector.detect("Hello world"),
            DetectedLanguage::Known("en".to_string())
        );
        assert_eq!(
            detector.detect("你好世界"),
            DetectedLanguage::Known("zh".to_string())
        );
        assert_eq!(
            detector.detect("こんにちは世界"),
            DetectedLanguage::Known("ja".to_string())
        );
        assert_eq!(
            detector.detect("안녕하세요 세계"),
            DetectedLanguage::Known("ko".to_string())
        );
    }

    #[test]
    fn detect_french() {
        let detector = WhichlangLanguageDetector::new();
        assert_eq!(
            detector.detect("Bonjour le monde"),
            DetectedLanguage::Known("fr".to_string())
        );
    }

    #[test]
    fn detect_german() {
        let detector = WhichlangLanguageDetector::new();
        assert_eq!(
            detector.detect("Guten Tag Welt"),
            DetectedLanguage::Known("de".to_string())
        );
    }

    #[test]
    fn detect_returns_unknown_for_empty_text() {
        let detector = WhichlangLanguageDetector::new();

        assert_eq!(detector.detect("  "), DetectedLanguage::Unknown);
    }
}
