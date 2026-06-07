use crate::adapters::outbound::translation::http_utils::{map_reqwest_error, timeout_duration};
use crate::domain::services::translation_service::normalize_endpoint;
use crate::ports::outbound::language_detection::{
    DetectedLanguage, LanguageDetectionFuture, LanguageDetector,
};
use log::warn;
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct LibreTranslateLanguageDetector {
    client: Client,
    endpoint: String,
}

#[derive(Debug, Deserialize)]
struct DetectResponseItem {
    confidence: f64,
    language: String,
}

impl LibreTranslateLanguageDetector {
    pub fn new(endpoint: String, timeout_ms: u64) -> Self {
        let client = Client::builder()
            .timeout(timeout_duration(timeout_ms))
            .no_proxy()
            .build()
            .unwrap_or_else(|error| {
                warn!("Failed to configure LibreTranslate language detector timeout: {error}");
                Client::new()
            });

        Self { client, endpoint }
    }

    async fn detect_inner(&self, text: &str) -> DetectedLanguage {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return DetectedLanguage::Unknown;
        }

        match self.request_detection(trimmed).await {
            Ok(Some(language)) => DetectedLanguage::Known(language),
            Ok(None) => DetectedLanguage::Unknown,
            Err(error) => {
                warn!("LibreTranslate language detection failed: {error}");
                DetectedLanguage::Unknown
            }
        }
    }

    async fn request_detection(&self, text: &str) -> Result<Option<String>, String> {
        let response = self
            .client
            .post(format!("{}/detect", normalize_endpoint(&self.endpoint)))
            .form(&[("q", text)])
            .send()
            .await
            .map_err(|error| map_reqwest_error(error).to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "LibreTranslate /detect returned {}: {}",
                status, body
            ));
        }

        let items = response
            .json::<Vec<DetectResponseItem>>()
            .await
            .map_err(|error| map_reqwest_error(error).to_string())?;

        Ok(select_detected_language(items))
    }
}

impl LanguageDetector for LibreTranslateLanguageDetector {
    fn detect<'a>(&'a self, text: &'a str) -> LanguageDetectionFuture<'a> {
        Box::pin(async move { self.detect_inner(text).await })
    }
}

fn select_detected_language(items: Vec<DetectResponseItem>) -> Option<String> {
    items
        .into_iter()
        .filter(|item| !item.language.trim().is_empty())
        .max_by(|left, right| left.confidence.total_cmp(&right.confidence))
        .map(|item| item.language.trim().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};

    #[test]
    fn select_detected_language_parses_detect_response_and_returns_highest_confidence_language() {
        let items = serde_json::from_str::<Vec<DetectResponseItem>>(
            r#"[{"confidence":12.0,"language":"fr"},{"confidence":88.0,"language":"EN"}]"#,
        )
        .unwrap();

        let result = select_detected_language(items);

        assert_eq!(result, Some("en".to_string()));
    }

    #[test]
    fn detect_returns_unknown_for_empty_or_invalid_endpoint() {
        let detector = LibreTranslateLanguageDetector::new(":// invalid".to_string(), 1_000);

        let empty = tauri::async_runtime::block_on(detector.detect("  "));
        let failed = tauri::async_runtime::block_on(detector.detect("Hello world"));

        assert_eq!(empty, DetectedLanguage::Unknown);
        assert_eq!(failed, DetectedLanguage::Unknown);
    }
}
