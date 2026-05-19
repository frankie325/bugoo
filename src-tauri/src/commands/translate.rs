use crate::adapters::outbound::translation::{
    deepl::DeepLTranslationProvider, google::GoogleTranslationProvider,
    openai::OpenAiTranslationProvider,
};
use crate::commands::AppState;
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationProvider, TranslationRequest, TranslationResult,
};
use crate::ports::outbound::word_insight::WordInsightProvider;
use std::collections::HashMap;

#[tauri::command]
pub async fn translate_text(
    state: tauri::State<'_, AppState>,
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<TranslationResult, String> {
    let config = load_translation_config(state.inner())?;
    let provider = create_translation_provider(config)?;
    let request = TranslationRequest {
        text,
        source_lang,
        target_lang,
    };

    provider
        .translate(request)
        .await
        .map_err(|error| error.to_string())
}

pub(crate) fn load_translation_config(state: &AppState) -> Result<TranslationConfig, String> {
    let settings = read_settings_map(state)?;
    Ok(TranslationConfig {
        engine: setting_or_default(&settings, "translationEngine", "openai"),
        api_endpoint: setting_or_default(&settings, "apiEndpoint", "https://api.openai.com/v1"),
        api_key: setting_or_default(&settings, "apiKey", ""),
        translation_model: setting_or_default(&settings, "translationModel", "gpt-4o-mini"),
        translation_prompt: setting_or_default(&settings, "translationPrompt", ""),
        word_detail_prompt: setting_or_default(&settings, "wordDetailPrompt", ""),
        timeout_ms: setting_or_default(&settings, "translationTimeoutMs", "15000")
            .parse::<u64>()
            .unwrap_or(15_000),
    })
}

pub(crate) fn create_translation_provider(
    config: TranslationConfig,
) -> Result<Box<dyn TranslationProvider>, String> {
    match config.engine.trim().to_lowercase().as_str() {
        "openai" | "custom" => OpenAiTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "deepl" => Ok(Box::new(DeepLTranslationProvider)),
        "google" => Ok(Box::new(GoogleTranslationProvider)),
        engine => Err(TranslationError::UnsupportedEngine(engine.to_string()).to_string()),
    }
}

pub(crate) fn create_word_insight_provider(
    config: TranslationConfig,
) -> Result<Box<dyn WordInsightProvider>, String> {
    match config.engine.trim().to_lowercase().as_str() {
        "openai" | "custom" => OpenAiTranslationProvider::new(config)
            .map(|provider| Box::new(provider) as Box<dyn WordInsightProvider>)
            .map_err(|error| error.to_string()),
        "deepl" | "google" => Err(TranslationError::UnsupportedEngine(config.engine).to_string()),
        engine => Err(TranslationError::UnsupportedEngine(engine.to_string()).to_string()),
    }
}

pub(crate) fn read_settings_map(state: &AppState) -> Result<HashMap<String, String>, String> {
    let conn = state.db.connection();
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?;

    let mut settings = HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(|error| error.to_string())?;
        settings.insert(key, value);
    }

    Ok(settings)
}

fn setting_or_default(settings: &HashMap<String, String>, key: &str, default: &str) -> String {
    settings
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| default.to_string())
}
