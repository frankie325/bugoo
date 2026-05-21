use crate::adapters::outbound::translation::http_utils::{
    format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::{normalize_endpoint, validate_text};
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationExample, TranslationFuture, TranslationProvider,
    TranslationRequest, TranslationResult,
};
use crate::ports::outbound::word_insight::{
    GeneratedWordDetail, WordInsightFuture, WordInsightProvider, WordInsightRequest,
};
use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const SYSTEM_TRANSLATION_PROMPT: &str = include_str!("prompts/system_translation_prompt.txt");
const WORD_DETAIL_PROMPT: &str = include_str!("prompts/word_detail_prompt.txt");

#[derive(Clone)]
pub struct CustomTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    response_format: ResponseFormat,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslationResponse {
    translation: String,
    #[serde(default, alias = "detected_source_lang")]
    detected_source_lang: Option<String>,
    #[serde(default)]
    phonetic: Option<String>,
    #[serde(default, alias = "part_of_speech")]
    part_of_speech: Vec<String>,
    #[serde(default)]
    definitions: Vec<String>,
    #[serde(default)]
    examples: Vec<TranslationExample>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WordDetailResponse {
    translation: String,
    phonetic: Option<String>,
    part_of_speech: Vec<String>,
    definitions: Vec<String>,
    examples: Vec<TranslationExample>,
    memory_tip: String,
    detail: String,
}

impl CustomTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_endpoint.trim().is_empty() {
            return Err(TranslationError::MissingEndpoint);
        }
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
        }
        if config.translation_model.trim().is_empty() {
            return Err(TranslationError::MissingModel);
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
    ) -> Result<TranslationResult, TranslationError> {
        validate_text(&request.text)?;

        let user_prompt = format!(
            "请将以下文本从 {source_lang} 翻译为 {target_lang}，只返回 JSON：\n\
             {{\n\
             \"translation\":\"string\",\n\
             \"detectedSourceLang\":\"string | null\",\n\
             \"phonetic\":\"string | null\",\n\
             \"partOfSpeech\":[\"string\"],\n\
             \"definitions\":[\"string\"],\n\
             \"examples\":[{{\"sentence\":\"string\",\"translation\":\"string\"}}]\n\
             }}\n\n\
             文本：{text}",
            source_lang = request.source_lang,
            target_lang = request.target_lang,
            text = request.text
        );

        let content = self
            .send_chat_completion(self.build_translation_system_prompt(), user_prompt)
            .await?;

        parse_translation_result(&content)
    }

    async fn generate_word_detail_inner(
        &self,
        request: WordInsightRequest,
    ) -> Result<GeneratedWordDetail, TranslationError> {
        validate_text(&request.word)?;

        let user_prompt = format!(
            "单词：{word}\n译文：{translation}\n来源语言：{source_lang}\n目标语言：{target_lang}",
            word = request.word,
            translation = request.translation,
            source_lang = request.source_lang,
            target_lang = request.target_lang
        );

        let content = self
            .send_chat_completion(self.build_word_detail_system_prompt(), user_prompt)
            .await?;

        parse_word_detail(&content).map_err(|_| TranslationError::InvalidJson)
    }

    async fn send_chat_completion(
        &self,
        system_prompt: String,
        user_prompt: String,
    ) -> Result<String, TranslationError> {
        let payload = ChatCompletionRequest {
            model: self.config.translation_model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            response_format: ResponseFormat {
                kind: "json_object".to_string(),
            },
        };

        let response = self
            .client
            .post(self.chat_completions_url())
            .bearer_auth(self.config.api_key.trim())
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
            .json::<ChatCompletionResponse>()
            .await
            .map_err(map_reqwest_error)?;

        body.choices
            .into_iter()
            .next()
            .map(|choice| {
                debug!("LLM raw response content: {}", choice.message.content);
                choice.message.content
            })
            .ok_or(TranslationError::InvalidResponse)
    }

    fn chat_completions_url(&self) -> String {
        build_chat_completions_url(&self.config.api_endpoint)
    }

    fn build_translation_system_prompt(&self) -> String {
        append_custom_prompt(SYSTEM_TRANSLATION_PROMPT, &self.config.translation_prompt)
    }

    fn build_word_detail_system_prompt(&self) -> String {
        append_custom_prompt(WORD_DETAIL_PROMPT, &self.config.word_detail_prompt)
    }
}

impl TranslationProvider for CustomTranslationProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

impl WordInsightProvider for CustomTranslationProvider {
    fn generate_word_detail<'a>(&'a self, request: WordInsightRequest) -> WordInsightFuture<'a> {
        Box::pin(async move { self.generate_word_detail_inner(request).await })
    }
}

