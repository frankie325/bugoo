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

pub const DEFAULT_ENDPOINT_LOCAL: &str = "http://localhost:5005";
pub const DEFAULT_ENDPOINT_BAIDU: &str = "https://fanyi-api.baidu.com/api/trans/vip/translate";
pub const DEFAULT_ENDPOINT_DEEPL: &str = "https://api-free.deepl.com/v2/translate";
pub const DEFAULT_ENDPOINT_GOOGLE: &str = "https://translation.googleapis.com/language/translate/v2";
pub const DEFAULT_ENDPOINT_MICROSOFT: &str = "https://api.cognitive.microsofttranslator.com/translate";
pub const DEFAULT_ENDPOINT_TENCENT: &str = "https://tmt.tencentcloud.tencentcloudapi.com/api/trans/v3";
pub const DEFAULT_ENDPOINT_YOUDAO: &str = "https://openapi.youdao.com/api";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineEndpoints {
    pub local: String,
    pub baidu: String,
    pub deepl: String,
    pub google: String,
    pub microsoft: String,
    pub tencent: String,
    pub youdao: String,
    pub custom: String,
}

impl Default for EngineEndpoints {
    fn default() -> Self {
        Self {
            local: DEFAULT_ENDPOINT_LOCAL.to_string(),
            baidu: DEFAULT_ENDPOINT_BAIDU.to_string(),
            deepl: DEFAULT_ENDPOINT_DEEPL.to_string(),
            google: DEFAULT_ENDPOINT_GOOGLE.to_string(),
            microsoft: DEFAULT_ENDPOINT_MICROSOFT.to_string(),
            tencent: DEFAULT_ENDPOINT_TENCENT.to_string(),
            youdao: DEFAULT_ENDPOINT_YOUDAO.to_string(),
            custom: String::new(),
        }
    }
}

impl EngineEndpoints {
    pub fn endpoint_for(&self, engine: &str) -> Option<String> {
        let endpoint = match engine {
            "local" => &self.local,
            "baidu" => &self.baidu,
            "deepl" => &self.deepl,
            "google" => &self.google,
            "microsoft" => &self.microsoft,
            "tencent" => &self.tencent,
            "youdao" => &self.youdao,
            "custom" => &self.custom,
            _ => return None,
        };
        if endpoint.trim().is_empty() {
            None
        } else {
            Some(endpoint.clone())
        }
    }

    pub fn endpoint_or_default(&self, engine: &str) -> String {
        self.endpoint_for(engine)
            .unwrap_or_else(|| EngineEndpoints::default().endpoint_for(engine).unwrap_or_default())
    }
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