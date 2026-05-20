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
pub struct GoogleTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct GoogleTranslateRequest {
    q: String,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    format: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateResponse {
    data: GoogleTranslateData,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslation {
    #[serde(rename = "translatedText")]
    translated_text: String,
    #[serde(rename = "detectedSourceLanguage")]
    detected_source_language: Option<String>,
}

impl GoogleTranslationProvider {
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

    fn endpoint(&self) -> String {
        if self.config.api_endpoint.trim().is_empty() {
            "https://translation.googleapis.com/language/translate/v2".to_string()
        } else {
            self.config.api_endpoint.clone()
        }
    }

    async fn translate_inner(
        &self,
        request: TranslationRequest,
    ) -> Result<crate::ports::outbound::translation::TranslationResult, TranslationError> {
        validate_text(&request.text)?;

        let source = optional_lang(&request.source_lang);
        let payload = GoogleTranslateRequest {
            q: request.text,
            target: request.target_lang.trim().to_lowercase(),
            source: source.clone(),
            format: "text".to_string(),
        };

        let response = self
            .client
            .post(self.endpoint())
            .query(&[("key", self.config.api_key.trim())])
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
            .json::<GoogleTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;

        let translated = body
            .data
            .translations
            .into_iter()
            .next()
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(
            translated.translated_text,
            translated.detected_source_language.or(source),
        ))
    }
}

impl TranslationProvider for GoogleTranslationProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

fn optional_lang(lang: &str) -> Option<String> {
    let value = lang.trim().to_lowercase();
    if value.is_empty() || value == "auto" {
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
            engine: "google".to_string(),
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

        let result = GoogleTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingApiKey)));
    }

    #[test]
    fn optional_lang_omits_auto() {
        assert_eq!(optional_lang("auto"), None);
        assert_eq!(optional_lang("EN"), Some("en".to_string()));
    }
}