fn build_chat_completions_url(endpoint: &str) -> String {
    let endpoint = normalize_endpoint(endpoint);
    if endpoint.ends_with("/chat/completions") {
        endpoint
    } else {
        format!("{endpoint}/chat/completions")
    }
}

fn append_custom_prompt(base: &str, custom: &str) -> String {
    let custom = custom.trim();
    if custom.is_empty() {
        base.trim().to_string()
    } else {
        format!("{}\n\n{}", base.trim(), custom)
    }
}

fn parse_translation_result(content: &str) -> Result<TranslationResult, TranslationError> {
    let json_str = extract_json(content);
    let parsed = serde_json::from_str::<TranslationResponse>(json_str)
        .map_err(|_| TranslationError::InvalidJson)?;
    Ok(TranslationResult {
        translation: parsed.translation,
        detected_source_lang: parsed.detected_source_lang,
        phonetic: parsed.phonetic,
        part_of_speech: parsed.part_of_speech,
        definitions: parsed.definitions,
        examples: parsed.examples,
    })
}

fn parse_word_detail(content: &str) -> Result<GeneratedWordDetail, TranslationError> {
    let json_str = extract_json(content);
    let parsed = serde_json::from_str::<WordDetailResponse>(json_str)
        .map_err(|_| TranslationError::InvalidJson)?;
    if parsed.translation.trim().is_empty()
        || parsed.memory_tip.trim().is_empty()
        || parsed.detail.trim().is_empty()
        || parsed.definitions.is_empty()
        || parsed.examples.is_empty()
    {
        return Err(TranslationError::InvalidJson);
    }

    Ok(GeneratedWordDetail {
        translation: parsed.translation,
        phonetic: parsed.phonetic,
        part_of_speech: parsed.part_of_speech,
        definitions: parsed.definitions,
        examples: parsed.examples,
        memory_tip: parsed.memory_tip,
        detail: parsed.detail,
    })
}

/// Extract JSON from LLM response content.
/// Handles: bare JSON, markdown-wrapped JSON (```json...```), and JSON embedded in reasoning text.
fn extract_json(content: &str) -> &str {
    let trimmed = content.trim();

    // Case 1: already valid JSON (starts with '{' or '[')
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return trimmed;
    }

    // Case 2: JSON wrapped in markdown code blocks — ```json ... ``` or ``` ... ```
    if let Some(json) = extract_from_markdown_code_block(trimmed) {
        return json;
    }

    // Case 3: JSON embedded in thinking/reasoning text — find first '{' to last '}'
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if start < end {
                return &trimmed[start..=end];
            }
        }
    }

    // Fallback: return original content (serde_json::from_str will fail)
    content
}

