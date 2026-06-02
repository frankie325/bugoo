use crate::ports::outbound::translation::EngineEndpoints;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct EngineEndpointsFile {
    #[serde(rename = "local")]
    local: Option<String>,
    #[serde(rename = "baidu")]
    baidu: Option<String>,
    #[serde(rename = "deepl")]
    deepl: Option<String>,
    #[serde(rename = "google")]
    google: Option<String>,
    #[serde(rename = "microsoft")]
    microsoft: Option<String>,
    #[serde(rename = "tencent")]
    tencent: Option<String>,
    #[serde(rename = "youdao")]
    youdao: Option<String>,
    #[serde(rename = "custom")]
    custom: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EngineEndpointsError {
    #[error("引擎端点配置读取失败：{0}")]
    ReadFailed(String),
    #[error("引擎端点配置格式异常：{0}")]
    InvalidJson(String),
}

pub fn read_engine_endpoints(path: &Path) -> Result<EngineEndpoints, EngineEndpointsError> {
    let content = fs::read_to_string(path)
        .map_err(|error| EngineEndpointsError::ReadFailed(error.to_string()))?;
    let parsed = serde_json::from_str::<EngineEndpointsFile>(&content)
        .map_err(|error| EngineEndpointsError::InvalidJson(error.to_string()))?;

    Ok(EngineEndpoints {
        local: parsed
            .local
            .unwrap_or_else(|| "http://localhost:5005".to_string()),
        baidu: parsed
            .baidu
            .unwrap_or_else(|| "https://fanyi-api.baidu.com/api/trans/vip/translate".to_string()),
        deepl: parsed
            .deepl
            .unwrap_or_else(|| "https://api-free.deepl.com/v2/translate".to_string()),
        google: parsed.google.unwrap_or_else(|| {
            "https://translation.googleapis.com/language/translate/v2".to_string()
        }),
        microsoft: parsed.microsoft.unwrap_or_else(|| {
            "https://api.cognitive.microsofttranslator.com/translate".to_string()
        }),
        tencent: parsed.tencent.unwrap_or_else(|| {
            "https://tmt.tencentcloud.tencentcloudapi.com/api/trans/v3".to_string()
        }),
        youdao: parsed
            .youdao
            .unwrap_or_else(|| "https://openapi.youdao.com/api".to_string()),
        custom: parsed.custom.unwrap_or_default(),
    })
}
