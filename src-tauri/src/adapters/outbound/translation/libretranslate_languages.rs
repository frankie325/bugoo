use crate::ports::outbound::translation::LibreTranslateLanguages;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LibreTranslateLanguagesError {
    #[error("LibreTranslate 语言配置读取失败：{0}")]
    ReadFailed(String),
    #[error("LibreTranslate 语言配置格式异常：{0}")]
    InvalidJson(String),
    #[error("LibreTranslate 语言配置不能为空")]
    EmptyLanguages,
}

pub fn read_libretranslate_languages(
    path: &Path,
) -> Result<LibreTranslateLanguages, LibreTranslateLanguagesError> {
    let content = fs::read_to_string(path)
        .map_err(|error| LibreTranslateLanguagesError::ReadFailed(error.to_string()))?;
    let languages = serde_json::from_str::<LibreTranslateLanguages>(&content)
        .map_err(|error| LibreTranslateLanguagesError::InvalidJson(error.to_string()))?;

    if languages.source_languages.is_empty() || languages.target_languages.is_empty() {
        return Err(LibreTranslateLanguagesError::EmptyLanguages);
    }

    Ok(languages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::outbound::translation::{
        is_supported_source_language, is_supported_target_language, normalize_language_code,
        LibreTranslateLanguage,
    };

    #[test]
    fn normalize_language_code_maps_simplified_chinese() {
        assert_eq!(normalize_language_code("zh-CN"), "zh");
    }

    #[test]
    fn normalize_language_code_maps_traditional_chinese() {
        assert_eq!(normalize_language_code("zh-TW"), "zt");
    }

    #[test]
    fn read_libretranslate_languages_returns_source_and_target_languages() {
        let dir = std::env::temp_dir().join(format!("bugoo-languages-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("libretranslate-languages.json");
        std::fs::write(
            &path,
            r#"{
              "sourceLanguages":[{"code":"auto","name":"Auto Detect"},{"code":"en","name":"English"}],
              "targetLanguages":[{"code":"zh","name":"Chinese"}]
            }"#,
        )
        .unwrap();

        let languages = read_libretranslate_languages(&path).unwrap();

        assert_eq!(languages.source_languages[0].code, "auto");
    }

    #[test]
    fn is_supported_source_language_accepts_auto() {
        let languages = LibreTranslateLanguages {
            source_languages: vec![LibreTranslateLanguage {
                code: "en".to_string(),
                name: "English".to_string(),
            }],
            target_languages: vec![LibreTranslateLanguage {
                code: "zh".to_string(),
                name: "Chinese".to_string(),
            }],
        };

        assert!(is_supported_source_language(&languages, "auto"));
    }

    #[test]
    fn is_supported_target_language_rejects_auto() {
        let languages = LibreTranslateLanguages {
            source_languages: vec![LibreTranslateLanguage {
                code: "auto".to_string(),
                name: "Auto Detect".to_string(),
            }],
            target_languages: vec![LibreTranslateLanguage {
                code: "zh".to_string(),
                name: "Chinese".to_string(),
            }],
        };

        assert!(!is_supported_target_language(&languages, "auto"));
    }
}
