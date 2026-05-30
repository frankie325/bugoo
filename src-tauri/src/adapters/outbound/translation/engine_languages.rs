use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineLanguagesError {
    #[error("语言配置读取失败：{0}")]
    ReadFailed(String),
    #[error("语言配置格式异常：{0}")]
    InvalidJson(String),
    #[error("语言配置不能为空")]
    EmptyLanguages,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineLanguage {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineLanguages {
    #[serde(rename = "sourceLanguages")]
    pub source_languages: Vec<EngineLanguage>,
    #[serde(rename = "targetLanguages")]
    pub target_languages: Vec<EngineLanguage>,
}

pub fn read_engine_languages(path: &Path) -> Result<EngineLanguages, EngineLanguagesError> {
    let content =
        fs::read_to_string(path).map_err(|e| EngineLanguagesError::ReadFailed(e.to_string()))?;
    let languages = serde_json::from_str::<EngineLanguages>(&content)
        .map_err(|e| EngineLanguagesError::InvalidJson(e.to_string()))?;
    if languages.source_languages.is_empty() || languages.target_languages.is_empty() {
        return Err(EngineLanguagesError::EmptyLanguages);
    }
    Ok(languages)
}

pub fn is_engine_source_language_supported(languages: &EngineLanguages, lang: &str) -> bool {
    if lang.trim().eq_ignore_ascii_case("auto") {
        return true;
    }
    let lang_lower = lang.trim().to_lowercase();
    languages
        .source_languages
        .iter()
        .any(|language| language.code.to_lowercase() == lang_lower)
}

pub fn is_engine_target_language_supported(languages: &EngineLanguages, lang: &str) -> bool {
    let lang_lower = lang.trim().to_lowercase();
    languages
        .target_languages
        .iter()
        .any(|language| language.code.to_lowercase() == lang_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_engine_languages_returns_source_and_target_languages() {
        let dir =
            std::env::temp_dir().join(format!("bugoo-languages-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test-languages.json");
        std::fs::write(
            &path,
            r#"{
              "sourceLanguages":[{"code":"auto","name":"Auto Detect"},{"code":"en","name":"English"}],
              "targetLanguages":[{"code":"zh","name":"Chinese"}]
            }"#,
        )
        .unwrap();

        let languages = read_engine_languages(&path).unwrap();

        assert_eq!(languages.source_languages[0].code, "auto");
    }

    #[test]
    fn is_engine_source_language_supported_accepts_auto() {
        let languages = EngineLanguages {
            source_languages: vec![EngineLanguage {
                code: "en".to_string(),
                name: "English".to_string(),
            }],
            target_languages: vec![EngineLanguage {
                code: "zh".to_string(),
                name: "Chinese".to_string(),
            }],
        };

        assert!(is_engine_source_language_supported(&languages, "auto"));
    }

    #[test]
    fn is_engine_target_language_supported_rejects_auto() {
        let languages = EngineLanguages {
            source_languages: vec![EngineLanguage {
                code: "auto".to_string(),
                name: "Auto Detect".to_string(),
            }],
            target_languages: vec![EngineLanguage {
                code: "zh".to_string(),
                name: "Chinese".to_string(),
            }],
        };

        assert!(!is_engine_target_language_supported(&languages, "auto"));
    }
}