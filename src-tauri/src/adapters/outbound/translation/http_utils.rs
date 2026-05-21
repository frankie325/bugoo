use crate::ports::outbound::translation::{TranslationError, TranslationResult};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Option<CustomError>,
}

#[derive(Debug, Deserialize)]
struct CustomError {
    message: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    code: Option<Value>,
}

pub(crate) fn timeout_duration(timeout_ms: u64) -> Duration {
    if timeout_ms == 0 {
        Duration::from_secs(30)
    } else {
        Duration::from_millis(timeout_ms)
    }
}

pub(crate) fn empty_translation_result(
    translation: String,
    detected_source_lang: Option<String>,
) -> TranslationResult {
    TranslationResult {
        translation,
        detected_source_lang,
        phonetic: None,
        part_of_speech: Vec::new(),
        definitions: Vec::new(),
        examples: Vec::new(),
    }
}

pub(crate) fn map_reqwest_error(error: reqwest::Error) -> TranslationError {
    if error.is_timeout() {
        TranslationError::RequestTimeout
    } else {
        TranslationError::RequestFailed(error.to_string())
    }
}

pub(crate) fn format_http_error(
    status: reqwest::StatusCode,
    body: &str,
    secrets: &[&str],
) -> String {
    let status_text = format!("HTTP {status}");
    let redacted_body = secrets
        .iter()
        .fold(body.to_string(), |acc, secret| redact_secret(&acc, secret));
    let trimmed = redacted_body.trim();
    if trimmed.is_empty() {
        return status_text;
    }

    if let Ok(parsed) = serde_json::from_str::<ErrorResponse>(trimmed) {
        if let Some(error) = parsed.error {
            if let Some(message) = non_empty_string(error.message) {
                let mut details = Vec::new();
                if let Some(kind) = non_empty_string(error.kind) {
                    details.push(format!("type: {}", truncate_for_error(&kind)));
                }
                if let Some(code) = error.code.and_then(error_code_to_string) {
                    details.push(format!("code: {}", truncate_for_error(&code)));
                }

                let message = truncate_for_error(&message);
                if details.is_empty() {
                    return format!("{status_text}: {message}");
                }
                return format!("{status_text}: {message} ({})", details.join(", "));
            }
        }
    }

    format!("{status_text}: {}", truncate_for_error(trimmed))
}

fn redact_secret(value: &str, secret: &str) -> String {
    let secret = secret.trim();
    if secret.is_empty() || !value.contains(secret) {
        return value.to_string();
    }

    value.replace(secret, "[redacted]")
}

fn non_empty_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn error_code_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(value) => non_empty_string(Some(value)),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn truncate_for_error(value: &str) -> String {
    const MAX_ERROR_BODY_CHARS: usize = 500;
    if value.chars().count() <= MAX_ERROR_BODY_CHARS {
        return value.to_string();
    }

    let mut truncated = value.chars().take(MAX_ERROR_BODY_CHARS).collect::<String>();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_secrets_hides_all_non_empty_values() {
        assert_eq!(
            redact_secret("key=abc secret=def", "abc"),
            "key=[redacted] secret=def"
        );
        assert_eq!(
            redact_secret("key=abc secret=def", "def"),
            "key=abc secret=[redacted]"
        );
    }

    #[test]
    fn empty_translation_result_has_learning_fields_empty() {
        let result = empty_translation_result("你好".to_string(), Some("en".to_string()));

        assert_eq!(result.translation, "你好");
        assert_eq!(result.detected_source_lang, Some("en".to_string()));
        assert_eq!(result.phonetic, None);
        assert!(result.part_of_speech.is_empty());
        assert!(result.definitions.is_empty());
        assert!(result.examples.is_empty());
    }
}
