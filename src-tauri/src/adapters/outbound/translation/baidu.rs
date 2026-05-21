use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::validate_text;
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct BaiduTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Deserialize)]
struct BaiduTranslateResponse {
    trans_result: Option<Vec<BaiduTranslationItem>>,
}

#[derive(Debug, Deserialize)]
struct BaiduTranslationItem {
    dst: String,
}

impl BaiduTranslationProvider {
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
            "https://fanyi-api.baidu.com/api/trans/vip/translate".to_string()
        } else {
            self.config.api_endpoint.clone()
        };

        let salt = format!("{}", chrono::Utc::now().timestamp_millis());
        let source = normalize_source_lang(&request.source_lang);
        let target = normalize_target_lang(&request.target_lang);
        let sign = build_sign(
            self.config.api_key.trim(),
            &request.text,
            &salt,
            self.config.api_secret.trim(),
        );

        let response = self
            .client
            .post(endpoint)
            .form(&[
                ("q", request.text),
                ("from", source),
                ("to", target),
                ("appid", self.config.api_key.trim().to_string()),
                ("salt", salt),
                ("sign", sign),
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
            .json::<BaiduTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let item = body
            .trans_result
            .and_then(|rows| rows.into_iter().next())
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(item.dst, None))
    }
}

impl TranslationProvider for BaiduTranslationProvider {
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

fn build_sign(appid: &str, q: &str, salt: &str, secret: &str) -> String {
    format!(
        "{:x}",
        md5::compute(format!("{appid}{q}{salt}{secret}").as_bytes())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "baidu".to_string(),
            api_endpoint: String::new(),
            api_key: "appid".to_string(),
            api_secret: "secret".to_string(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn build_sign_matches_baidu_rule() {
        assert_eq!(
            build_sign("appid", "apple", "salt", "secret"),
            format!("{:x}", md5::compute("appidapplesaltsecret"))
        );
    }

    #[test]
    fn new_rejects_missing_secret() {
        let mut config = config();
        config.api_secret = " ".to_string();
        let result = BaiduTranslationProvider::new(config);
        assert!(matches!(result, Err(TranslationError::MissingApiSecret)));
    }
}
