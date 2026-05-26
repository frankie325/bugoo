use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::validate_text;
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider, TranslationRequest,
};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct TencentTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct TencentTranslateRequest {
    #[serde(rename = "SourceText")]
    source_text: String,
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Target")]
    target: String,
    #[serde(rename = "ProjectId")]
    project_id: i32,
}

#[derive(Debug, Deserialize)]
struct TencentTranslateResponse {
    #[serde(rename = "Response")]
    response: TencentTranslateBody,
}

#[derive(Debug, Deserialize)]
struct TencentTranslateBody {
    #[serde(rename = "TargetText")]
    target_text: Option<String>,
}

impl TencentTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
        }
        if config.api_secret.trim().is_empty() {
            return Err(TranslationError::MissingApiSecret);
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

    async fn translate_inner(
        &self,
        request: TranslationRequest,
    ) -> Result<crate::ports::outbound::translation::TranslationResult, TranslationError> {
        validate_text(&request.text)?;

        let source = normalize_source_lang(&request.source_lang);
        let target = normalize_target_lang(&request.target_lang);
        let payload = TencentTranslateRequest {
            source_text: request.text,
            source,
            target,
            project_id: 0,
        };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|error| TranslationError::RequestFailed(error.to_string()))?;

        let timestamp = chrono::Utc::now().timestamp();
        let authorization = build_authorization(
            self.config.api_key.trim(),
            self.config.api_secret.trim(),
            timestamp,
            &payload_json,
        );

        let response = self
            .client
            .post("https://tmt.tencentcloudapi.com")
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", "tmt.tencentcloudapi.com")
            .header("Authorization", authorization)
            .header("X-TC-Action", "TextTranslate")
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Version", "2018-03-21")
            .header("X-TC-Region", self.config.api_region.trim())
            .body(payload_json)
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
            .json::<TencentTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let translation = body
            .response
            .target_text
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(translation, None))
    }
}

impl TranslationProvider for TencentTranslationProvider {
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

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    hex::encode(digest)
}

fn hmac_sha256(key: &[u8], message: &str) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(key).expect("HMAC can take key of any size for SHA-256");
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn build_authorization(secret_id: &str, secret_key: &str, timestamp: i64, payload: &str) -> String {
    let date = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%d")
        .to_string();
    let service = "tmt";
    let host = "tmt.tencentcloudapi.com";
    let signed_headers = "content-type;host";
    let hashed_payload = sha256_hex(payload);
    let canonical_request = format!(
        "POST\n/\n\ncontent-type:application/json; charset=utf-8\nhost:{host}\n\n{signed_headers}\n{hashed_payload}"
    );
    let credential_scope = format!("{date}/{service}/tc3_request");
    let string_to_sign = format!(
        "TC3-HMAC-SHA256\n{timestamp}\n{credential_scope}\n{}",
        sha256_hex(&canonical_request)
    );

    let secret_date = hmac_sha256(format!("TC3{secret_key}").as_bytes(), &date);
    let secret_service = hmac_sha256(&secret_date, service);
    let secret_signing = hmac_sha256(&secret_service, "tc3_request");
    let signature = hex::encode(hmac_sha256(&secret_signing, &string_to_sign));

    format!(
        "TC3-HMAC-SHA256 Credential={secret_id}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "tencent".to_string(),
            api_endpoint: String::new(),
            api_key: "secret-id".to_string(),
            api_secret: "secret-key".to_string(),
            api_region: "ap-guangzhou".to_string(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn sha256_hex_matches_known_value() {
        assert_eq!(
            sha256_hex("hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn new_rejects_missing_secret() {
        let mut config = config();
        config.api_secret = String::new();

        let result = TencentTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingApiSecret)));
    }
}
