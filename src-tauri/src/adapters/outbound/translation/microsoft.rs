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
pub struct MicrosoftTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct MicrosoftTranslateRequest {
    #[serde(rename = "Text")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTranslateResponseItem {
    #[serde(rename = "detectedLanguage")]
    detected_language: Option<DetectedLanguage>,
    translations: Vec<MicrosoftTranslation>,
}

#[derive(Debug, Deserialize)]
struct DetectedLanguage {
    language: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTranslation {
    text: String,
}

impl MicrosoftTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
        }
        if config.api_region.trim().is_empty() {
            return Err(TranslationError::MissingRegion);
        }

        let client = Client::builder()
            .timeout(timeout_duration(config.timeout_ms))
            .build()
            .map_err(|error| TranslationError::RequestFailed(error.to_string()))?;
        Ok(Self { client, config })
    }

    fn endpoint(&self) -> String {
        self.config.api_endpoint.clone()
    }

    async fn translate_inner(
        &self,
        request: TranslationRequest,
    ) -> Result<crate::ports::outbound::translation::TranslationResult, TranslationError> {
        validate_text(&request.text)?;

        let mut query = vec![
            ("api-version", "3.0".to_string()),
            ("to", request.target_lang.trim().to_string()),
        ];
        let source = request.source_lang.trim().to_string();
        if !source.is_empty() && source != "auto" {
            query.push(("from", source.clone()));
        }

        let body = vec![MicrosoftTranslateRequest { text: request.text }];
        let response = self
            .client
            .post(self.endpoint())
            .query(&query)
            .header("Ocp-Apim-Subscription-Key", self.config.api_key.trim())
            .header(
                "Ocp-Apim-Subscription-Region",
                self.config.api_region.trim(),
            )
            .header("Content-Type", "application/json")
            .json(&body)
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
            .json::<Vec<MicrosoftTranslateResponseItem>>()
            .await
            .map_err(map_reqwest_error)?;
        let first = body
            .into_iter()
            .next()
            .ok_or(TranslationError::InvalidResponse)?;
        let translation = first
            .translations
            .into_iter()
            .next()
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(
            translation.text,
            first
                .detected_language
                .map(|value| value.language)
                .or_else(|| {
                    if source == "auto" || source.is_empty() {
                        None
                    } else {
                        Some(source)
                    }
                }),
        ))
    }
}

impl TranslationProvider for MicrosoftTranslationProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "microsoft".to_string(),
            api_endpoint: String::new(),
            api_key: "key".to_string(),
            api_secret: String::new(),
            api_region: "eastasia".to_string(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn new_rejects_missing_region() {
        let mut config = config();
        config.api_region = " ".to_string();

        let result = MicrosoftTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingRegion)));
    }
}
