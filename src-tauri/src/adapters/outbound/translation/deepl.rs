use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::validate_text;
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DeepLTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslateResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslation {
    text: String,
    detected_source_language: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeepLTranslateRequest {
    text: Vec<String>,
    target_lang: String,
    source_lang: Option<String>,
}

impl DeepLTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
        }

        let client = Client::builder()
            .timeout(timeout_duration(config.timeout_ms))
            .build()
            .map_err(|error| TranslationError::RequestFailed(error.to_string()))?;
        Ok(Self { client, config })
    }

    async fn translate_inner(
        &self,
        request: TranslationRequest,
    ) -> Result<crate::ports::outbound::translation::TranslationResult, TranslationError> {
        validate_text(&request.text)?;
        let endpoint = self.config.api_endpoint.clone();

        let source_lang = optional_deepl_lang(&request.source_lang);
        let payload = DeepLTranslateRequest {
            text: vec![request.text],
            target_lang: request.target_lang.trim().to_uppercase(),
            source_lang,
        };

        let response = self
            .client
            .post(endpoint)
            .header(
                "Authorization",
                format!("DeepL-Auth-Key {}", self.config.api_key.trim()),
            )
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.map_err(map_reqwest_error)?;
            return Err(TranslationError::RequestFailed(format_http_error(
                status,
                &body,
                &[&self.config.api_key],
            )));
        }

        let body = response
            .json::<DeepLTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let translated = body
            .translations
            .into_iter()
            .next()
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(
            translated.text,
            translated
                .detected_source_language
                .map(|value| value.to_string()),
        ))
    }
}

impl TranslationProvider for DeepLTranslationProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

fn optional_deepl_lang(lang: &str) -> Option<String> {
    let value = lang.trim().to_uppercase();
    if value.is_empty() || value == "AUTO" {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "deepl".to_string(),
            api_endpoint: String::new(),
            api_key: "key".to_string(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn new_rejects_missing_api_key() {
        let mut config = config();
        config.api_key = " ".to_string();

        let result = DeepLTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingApiKey)));
    }

    #[test]
    fn optional_deepl_lang_omits_auto() {
        assert_eq!(optional_deepl_lang("auto"), None);
        assert_eq!(optional_deepl_lang("en"), Some("EN".to_string()));
    }
}
