use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

pub type TranslationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<TranslationResult, TranslationError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone)]
pub struct TranslationConfig {
    pub engine: String,
    pub api_endpoint: String,
    pub api_key: String,
    pub api_secret: String,
    pub api_region: String,
    pub translation_model: String,
    pub translation_prompt: String,
    pub word_detail_prompt: String,
    pub timeout_ms: u64,
}

pub const DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT: &str = "http://localhost:5005";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEngineConfig {
    pub libretranslate_endpoint: String,
}

impl LocalEngineConfig {
    pub fn default_local() -> Self {
        Self {
            libretranslate_endpoint: DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibreTranslateLanguage {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LibreTranslateLanguages {
    #[serde(rename = "sourceLanguages")]
    pub source_languages: Vec<LibreTranslateLanguage>,
    #[serde(rename = "targetLanguages")]
    pub target_languages: Vec<LibreTranslateLanguage>,
}

pub fn is_supported_source_language(languages: &LibreTranslateLanguages, lang: &str) -> bool {
    if lang.trim().eq_ignore_ascii_case("auto") {
        return true;
    }
    languages
        .source_languages
        .iter()
        .any(|language| language.code == lang)
}

pub fn is_supported_target_language(languages: &LibreTranslateLanguages, lang: &str) -> bool {
    languages
        .target_languages
        .iter()
        .any(|language| language.code == lang)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationExample {
    pub sentence: String,
    pub translation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationResult {
    pub translation: String,
    pub detected_source_lang: Option<String>,
    pub phonetic: Option<String>,
    pub part_of_speech: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<TranslationExample>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TranslationError {
    #[error("翻译文本不能为空")]
    EmptyText,
    #[error("请先在设置页填写 API 密钥")]
    MissingApiKey,
    #[error("请先在设置页填写 API Secret")]
    MissingApiSecret,
    #[error("请先在设置页填写服务区域")]
    MissingRegion,
    #[error("请先在设置页填写 API 地址")]
    MissingEndpoint,
    #[error("请先填写模型名称")]
    MissingModel,
    #[error("当前翻译引擎暂未完整支持：{0}")]
    UnsupportedEngine(String),
    #[error("翻译服务请求超时，请稍后重试")]
    RequestTimeout,
    #[error("翻译服务请求失败：{0}")]
    RequestFailed(String),
    #[error("翻译服务返回格式异常")]
    InvalidResponse,
    #[error("单词详情返回格式异常")]
    InvalidJson,
    #[error("单词不存在")]
    WordNotFound,
    #[error("当前翻译引擎不支持该语言：{0}")]
    UnsupportedLanguage(String),
}

pub trait TranslationProvider: Send + Sync {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a>;
}
