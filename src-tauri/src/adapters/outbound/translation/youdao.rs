use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::validate_text;
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct YoudaoTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Deserialize)]
struct YoudaoTranslateResponse {
    translation: Option<Vec<String>>,
}

impl YoudaoTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
        }
        if config.api_secret.trim().is_empty() {
            return Err(TranslationError::MissingApiSecret);
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

        let endpoint = if self.config.api_endpoint.trim().is_empty() {
            "https://openapi.youdao.com/api".to_string()
        } else {
            self.config.api_endpoint.clone()
        };

        let salt = format!("{}", chrono::Utc::now().timestamp_millis());
        let curtime = format!("{}", chrono::Utc::now().timestamp());
        let sign = build_sign(
            self.config.api_key.trim(),
            &request.text,
            &salt,
            &curtime,
            self.config.api_secret.trim(),
        );

        let source = normalize_source_lang(&request.source_lang);
        let target = normalize_target_lang(&request.target_lang);

        let response = self
            .client
            .post(endpoint)
            .form(&[
                ("q", request.text),
                ("from", source),
                ("to", target),
                ("appKey", self.config.api_key.trim().to_string()),
                ("salt", salt),
                ("sign", sign),
                ("signType", "v3".to_string()),
                ("curtime", curtime),
            ])
            .send()
            .await
            .map_err(map_reqwest_error)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.map_err(map_reqwest_error)?;
            return Err(TranslationError::RequestFailed(format_http_error(
                status,
                &body,
                &[&self.config.api_key, &self.config.api_secret],
            )));
        }

        let body = response
            .json::<YoudaoTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let translation = body
            .translation
            .and_then(|rows| rows.into_iter().next())
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(translation, None))
    }
}

impl TranslationProvider for YoudaoTranslationProvider {
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

fn build_sign(app_key: &str, q: &str, salt: &str, curtime: &str, app_secret: &str) -> String {
    let raw = format!(
        "{app_key}{}{salt}{curtime}{app_secret}",
        truncate_for_sign(q)
    );
    let digest = Sha256::digest(raw.as_bytes());
    format!("{digest:x}")
}

fn truncate_for_sign(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.len() <= 20 {
        input.to_string()
    } else {
        format!(
            "{}{}{}",
            chars.iter().take(10).collect::<String>(),
            chars.len(),
            chars
                .iter()
                .skip(chars.len().saturating_sub(10))
                .collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_for_sign_short_input() {
        assert_eq!(truncate_for_sign("hello"), "hello");
    }

    #[test]
    fn truncate_for_sign_long_input() {
        assert_eq!(
            truncate_for_sign("abcdefghijklmnopqrstuvwxyz"),
            "abcdefghij26qrstuvwxyz"
        );
    }
}
