use crate::adapters::outbound::translation::{
    custom::CustomTranslationProvider, deepl::DeepLTranslationProvider,
    google::GoogleTranslationProvider,
};
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationProvider, TranslationRequest, TranslationResult,
};
use crate::ports::outbound::word_insight::{GeneratedWordDetail, WordInsightProvider, WordInsightRequest};
use std::collections::HashMap;

pub struct TranslationService;

impl TranslationService {
    pub async fn translate(
        settings: HashMap<String, String>,
        text: String,
        source_lang: String,
        target_lang: String,
    ) -> Result<TranslationResult, String> {
        validate_text(&text).map_err(|e| e.to_string())?;
        let config = build_translation_config(&settings);
        let provider = create_translation_provider(config)?;
        let request = TranslationRequest {
            text,
            source_lang,
            target_lang,
        };
        provider.translate(request).await.map_err(|e| e.to_string())
    }

    pub async fn generate_word_detail(
        settings: HashMap<String, String>,
        word: String,
        translation: String,
        source_lang: String,
        target_lang: String,
    ) -> Result<GeneratedWordDetail, String> {
        validate_text(&word).map_err(|e| e.to_string())?;
        let config = build_translation_config(&settings);
        let provider = create_word_insight_provider(config)?;
        let request = WordInsightRequest {
            word,
            translation,
            source_lang,
            target_lang,
        };
        provider
            .generate_word_detail(request)
            .await
            .map_err(|e| e.to_string())
    }
}

pub fn validate_text(text: &str) -> Result<(), TranslationError> {
    if text.trim().is_empty() {
        return Err(TranslationError::EmptyText);
    }
    Ok(())
}

pub fn normalize_endpoint(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

fn build_translation_config(settings: &HashMap<String, String>) -> TranslationConfig {
    TranslationConfig {
        engine: setting_or_default(settings, "translationEngine", "custom"),
        api_endpoint: setting_or_default(settings, "apiEndpoint", ""),
        api_key: setting_or_default(settings, "apiKey", ""),
        translation_model: setting_or_default(settings, "translationModel", ""),
        translation_prompt: setting_or_default(settings, "translationPrompt", ""),
        word_detail_prompt: setting_or_default(settings, "wordDetailPrompt", ""),
        timeout_ms: setting_or_default(settings, "translationTimeoutMs", "15000")
            .parse::<u64>()
            .unwrap_or(15_000),
    }
}

fn create_translation_provider(
    config: TranslationConfig,
) -> Result<Box<dyn TranslationProvider>, String> {
    match config.engine.trim().to_lowercase().as_str() {
        "openai" | "custom" => CustomTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "deepl" => Ok(Box::new(DeepLTranslationProvider)),
        "google" => Ok(Box::new(GoogleTranslationProvider)),
        engine => Err(TranslationError::UnsupportedEngine(engine.to_string()).to_string()),
    }
}

fn create_word_insight_provider(
    config: TranslationConfig,
) -> Result<Box<dyn WordInsightProvider>, String> {
    match config.engine.trim().to_lowercase().as_str() {
        "openai" | "custom" => CustomTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn WordInsightProvider>)
            .map_err(|error| error.to_string()),
        "deepl" | "google" => Err(TranslationError::UnsupportedEngine(config.engine).to_string()),
        engine => Err(TranslationError::UnsupportedEngine(engine.to_string()).to_string()),
    }
}

fn setting_or_default(settings: &HashMap<String, String>, key: &str, default: &str) -> String {
    settings
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_text_rejects_empty_text() {
        assert_eq!(validate_text("  "), Err(TranslationError::EmptyText));
    }

    #[test]
    fn validate_text_accepts_non_empty_text() {
        assert!(validate_text("hello").is_ok());
    }

    #[test]
    fn normalize_endpoint_trims_trailing_slashes() {
        assert_eq!(
            normalize_endpoint(" https://api.example.com/v1/// "),
            "https://api.example.com/v1"
        );
    }

    #[test]
    fn setting_or_default_returns_value_when_present() {
        let settings = HashMap::from([("translationEngine".to_string(), "deepl".to_string())]);
        assert_eq!(
            setting_or_default(&settings, "translationEngine", "custom"),
            "deepl"
        );
    }

    #[test]
    fn setting_or_default_returns_default_when_missing() {
        let settings = HashMap::new();
        assert_eq!(
            setting_or_default(&settings, "translationEngine", "custom"),
            "custom"
        );
    }

    #[test]
    fn setting_or_default_returns_default_when_empty() {
        let settings = HashMap::from([("translationEngine".to_string(), "  ".to_string())]);
        assert_eq!(
            setting_or_default(&settings, "translationEngine", "custom"),
            "custom"
        );
    }
}