fn extract_from_markdown_code_block(content: &str) -> Option<&str> {
    // Match ```json ... ``` or ``` ... ``` patterns
    let start_marker = content.find("```");
    let end_marker = content.rfind("```");

    if start_marker.is_none() || end_marker.is_none() || start_marker == end_marker {
        return None;
    }

    let start = start_marker.unwrap();
    let end = end_marker.unwrap();

    // Skip the ``` and optional language tag (e.g., "json", "JSON")
    let block_start = content[start + 3..].find('\n').map_or(start + 3, |nl| start + 3 + nl + 1);
    let block_end = end;

    if block_start >= block_end {
        return None;
    }

    Some(content[block_start..block_end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> TranslationConfig {
        TranslationConfig {
            engine: "custom".to_string(),
            api_endpoint: "https://api.example.com/v1".to_string(),
            api_key: "test-key".to_string(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: "test-model".to_string(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn chat_completions_url_appends_path_when_needed() {
        assert_eq!(
            build_chat_completions_url(" https://api.example.com/v1/ "),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn chat_completions_url_does_not_duplicate_path() {
        assert_eq!(
            build_chat_completions_url("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn new_rejects_missing_api_key() {
        let mut config = valid_config();
        config.api_key = "  ".to_string();

        let result = CustomTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingApiKey)));
    }

    #[test]
    fn prompt_builders_append_custom_prompt() {
        let mut config = valid_config();
        config.translation_prompt = "保持术语一致。".to_string();
        config.word_detail_prompt = "偏向商务语境。".to_string();
        let provider = CustomTranslationProvider::new(config).unwrap();

        assert!(provider
            .build_translation_system_prompt()
            .contains(SYSTEM_TRANSLATION_PROMPT.trim()));
        assert!(provider
            .build_translation_system_prompt()
            .contains("保持术语一致。"));
        assert!(provider
            .build_word_detail_system_prompt()
            .contains(WORD_DETAIL_PROMPT.trim()));
        assert!(provider
            .build_word_detail_system_prompt()
            .contains("偏向商务语境。"));
    }

    #[test]
    fn parse_word_detail_accepts_camel_case_fields() {
        let content = r#"{
            "translation": "苹果",
            "phonetic": "ˈæpəl",
            "partOfSpeech": ["noun"],
            "definitions": ["一种水果"],
            "examples": [
                {"sentence": "I ate an apple.", "translation": "我吃了一个苹果。"}
            ],
            "memoryTip": "apple 可以联想到苹果。",
            "detail": "常用作可数名词。"
        }"#;

        let detail = parse_word_detail(content).unwrap();

        assert_eq!(detail.translation, "苹果");
        assert_eq!(detail.part_of_speech, vec!["noun"]);
        assert_eq!(detail.memory_tip, "apple 可以联想到苹果。");
    }

    #[test]
    fn parse_translation_result_accepts_valid_json() {
        let content = r#"{
            "translation": "你好",
            "detectedSourceLang": "en",
            "phonetic": null,
            "partOfSpeech": ["interjection"],
            "definitions": ["问候语"],
            "examples": [
                {"sentence": "Hello there.", "translation": "你好。"}
            ]
        }"#;

        let result = parse_translation_result(content).unwrap();

        assert_eq!(result.translation, "你好");
        assert_eq!(result.detected_source_lang, Some("en".to_string()));
        assert_eq!(result.part_of_speech, vec!["interjection"]);
        assert_eq!(result.examples[0].sentence, "Hello there.");
    }

    #[test]
    fn parse_translation_result_rejects_invalid_json() {
        let result = parse_translation_result("plain translated text");

        assert!(matches!(result, Err(TranslationError::InvalidJson)));
    }

    #[test]
    fn parse_word_detail_rejects_empty_required_semantic_fields() {
        let content = r#"{
            "translation": " ",
            "phonetic": null,
            "partOfSpeech": ["noun"],
            "definitions": ["一种水果"],
            "examples": [
                {"sentence": "I ate an apple.", "translation": "我吃了一个苹果。"}
            ],
            "memoryTip": "apple 可以联想到苹果。",
            "detail": "常用作可数名词。"
        }"#;

        let result = parse_word_detail(content);

        assert!(matches!(result, Err(TranslationError::InvalidJson)));
    }

    #[test]
    fn parse_word_detail_rejects_empty_memory_tip() {
        let content = r#"{
            "translation": "苹果",
            "phonetic": null,
            "partOfSpeech": ["noun"],
            "definitions": ["一种水果"],
            "examples": [
                {"sentence": "I ate an apple.", "translation": "我吃了一个苹果。"}
            ],
            "memoryTip": " ",
            "detail": "常用作可数名词。"
        }"#;

        let result = parse_word_detail(content);

        assert!(matches!(result, Err(TranslationError::InvalidJson)));
    }

    #[test]
    fn parse_word_detail_rejects_empty_detail() {
        let content = r#"{
            "translation": "苹果",
            "phonetic": null,
            "partOfSpeech": ["noun"],
            "definitions": ["一种水果"],
            "examples": [
                {"sentence": "I ate an apple.", "translation": "我吃了一个苹果。"}
            ],
            "memoryTip": "apple 可以联想到苹果。",
            "detail": " "
        }"#;

        let result = parse_word_detail(content);

        assert!(matches!(result, Err(TranslationError::InvalidJson)));
    }

    #[test]
    fn parse_word_detail_rejects_empty_definitions_and_examples() {
        let content = r#"{
            "translation": "苹果",
            "phonetic": null,
            "partOfSpeech": ["noun"],
            "definitions": [],
            "examples": [],
            "memoryTip": "apple 可以联想到苹果。",
            "detail": "常用作可数名词。"
        }"#;

        let result = parse_word_detail(content);

        assert!(matches!(result, Err(TranslationError::InvalidJson)));
    }

    #[test]
    fn http_error_message_uses_custom_error_fields() {
        let body = r#"{"error":{"message":"model not found","type":"invalid_request_error","code":"model_not_found"}}"#;

        assert_eq!(
            format_http_error(reqwest::StatusCode::BAD_REQUEST, body, &["test-key"]),
            "HTTP 400 Bad Request: model not found (type: invalid_request_error, code: model_not_found)"
        );
    }

    #[test]
    fn http_error_message_redacts_api_key_before_formatting() {
        let body = r#"{"error":{"message":"invalid key sk-test-secret","type":"auth","code":"sk-test-secret"}}"#;

        assert_eq!(
            format_http_error(reqwest::StatusCode::UNAUTHORIZED, body, &["sk-test-secret"]),
            "HTTP 401 Unauthorized: invalid key [redacted] (type: auth, code: [redacted])"
        );
    }
}
