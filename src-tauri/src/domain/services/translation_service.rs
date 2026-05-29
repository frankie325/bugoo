use crate::adapters::outbound::translation::{
    baidu::BaiduTranslationProvider, custom::CustomTranslationProvider,
    deepl::DeepLTranslationProvider, google::GoogleTranslationProvider,
    libretranslate::LibreTranslateProvider, microsoft::MicrosoftTranslationProvider,
    tencent::TencentTranslationProvider, youdao::YoudaoTranslationProvider,
};
use crate::ports::outbound::dictionary::{
    should_lookup_dictionary, DictionaryLookupRequest, DictionaryProvider,
};
use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};
use crate::ports::outbound::translation::{
    is_supported_source_language, is_supported_target_language,
    LibreTranslateLanguages, LocalEngineConfig, TranslationConfig, TranslationError,
    TranslationProvider, TranslationRequest, TranslationResult,
};
use crate::ports::outbound::word_insight::{
    GeneratedWordDetail, WordInsightProvider, WordInsightRequest,
};
use log::warn;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct TranslationService {
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
    local_engine_config: LocalEngineConfig,
    libretranslate_languages: LibreTranslateLanguages,
    language_detector: Arc<dyn LanguageDetector>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedLanguages {
    source_lang: String,
    target_lang: String,
}

async fn resolve_languages(
    settings: &HashMap<String, String>,
    language_detector: &dyn LanguageDetector,
    text: &str,
    detect_auto_source: bool,
) -> ResolvedLanguages {
    let configured_source = setting_or_default(settings, "sourceLanguage", "auto");
    let configured_target = setting_or_default(settings, "targetLanguage", "zh");
    let source_lang = if configured_source.trim().is_empty()
        || configured_source.trim().eq_ignore_ascii_case("auto")
    {
        if detect_auto_source {
            match language_detector.detect(text).await {
                DetectedLanguage::Known(lang) => lang,
                DetectedLanguage::Unknown => "auto".to_string(),
            }
        } else {
            "auto".to_string()
        }
    } else {
        configured_source
    };

    ResolvedLanguages {
        source_lang,
        target_lang: if configured_target.trim().is_empty() {
            "zh".to_string()
        } else {
            configured_target
        },
    }
}

fn validate_local_language_support(
    languages: &LibreTranslateLanguages,
    source_lang: &str,
    target_lang: &str,
) -> Result<(), TranslationError> {
    if !is_supported_source_language(languages, source_lang) {
        return Err(TranslationError::UnsupportedLanguage(
            source_lang.to_string(),
        ));
    }

    if !is_supported_target_language(languages, target_lang) {
        return Err(TranslationError::UnsupportedLanguage(
            target_lang.to_string(),
        ));
    }

    Ok(())
}

impl TranslationService {
    pub fn new(
        dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
        local_engine_config: LocalEngineConfig,
        libretranslate_languages: LibreTranslateLanguages,
        language_detector: Arc<dyn LanguageDetector>,
    ) -> Self {
        Self {
            dictionary_provider,
            local_engine_config,
            libretranslate_languages,
            language_detector,
        }
    }

    pub fn libretranslate_languages(&self) -> &LibreTranslateLanguages {
        &self.libretranslate_languages
    }

    pub async fn translate(
        &self,
        settings: HashMap<String, String>,
        text: String,
    ) -> Result<TranslationResult, String> {
        validate_text(&text).map_err(|e| e.to_string())?;
        let config = build_translation_config(&settings);
        let engine = config.engine.trim().to_lowercase();
        let resolved_languages = resolve_languages(
            &settings,
            self.language_detector.as_ref(),
            &text,
            engine == "local",
        )
        .await;

        // === local engine: try dictionary first ===
        if engine == "local"
            && should_lookup_dictionary(&text)
            && !resolved_languages
                .source_lang
                .trim()
                .eq_ignore_ascii_case("auto")
        {
            if let Some(provider) = &self.dictionary_provider {
                if provider.supports_language_pair(
                    &resolved_languages.source_lang,
                    &resolved_languages.target_lang,
                ) {
                    match provider.lookup(DictionaryLookupRequest {
                        text: text.clone(),
                        source_lang: resolved_languages.source_lang.clone(),
                        target_lang: resolved_languages.target_lang.clone(),
                    }) {
                        Ok(Some(result)) => {
                            return Ok(TranslationResult {
                                translation: result.translation,
                                detected_source_lang: Some(resolved_languages.source_lang),
                                phonetic: result.phonetic,
                                part_of_speech: result.part_of_speech,
                                definitions: result.definitions,
                                examples: result.examples,
                            });
                        }
                        Ok(None) => {}
                        Err(error) => {
                            warn!("Dictionary lookup failed, falling back to local LibreTranslate: {error}");
                        }
                    }
                }
            }
        }

        // === all engines: unified factory path ===
        // For "local" engine, use the configured LibreTranslate endpoint from local_engine_config
        let config = if engine == "local" {
            TranslationConfig {
                api_endpoint: self.local_engine_config.libretranslate_endpoint.clone(),
                ..config
            }
        } else {
            config
        };

        validate_local_language_support(
            &self.libretranslate_languages,
            &resolved_languages.source_lang,
            &resolved_languages.target_lang,
        )
        .map_err(|error| error.to_string())?;

        let provider = create_translation_provider(config)?;
        let request = TranslationRequest {
            text,
            source_lang: resolved_languages.source_lang.clone(),
            target_lang: resolved_languages.target_lang.clone(),
        };
        provider.translate(request).await.map_err(|e| e.to_string())
    }

    pub async fn generate_word_detail(
        &self,
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
        engine: setting_or_default(settings, "translationEngine", "local"),
        api_endpoint: setting_or_default(settings, "apiEndpoint", ""),
        api_key: setting_or_default(settings, "apiKey", ""),
        api_secret: setting_or_default(settings, "apiSecret", ""),
        api_region: setting_or_default(settings, "apiRegion", ""),
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
        "local" => LibreTranslateProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "custom" => CustomTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "deepl" => DeepLTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "google" => GoogleTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "microsoft" => MicrosoftTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "baidu" => BaiduTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "tencent" => TencentTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "youdao" => YoudaoTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        engine => Err(TranslationError::UnsupportedEngine(engine.to_string()).to_string()),
    }
}

fn create_word_insight_provider(
    config: TranslationConfig,
) -> Result<Box<dyn WordInsightProvider>, String> {
    match config.engine.trim().to_lowercase().as_str() {
        "custom" => CustomTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn WordInsightProvider>)
            .map_err(|error| error.to_string()),
        "local" | "libretranslate" | "deepl" | "google" | "microsoft" | "baidu" | "tencent"
        | "youdao" => Err(TranslationError::UnsupportedEngine(config.engine).to_string()),
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
    use crate::ports::outbound::dictionary::{DictionaryError, DictionaryLookupResult};
    use crate::ports::outbound::translation::LibreTranslateLanguage;

    struct MockLanguageDetector {
        result: DetectedLanguage,
        calls: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl LanguageDetector for MockLanguageDetector {
        fn detect<'a>(
            &'a self,
            _text: &'a str,
        ) -> crate::ports::outbound::language_detection::LanguageDetectionFuture<'a> {
            Box::pin(async move {
                self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.result.clone()
            })
        }
    }

    struct MockDictionaryProvider {
        supports: bool,
        result: Option<DictionaryLookupResult>,
    }

    impl DictionaryProvider for MockDictionaryProvider {
        fn lookup(
            &self,
            _request: DictionaryLookupRequest,
        ) -> Result<Option<DictionaryLookupResult>, DictionaryError> {
            Ok(self.result.clone())
        }

        fn supports_language_pair(&self, _source_lang: &str, _target_lang: &str) -> bool {
            self.supports
        }
    }

    struct CountingDictionaryProvider {
        lookup_calls: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl DictionaryProvider for CountingDictionaryProvider {
        fn lookup(
            &self,
            _request: DictionaryLookupRequest,
        ) -> Result<Option<DictionaryLookupResult>, DictionaryError> {
            self.lookup_calls
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Some(DictionaryLookupResult {
                word: "hello".to_string(),
                translation: "dictionary result".to_string(),
                phonetic: None,
                part_of_speech: Vec::new(),
                definitions: Vec::new(),
                examples: Vec::new(),
            }))
        }

        fn supports_language_pair(&self, _source_lang: &str, _target_lang: &str) -> bool {
            true
        }
    }

    fn test_libretranslate_languages() -> LibreTranslateLanguages {
        LibreTranslateLanguages {
            source_languages: vec![
                LibreTranslateLanguage {
                    code: "auto".to_string(),
                    name: "Auto Detect".to_string(),
                },
                LibreTranslateLanguage {
                    code: "en".to_string(),
                    name: "English".to_string(),
                },
                LibreTranslateLanguage {
                    code: "ja".to_string(),
                    name: "Japanese".to_string(),
                },
            ],
            target_languages: vec![
                LibreTranslateLanguage {
                    code: "zh".to_string(),
                    name: "Chinese".to_string(),
                },
                LibreTranslateLanguage {
                    code: "en".to_string(),
                    name: "English".to_string(),
                },
            ],
        }
    }

    fn test_language_detector() -> Arc<dyn LanguageDetector> {
        Arc::new(MockLanguageDetector {
            result: DetectedLanguage::Known("en".to_string()),
            calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    fn service_with_dictionary(
        supports: bool,
        result: Option<DictionaryLookupResult>,
    ) -> TranslationService {
        TranslationService::new(
            Some(Arc::new(MockDictionaryProvider { supports, result })),
            LocalEngineConfig::default_local(),
            test_libretranslate_languages(),
            test_language_detector(),
        )
    }

    fn service_with_detector_and_config(
        dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
        detector: Arc<dyn LanguageDetector>,
        local_engine_config: LocalEngineConfig,
    ) -> TranslationService {
        TranslationService::new(
            dictionary_provider,
            local_engine_config,
            test_libretranslate_languages(),
            detector,
        )
    }

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

    #[test]
    fn build_translation_config_reads_secret_region_and_defaults_to_local() {
        let settings = HashMap::from([
            ("apiSecret".to_string(), "secret-value".to_string()),
            ("apiRegion".to_string(), "eastasia".to_string()),
        ]);

        let config = build_translation_config(&settings);

        assert_eq!(config.engine, "local");
        assert_eq!(config.api_secret, "secret-value");
        assert_eq!(config.api_region, "eastasia");
    }

    #[test]
    fn resolve_languages_uses_detected_source_when_configured_auto() {
        let settings = HashMap::from([
            ("sourceLanguage".to_string(), "auto".to_string()),
            ("targetLanguage".to_string(), "zh-CN".to_string()),
        ]);
        let detector = MockLanguageDetector {
            result: DetectedLanguage::Known("en".to_string()),
            calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };

        let result =
            tauri::async_runtime::block_on(resolve_languages(&settings, &detector, "hello", true));

        assert_eq!(
            result,
            ResolvedLanguages {
                source_lang: "en".to_string(),
                target_lang: "zh-CN".to_string(),
            }
        );
    }

    #[test]
    fn resolve_languages_defaults_to_auto_source_and_zh_target() {
        let settings = HashMap::new();
        let detector = MockLanguageDetector {
            result: DetectedLanguage::Unknown,
            calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };

        let result =
            tauri::async_runtime::block_on(resolve_languages(&settings, &detector, "hello", true));

        assert_eq!(
            result,
            ResolvedLanguages {
                source_lang: "auto".to_string(),
                target_lang: "zh".to_string(),
            }
        );
    }

    #[test]
    fn validate_local_language_support_rejects_unknown_target() {
        let languages = test_libretranslate_languages();
        let result = validate_local_language_support(&languages, "en", "xx");

        assert_eq!(
            result,
            Err(TranslationError::UnsupportedLanguage("xx".to_string()))
        );
    }

    #[test]
    fn translate_returns_dictionary_result_for_local_supported_pair_from_settings() {
        let service = service_with_dictionary(
            true,
            Some(DictionaryLookupResult {
                word: "hello".to_string(),
                translation: "int. 你好".to_string(),
                phonetic: Some("həˈləʊ".to_string()),
                part_of_speech: vec!["int".to_string()],
                definitions: vec!["int. 你好".to_string()],
                examples: Vec::new(),
            }),
        );
        let settings = HashMap::from([
            ("translationEngine".to_string(), "local".to_string()),
            ("sourceLanguage".to_string(), "en".to_string()),
            ("targetLanguage".to_string(), "zh-CN".to_string()),
        ]);

        let result =
            tauri::async_runtime::block_on(service.translate(settings, "hello".to_string()))
                .unwrap();

        assert_eq!(result.translation, "int. 你好");
    }

    #[test]
    fn translate_does_not_detect_language_for_custom_engine_auto_source() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let detector = Arc::new(MockLanguageDetector {
            result: DetectedLanguage::Known("en".to_string()),
            calls: Arc::clone(&calls),
        });
        let service =
            service_with_detector_and_config(None, detector, LocalEngineConfig::default_local());
        let settings = HashMap::from([
            ("translationEngine".to_string(), "custom".to_string()),
            ("sourceLanguage".to_string(), "auto".to_string()),
            ("targetLanguage".to_string(), "zh".to_string()),
        ]);

        let error =
            tauri::async_runtime::block_on(service.translate(settings, "hello".to_string()))
                .unwrap_err();

        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(error.contains("API"));
    }

    #[test]
    fn translate_does_not_detect_language_for_local_fixed_source() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let detector = Arc::new(MockLanguageDetector {
            result: DetectedLanguage::Known("ja".to_string()),
            calls: Arc::clone(&calls),
        });
        let service = service_with_detector_and_config(
            Some(Arc::new(MockDictionaryProvider {
                supports: true,
                result: Some(DictionaryLookupResult {
                    word: "hello".to_string(),
                    translation: "int. 你好".to_string(),
                    phonetic: None,
                    part_of_speech: Vec::new(),
                    definitions: Vec::new(),
                    examples: Vec::new(),
                }),
            })),
            detector,
            LocalEngineConfig::default_local(),
        );
        let settings = HashMap::from([
            ("translationEngine".to_string(), "local".to_string()),
            ("sourceLanguage".to_string(), "en".to_string()),
            ("targetLanguage".to_string(), "zh-CN".to_string()),
        ]);

        let result =
            tauri::async_runtime::block_on(service.translate(settings, "hello".to_string()))
                .unwrap();

        assert_eq!(result.translation, "int. 你好");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn translate_skips_dictionary_when_detection_unknown() {
        let lookup_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let detector = Arc::new(MockLanguageDetector {
            result: DetectedLanguage::Unknown,
            calls: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        });
        let service = service_with_detector_and_config(
            Some(Arc::new(CountingDictionaryProvider {
                lookup_calls: Arc::clone(&lookup_calls),
            })),
            detector,
            LocalEngineConfig {
                libretranslate_endpoint: ":// invalid".to_string(),
            },
        );
        let settings = HashMap::from([
            ("translationEngine".to_string(), "local".to_string()),
            ("sourceLanguage".to_string(), "auto".to_string()),
            ("targetLanguage".to_string(), "zh".to_string()),
        ]);

        let error =
            tauri::async_runtime::block_on(service.translate(settings, "hello".to_string()))
                .unwrap_err();

        assert_eq!(lookup_calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(!error.is_empty());
    }

    #[test]
    fn translate_detects_language_for_local_auto_source_before_dictionary_lookup() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let detector = Arc::new(MockLanguageDetector {
            result: DetectedLanguage::Known("en".to_string()),
            calls: Arc::clone(&calls),
        });
        let service = service_with_detector_and_config(
            Some(Arc::new(MockDictionaryProvider {
                supports: true,
                result: Some(DictionaryLookupResult {
                    word: "hello".to_string(),
                    translation: "dictionary result".to_string(),
                    phonetic: None,
                    part_of_speech: Vec::new(),
                    definitions: Vec::new(),
                    examples: Vec::new(),
                }),
            })),
            detector,
            LocalEngineConfig::default_local(),
        );
        let settings = HashMap::from([
            ("translationEngine".to_string(), "local".to_string()),
            ("sourceLanguage".to_string(), "auto".to_string()),
            ("targetLanguage".to_string(), "zh-CN".to_string()),
        ]);

        let result =
            tauri::async_runtime::block_on(service.translate(settings, "hello".to_string()))
                .unwrap();

        assert_eq!(result.translation, "dictionary result");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
