use crate::ports::outbound::translation::{
    LocalEngineConfig, DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT,
};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct LocalEngineConfigFile {
    #[serde(rename = "libretranslateEndpoint")]
    libretranslate_endpoint: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LocalEngineConfigError {
    #[error("本地翻译配置读取失败：{0}")]
    ReadFailed(String),
    #[error("本地翻译配置格式异常：{0}")]
    InvalidJson(String),
}

pub fn read_local_engine_config(path: &Path) -> Result<LocalEngineConfig, LocalEngineConfigError> {
    let content = fs::read_to_string(path)
        .map_err(|error| LocalEngineConfigError::ReadFailed(error.to_string()))?;
    let parsed = serde_json::from_str::<LocalEngineConfigFile>(&content)
        .map_err(|error| LocalEngineConfigError::InvalidJson(error.to_string()))?;
    let endpoint = parsed
        .libretranslate_endpoint
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT.to_string());

    Ok(LocalEngineConfig {
        libretranslate_endpoint: endpoint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_local_engine_config_returns_endpoint_from_json() {
        let dir = std::env::temp_dir().join(format!("bugoo-local-config-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("local-engine.json");
        std::fs::write(
            &path,
            r#"{"libretranslateEndpoint":"http://localhost:5005"}"#,
        )
        .unwrap();

        let config = read_local_engine_config(&path).unwrap();

        assert_eq!(config.libretranslate_endpoint, "http://localhost:5005");
    }

    #[test]
    fn read_local_engine_config_uses_default_when_endpoint_is_blank() {
        let dir = std::env::temp_dir().join(format!("bugoo-local-config-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("local-engine.json");
        std::fs::write(&path, r#"{"libretranslateEndpoint":"  "}"#).unwrap();

        let config = read_local_engine_config(&path).unwrap();

        assert_eq!(
            config.libretranslate_endpoint,
            DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT
        );
    }
}
