use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::{normalize_endpoint, validate_text};
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct LibreTranslateProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct LibreTranslateRequest {
    q: String,
    source: String,
    target: String,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LibreTranslateResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

impl LibreTranslateProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_endpoint.trim().is_empty() {
            return Err(TranslationError::MissingEndpoint);
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

        let source = normalize_source_lang(&request.source_lang);
        let target = normalize_target_lang(&request.target_lang);
        let payload = LibreTranslateRequest {
            q: request.text,
            source: source.clone(),
            target,
            format: "text".to_string(),
            api_key: non_empty_api_key(&self.config.api_key),
        };

        let response = self
            .client
            .post(format!(
                "{}/translate",
                normalize_endpoint(&self.config.api_endpoint)
            ))
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
            .json::<LibreTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;

        if body.translated_text.trim().is_empty() {
            return Err(TranslationError::InvalidResponse);
        }

        Ok(empty_translation_result(
            body.translated_text,
            if source == "auto" { None } else { Some(source) },
        ))
    }
}

impl TranslationProvider for LibreTranslateProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

fn normalize_source_lang(lang: &str) -> String {
    let normalized = lang.trim().to_lowercase();
    if normalized.is_empty() || normalized == "auto" {
        "auto".to_string()
    } else {
        normalized
    }
}

fn normalize_target_lang(lang: &str) -> String {
    lang.trim().to_lowercase()
}

fn non_empty_api_key(api_key: &str) -> Option<String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "libretranslate".to_string(),
            api_endpoint: "http://localhost:5000".to_string(),
            api_key: String::new(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn new_rejects_missing_endpoint() {
        let mut config = config();
        config.api_endpoint = " ".to_string();

        let result = LibreTranslateProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingEndpoint)));
    }

    #[test]
    fn normalize_source_lang_uses_auto_for_empty() {
        assert_eq!(normalize_source_lang(""), "auto");
        assert_eq!(normalize_source_lang("EN"), "en");
    }
}
