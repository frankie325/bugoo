# 本地翻译路由实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将翻译引擎的系统内置选项持久化为 `local`，新增源语言/目标语言设置，并让本地引擎按“词典优先，LibreTranslate 兜底，语言不支持则明确报错”的规则工作。

**Architecture:** 后端保持六边形边界：Tauri 装配层读取 bundled resource 配置，领域服务只接收配置值和端口能力；词典端口声明自身支持的语言对；本地路由在 `TranslationService` 内完成，第三方和自定义引擎绕过词典与本地 LibreTranslate。前端设置页只保存 `translationEngine/sourceLanguage/targetLanguage`，本地 LibreTranslate endpoint 和可选语言列表固定来自 `src-tauri/resources/translation/*.json`。

**Tech Stack:** Tauri 2.x Rust、serde/serde_json、thiserror、React 19、HeroUI v3 Select、Zustand、Vitest、Cargo test。

---

## 文件结构

- Create: `src-tauri/resources/translation/local-engine.json`
  - 记录本地 LibreTranslate 服务地址，默认 `http://localhost:5005`。
- Create: `src-tauri/resources/translation/libretranslate-languages.json`
  - 记录从 LibreTranslate 官方支持语言整理出的可选源语言/目标语言列表。
- Create: `src-tauri/src/adapters/outbound/config/mod.rs`
  - 暴露配置 adapter 模块。
- Create: `src-tauri/src/adapters/outbound/config/local_engine.rs`
  - 从 Tauri resource 解析本地引擎配置文件。
- Create: `src-tauri/src/adapters/outbound/translation/libretranslate_languages.rs`
  - 读取 LibreTranslate 语言 resource 配置，并提供语言支持校验。
- Create: `src-tauri/src/ports/outbound/language_detection.rs`
  - 定义语言检测端口，隔离领域服务和第三方检测库。
- Create: `src-tauri/src/adapters/outbound/language_detection/mod.rs`
  - 暴露语言检测 adapter 模块。
- Create: `src-tauri/src/adapters/outbound/language_detection/whichlang_detector.rs`
  - 使用 `whichlang` crate 做轻量语言检测，并映射到 LibreTranslate 语言代码。
- Create: `src-tauri/src/commands/translation_languages.rs`
  - 给设置页暴露 LibreTranslate 支持语言列表。
- Create: `src/lib/api/translationLanguages.ts`
  - 前端封装 `get_translation_languages`。
- Create: `src/pages/Settings/panels/translationSettingsModel.ts`
  - 前端设置页的纯逻辑：引擎归一化、字段可见性、语言列表 fallback。
- Create: `src/pages/Settings/panels/__test__/translationSettingsModel.test.ts`
  - 前端设置逻辑测试，符合项目“测试放功能目录 `__test__`”规则。
- Modify: `src-tauri/tauri.conf.json`
  - 按 Tauri 官方 resources 方式打包 `resources/translation/` 和 `resources/stardict-ecdict/` 目录，避免缺失词典资源导致冷构建失败。
- Modify: `src-tauri/src/adapters/outbound/mod.rs`
  - 暴露 `config` adapter。
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`
  - 暴露 `libretranslate_languages`。
- Modify: `src-tauri/src/lib.rs`
  - 在 setup 中解析本地引擎配置和 LibreTranslate 语言配置，装配 `WhichlangLanguageDetector`，并传入 `TranslationService`。
- Modify: `src-tauri/Cargo.toml`
  - 添加 `whichlang` 依赖。
- Modify: `src-tauri/Cargo.lock`
  - 由 `cargo` 更新。
- Modify: `src-tauri/src/ports/outbound/mod.rs`
  - 暴露 language detection 端口。
- Modify: `src-tauri/src/commands/mod.rs`
  - 暴露 translation language 命令模块。
- Modify: `src/lib/api/index.ts`
  - 导出 translation language API。
- Modify: `src-tauri/src/ports/outbound/dictionary.rs`
  - 给 `DictionaryProvider` 增加 `supports_language_pair`。
- Modify: `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`
  - 声明 ECDICT 支持 `en -> zh-CN/zh-TW/zh/zt`。
- Modify: `src-tauri/src/ports/outbound/translation.rs`
  - 增加 `UnsupportedLanguage` 错误。
- Modify: `src-tauri/src/domain/services/mod.rs`
  - 暴露 `language_detection`。
- Modify: `src-tauri/src/domain/services/translation_service.rs`
  - 实现 `local` 路由、源/目标语言设置、第三方绕过词典。
- Modify: `src-tauri/src/commands/settings.rs`
  - 默认 `translationEngine=local`、`sourceLanguage=auto`、`targetLanguage=zh`，目标语言使用 LibreTranslate 配置中的语言代码。
- Modify: `src/pages/Settings/panels/TranslationPanel.tsx`
  - 系统内置选项显示为“本地”，新增源语言/目标语言 Select，移除 local 的 endpoint/key 字段。
- Modify: `src/locales/*/common.json`
  - 添加设置页文案键。`zh-CN` 使用中文，`zh-TW` 使用繁中，其余语言使用英文 fallback。

## 背景约束

- 当前 worktree：`/Users/frank/code/mine/bugoo/.worktrees/local-translation-routing`。
- 当前 baseline：`pnpm install --frozen-lockfile` 成功；`cargo build` 失败，因为 `tauri.conf.json` 引用的 ECDICT 三件套在 worktree 中不存在。Task 1 会将资源配置改为目录打包，让缺失词典时构建不失败，运行时仍由现有 dictionary unavailable 日志降级。
- Tauri v2 官方建议 bundled resources 通过 `tauri.conf.json > bundle.resources` 声明，运行时用 `app.path().resolve(..., BaseDirectory::Resource)` 解析。参考：https://v2.tauri.app/develop/resources/
- LibreTranslate 支持语言数据来自官方文档：https://docs.libretranslate.com/guides/supported_languages/ 。设置页只展示该 resource 配置中的语言，后端也使用同一份配置做 `local` 引擎语言支持校验。

---

### Task 1: Tauri 资源配置和本地翻译配置文件

**Files:**
- Create: `src-tauri/resources/translation/local-engine.json`
- Create: `src-tauri/resources/translation/libretranslate-languages.json`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 写入本地引擎配置文件**

Create `src-tauri/resources/translation/local-engine.json`:

```json
{
  "libretranslateEndpoint": "http://localhost:5005"
}
```

- [ ] **Step 2: 写入 LibreTranslate 支持语言配置文件**

Create `src-tauri/resources/translation/libretranslate-languages.json`:

```json
{
  "sourceLanguages": [
    { "code": "auto", "name": "Auto Detect" },
    { "code": "sq", "name": "Albanian" },
    { "code": "ar", "name": "Arabic" },
    { "code": "az", "name": "Azerbaijani" },
    { "code": "eu", "name": "Basque" },
    { "code": "bn", "name": "Bengali" },
    { "code": "bg", "name": "Bulgarian" },
    { "code": "ca", "name": "Catalan" },
    { "code": "zh", "name": "Chinese" },
    { "code": "zt", "name": "Chinese Traditional" },
    { "code": "cs", "name": "Czech" },
    { "code": "da", "name": "Danish" },
    { "code": "nl", "name": "Dutch" },
    { "code": "en", "name": "English" },
    { "code": "eo", "name": "Esperanto" },
    { "code": "et", "name": "Estonian" },
    { "code": "fi", "name": "Finnish" },
    { "code": "fr", "name": "French" },
    { "code": "gl", "name": "Galician" },
    { "code": "de", "name": "German" },
    { "code": "el", "name": "Greek" },
    { "code": "he", "name": "Hebrew" },
    { "code": "hi", "name": "Hindi" },
    { "code": "hu", "name": "Hungarian" },
    { "code": "id", "name": "Indonesian" },
    { "code": "ga", "name": "Irish" },
    { "code": "it", "name": "Italian" },
    { "code": "ja", "name": "Japanese" },
    { "code": "ko", "name": "Korean" },
    { "code": "ky", "name": "Kyrgyz" },
    { "code": "lv", "name": "Latvian" },
    { "code": "lt", "name": "Lithuanian" },
    { "code": "ms", "name": "Malay" },
    { "code": "nb", "name": "Norwegian Bokmal" },
    { "code": "fa", "name": "Persian" },
    { "code": "pl", "name": "Polish" },
    { "code": "pt", "name": "Portuguese" },
    { "code": "pb", "name": "Portuguese Brazilian" },
    { "code": "ro", "name": "Romanian" },
    { "code": "ru", "name": "Russian" },
    { "code": "sk", "name": "Slovak" },
    { "code": "sl", "name": "Slovenian" },
    { "code": "es", "name": "Spanish" },
    { "code": "sv", "name": "Swedish" },
    { "code": "tl", "name": "Tagalog" },
    { "code": "th", "name": "Thai" },
    { "code": "tr", "name": "Turkish" },
    { "code": "uk", "name": "Ukrainian" },
    { "code": "ur", "name": "Urdu" },
    { "code": "vi", "name": "Vietnamese" }
  ],
  "targetLanguages": [
    { "code": "sq", "name": "Albanian" },
    { "code": "ar", "name": "Arabic" },
    { "code": "az", "name": "Azerbaijani" },
    { "code": "eu", "name": "Basque" },
    { "code": "bn", "name": "Bengali" },
    { "code": "bg", "name": "Bulgarian" },
    { "code": "ca", "name": "Catalan" },
    { "code": "zh", "name": "Chinese" },
    { "code": "zt", "name": "Chinese Traditional" },
    { "code": "cs", "name": "Czech" },
    { "code": "da", "name": "Danish" },
    { "code": "nl", "name": "Dutch" },
    { "code": "en", "name": "English" },
    { "code": "eo", "name": "Esperanto" },
    { "code": "et", "name": "Estonian" },
    { "code": "fi", "name": "Finnish" },
    { "code": "fr", "name": "French" },
    { "code": "gl", "name": "Galician" },
    { "code": "de", "name": "German" },
    { "code": "el", "name": "Greek" },
    { "code": "he", "name": "Hebrew" },
    { "code": "hi", "name": "Hindi" },
    { "code": "hu", "name": "Hungarian" },
    { "code": "id", "name": "Indonesian" },
    { "code": "ga", "name": "Irish" },
    { "code": "it", "name": "Italian" },
    { "code": "ja", "name": "Japanese" },
    { "code": "ko", "name": "Korean" },
    { "code": "ky", "name": "Kyrgyz" },
    { "code": "lv", "name": "Latvian" },
    { "code": "lt", "name": "Lithuanian" },
    { "code": "ms", "name": "Malay" },
    { "code": "nb", "name": "Norwegian Bokmal" },
    { "code": "fa", "name": "Persian" },
    { "code": "pl", "name": "Polish" },
    { "code": "pt", "name": "Portuguese" },
    { "code": "pb", "name": "Portuguese Brazilian" },
    { "code": "ro", "name": "Romanian" },
    { "code": "ru", "name": "Russian" },
    { "code": "sk", "name": "Slovak" },
    { "code": "sl", "name": "Slovenian" },
    { "code": "es", "name": "Spanish" },
    { "code": "sv", "name": "Swedish" },
    { "code": "tl", "name": "Tagalog" },
    { "code": "th", "name": "Thai" },
    { "code": "tr", "name": "Turkish" },
    { "code": "uk", "name": "Ukrainian" },
    { "code": "ur", "name": "Urdu" },
    { "code": "vi", "name": "Vietnamese" }
  ]
}
```

`sourceLanguages` 包含 `auto`，`targetLanguages` 不包含 `auto`。两组语言均来自 LibreTranslate 官方支持语言；如果后续官方新增语言，只改这个 JSON。

- [ ] **Step 3: 修改 Tauri resources 配置**

Replace `src-tauri/tauri.conf.json` with:

```json
{
  "productName": "Bugoo",
  "identifier": "com.bugoo",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [{ "title": "Bugoo", "width": 800, "height": 600, "decorations": true }],
    "security": {
      "capabilities": ["default"]
    }
  },
  "bundle": {
    "active": false,
    "resources": [
      "resources/stardict-ecdict/",
      "resources/translation/"
    ]
  }
}
```

- [ ] **Step 4: 验证构建资源缺失问题已解除**

Run:

```bash
cd src-tauri && cargo build
```

Expected: `Finished dev profile` 或进入 Rust 编译错误；不能再出现 `resource path resources/stardict-ecdict/stardict-ecdict-2.4.2.ifo doesn't exist`。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/tauri.conf.json src-tauri/resources/translation/local-engine.json src-tauri/resources/translation/libretranslate-languages.json
git commit -m "chore: configure local translation resources"
```

---

### Task 2: 本地引擎配置 Adapter

**Files:**
- Create: `src-tauri/src/adapters/outbound/config/mod.rs`
- Create: `src-tauri/src/adapters/outbound/config/local_engine.rs`
- Modify: `src-tauri/src/adapters/outbound/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`

- [ ] **Step 1: 新建配置 adapter 模块入口**

Create `src-tauri/src/adapters/outbound/config/mod.rs`:

```rust
pub mod local_engine;
```

- [ ] **Step 2: 实现本地引擎配置读取**

Create `src-tauri/src/adapters/outbound/config/local_engine.rs`:

```rust
use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT: &str = "http://localhost:5005";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEngineConfig {
    pub libretranslate_endpoint: String,
}

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

impl LocalEngineConfig {
    pub fn default_local() -> Self {
        Self {
            libretranslate_endpoint: DEFAULT_LOCAL_LIBRETRANSLATE_ENDPOINT.to_string(),
        }
    }
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
```

- [ ] **Step 3: 暴露 config adapter**

Modify `src-tauri/src/adapters/outbound/mod.rs` so it contains:

```rust
pub mod config;
pub mod dictionary;
pub mod selection_ui;
pub mod translation;
pub mod tts;
```

- [ ] **Step 4: 扩展 TranslationService 构造参数**

In `src-tauri/src/domain/services/translation_service.rs`, add the import:

```rust
use crate::adapters::outbound::config::local_engine::LocalEngineConfig;
```

Change the struct and constructor to:

```rust
#[derive(Clone)]
pub struct TranslationService {
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
    local_engine_config: LocalEngineConfig,
}

impl TranslationService {
    pub fn new(
        dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
        local_engine_config: LocalEngineConfig,
    ) -> Self {
        Self {
            dictionary_provider,
            local_engine_config,
        }
    }
}
```

Update all unit tests in this file to build the service with:

```rust
TranslationService::new(None, LocalEngineConfig::default_local())
```

- [ ] **Step 5: 在 Tauri setup 中读取 resource 配置**

Modify `src-tauri/src/lib.rs` imports:

```rust
use adapters::outbound::config::local_engine::{
    read_local_engine_config, LocalEngineConfig,
};
```

Inside `.setup(|app| { ... })`, after dictionary provider creation and before `TranslationService::new`, add:

```rust
let local_engine_config_path = app
    .path()
    .resolve(
        "resources/translation/local-engine.json",
        tauri::path::BaseDirectory::Resource,
    )
    .unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("translation")
            .join("local-engine.json")
    });

let local_engine_config = match read_local_engine_config(&local_engine_config_path) {
    Ok(config) => config,
    Err(error) => {
        log::warn!(
            "Local translation config unavailable at {:?}, using default endpoint: {}",
            local_engine_config_path,
            error
        );
        LocalEngineConfig::default_local()
    }
};
```

Replace:

```rust
let translation_service = TranslationService::new(dictionary_provider);
```

with:

```rust
let translation_service = TranslationService::new(dictionary_provider, local_engine_config);
```

- [ ] **Step 6: 运行配置 adapter 测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::config::local_engine
```

Expected: both `read_local_engine_config_*` tests pass.

- [ ] **Step 7: 提交**

```bash
git add src-tauri/src/adapters/outbound/config src-tauri/src/adapters/outbound/mod.rs src-tauri/src/lib.rs src-tauri/src/domain/services/translation_service.rs
git commit -m "feat: load local translation config from resources"
```

---

### Task 3: LibreTranslate 语言支持和词典语言对能力

**Files:**
- Create: `src-tauri/src/adapters/outbound/translation/libretranslate_languages.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/ports/outbound/dictionary.rs`
- Modify: `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`
- Modify: `src-tauri/src/ports/outbound/translation.rs`

- [ ] **Step 1: 添加 LibreTranslate 支持语言配置读取器**

Create `src-tauri/src/adapters/outbound/translation/libretranslate_languages.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LibreTranslateLanguagesError {
    #[error("LibreTranslate 语言配置读取失败：{0}")]
    ReadFailed(String),
    #[error("LibreTranslate 语言配置格式异常：{0}")]
    InvalidJson(String),
    #[error("LibreTranslate 语言配置不能为空")]
    EmptyLanguages,
}

pub fn normalize_language_code(lang: &str) -> String {
    match lang.trim().to_lowercase().as_str() {
        "zh-cn" | "zh-hans" => "zh".to_string(),
        "zh-tw" | "zh-hant" => "zt".to_string(),
        value => value.to_string(),
    }
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

pub fn is_supported_source_language(languages: &LibreTranslateLanguages, lang: &str) -> bool {
    if lang.trim().eq_ignore_ascii_case("auto") {
        return true;
    }

    let normalized = normalize_language_code(lang);
    languages
        .source_languages
        .iter()
        .any(|language| language.code == normalized)
}

pub fn is_supported_target_language(languages: &LibreTranslateLanguages, lang: &str) -> bool {
    let normalized = normalize_language_code(lang);
    languages
        .target_languages
        .iter()
        .any(|language| language.code == normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

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
```

- [ ] **Step 2: 暴露语言模块**

Modify `src-tauri/src/adapters/outbound/translation/mod.rs`:

```rust
pub mod baidu;
pub mod custom;
pub mod deepl;
pub mod google;
pub mod http_utils;
pub mod libretranslate;
pub mod libretranslate_languages;
pub mod microsoft;
pub mod tencent;
pub mod youdao;
```

- [ ] **Step 3: 将语言配置注入 TranslationService**

In `src-tauri/src/domain/services/translation_service.rs`, add the import:

```rust
use crate::adapters::outbound::translation::libretranslate_languages::{
    LibreTranslateLanguage, LibreTranslateLanguages,
};
```

Change the struct to:

```rust
#[derive(Clone)]
pub struct TranslationService {
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
    local_engine_config: LocalEngineConfig,
    libretranslate_languages: LibreTranslateLanguages,
}
```

Change the constructor to:

```rust
pub fn new(
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
    local_engine_config: LocalEngineConfig,
    libretranslate_languages: LibreTranslateLanguages,
) -> Self {
    Self {
        dictionary_provider,
        local_engine_config,
        libretranslate_languages,
    }
}
```

Add this test helper inside the test module so later tests do not duplicate language setup:

```rust
fn test_libretranslate_languages() -> LibreTranslateLanguages {
    LibreTranslateLanguages {
        source_languages: vec![
            LibreTranslateLanguage {
                code: "auto".to_string(),
                name: "Auto Detect".to_string(),
            },
            LibreTranslateLanguage {
                code: "en".to_string(),
                name: "English".to_string(),
            },
            LibreTranslateLanguage {
                code: "ja".to_string(),
                name: "Japanese".to_string(),
            },
        ],
        target_languages: vec![
            LibreTranslateLanguage {
                code: "zh".to_string(),
                name: "Chinese".to_string(),
            },
            LibreTranslateLanguage {
                code: "en".to_string(),
                name: "English".to_string(),
            },
        ],
    }
}
```

Update every `TranslationService::new(...)` call in tests to pass `test_libretranslate_languages()`.

- [ ] **Step 4: 在 Tauri setup 中读取语言配置**

In `src-tauri/src/lib.rs`, add imports:

```rust
use adapters::outbound::translation::libretranslate_languages::{
    read_libretranslate_languages, LibreTranslateLanguages,
};
```

After `local_engine_config` is read, add:

```rust
let libretranslate_languages_path = app
    .path()
    .resolve(
        "resources/translation/libretranslate-languages.json",
        tauri::path::BaseDirectory::Resource,
    )
    .unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("translation")
            .join("libretranslate-languages.json")
    });

let libretranslate_languages = match read_libretranslate_languages(&libretranslate_languages_path) {
    Ok(languages) => languages,
    Err(error) => {
        log::warn!(
            "LibreTranslate languages config unavailable at {:?}: {}",
            libretranslate_languages_path,
            error
        );
        LibreTranslateLanguages {
            source_languages: Vec::new(),
            target_languages: Vec::new(),
        }
    }
};
```

Replace:

```rust
let translation_service = TranslationService::new(dictionary_provider, local_engine_config);
```

with:

```rust
let translation_service = TranslationService::new(
    dictionary_provider,
    local_engine_config,
    libretranslate_languages,
);
```

- [ ] **Step 5: 给词典端口增加语言对能力**

Modify `src-tauri/src/ports/outbound/dictionary.rs` trait:

```rust
pub trait DictionaryProvider: Send + Sync {
    fn supports_language_pair(&self, source_lang: &str, target_lang: &str) -> bool;

    fn lookup(
        &self,
        request: DictionaryLookupRequest,
    ) -> Result<Option<DictionaryLookupResult>, DictionaryError>;
}
```

- [ ] **Step 6: 声明 ECDICT 支持的语言对**

In `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`, add helper functions before the `impl DictionaryProvider` block:

```rust
fn normalize_dictionary_lang(lang: &str) -> String {
    lang.trim().to_lowercase()
}

fn is_chinese_target(lang: &str) -> bool {
    matches!(
        normalize_dictionary_lang(lang).as_str(),
        "zh" | "zh-cn" | "zh-tw" | "zh-hans" | "zh-hant" | "zt"
    )
}
```

Add this method inside `impl DictionaryProvider for StarDictEcdictDictionaryProvider`:

```rust
fn supports_language_pair(&self, source_lang: &str, target_lang: &str) -> bool {
    normalize_dictionary_lang(source_lang) == "en" && is_chinese_target(target_lang)
}
```

Add tests in the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn supports_language_pair_accepts_english_to_simplified_chinese() {
    let dir = create_test_dictionary();
    let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

    assert!(provider.supports_language_pair("en", "zh-CN"));
}

#[test]
fn supports_language_pair_rejects_non_english_source() {
    let dir = create_test_dictionary();
    let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

    assert!(!provider.supports_language_pair("ja", "zh-CN"));
}

#[test]
fn supports_language_pair_rejects_non_chinese_target() {
    let dir = create_test_dictionary();
    let provider = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

    assert!(!provider.supports_language_pair("en", "ja"));
}
```

- [ ] **Step 7: 增加不支持语言错误**

Modify `src-tauri/src/ports/outbound/translation.rs` enum:

```rust
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
    #[error("当前翻译引擎不支持该语言：{0}")]
    UnsupportedLanguage(String),
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
}
```

- [ ] **Step 8: 运行语言和词典测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::libretranslate_languages
cd src-tauri && cargo test adapters::outbound::dictionary::stardict_ecdict
cd src-tauri && cargo test ports::outbound::dictionary
```

Expected: language support and dictionary support tests pass.

- [ ] **Step 9: 提交**

```bash
git add src-tauri/src/adapters/outbound/translation/libretranslate_languages.rs src-tauri/src/adapters/outbound/translation/mod.rs src-tauri/src/domain/services/translation_service.rs src-tauri/src/lib.rs src-tauri/src/ports/outbound/dictionary.rs src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs src-tauri/src/ports/outbound/translation.rs
git commit -m "feat: declare local translation language support"
```

---

### Task 4: 使用 whichlang 做源语言自动检测和翻译路由

**Files:**
- Create: `src-tauri/src/ports/outbound/language_detection.rs`
- Create: `src-tauri/src/adapters/outbound/language_detection/mod.rs`
- Create: `src-tauri/src/adapters/outbound/language_detection/whichlang_detector.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/ports/outbound/mod.rs`
- Modify: `src-tauri/src/adapters/outbound/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`
- Modify: `src-tauri/src/commands/settings.rs`

- [ ] **Step 1: 添加 whichlang 依赖**

Modify `src-tauri/Cargo.toml` dependencies:

```toml
whichlang = "0.1.1"
```

Run:

```bash
cd src-tauri && cargo update -p whichlang
```

Expected: `Cargo.lock` includes `whichlang`.

- [ ] **Step 2: 新增语言检测端口**

Create `src-tauri/src/ports/outbound/language_detection.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedLanguage {
    Known(String),
    Unknown,
}

pub trait LanguageDetector: Send + Sync {
    fn detect(&self, text: &str) -> DetectedLanguage;
}
```

Modify `src-tauri/src/ports/outbound/mod.rs`:

```rust
pub mod dictionary;
pub mod language_detection;
pub mod repository;
pub mod selection_ui;
pub mod speech;
pub mod translation;
pub mod word_insight;
```

- [ ] **Step 3: 新增 whichlang adapter**

Create `src-tauri/src/adapters/outbound/language_detection/mod.rs`:

```rust
pub mod whichlang_detector;
```

Create `src-tauri/src/adapters/outbound/language_detection/whichlang_detector.rs`:

```rust
use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};
use whichlang::{detect_language, Lang};

#[derive(Debug, Clone, Default)]
pub struct WhichlangLanguageDetector;

impl LanguageDetector for WhichlangLanguageDetector {
    fn detect(&self, text: &str) -> DetectedLanguage {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return DetectedLanguage::Unknown;
        }

        lang_to_libretranslate_code(detect_language(trimmed))
            .map(|code| DetectedLanguage::Known(code.to_string()))
            .unwrap_or(DetectedLanguage::Unknown)
    }
}

fn lang_to_libretranslate_code(lang: Lang) -> Option<&'static str> {
    match lang {
        Lang::Ara => Some("ar"),
        Lang::Cmn => Some("zh"),
        Lang::Deu => Some("de"),
        Lang::Eng => Some("en"),
        Lang::Fra => Some("fr"),
        Lang::Hin => Some("hi"),
        Lang::Ita => Some("it"),
        Lang::Jpn => Some("ja"),
        Lang::Kor => Some("ko"),
        Lang::Nld => Some("nl"),
        Lang::Por => Some("pt"),
        Lang::Rus => Some("ru"),
        Lang::Spa => Some("es"),
        Lang::Swe => Some("sv"),
        Lang::Tur => Some("tr"),
        Lang::Vie => Some("vi"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_english_for_english_text() {
        let detector = WhichlangLanguageDetector;

        let result = detector.detect("This is a short English sentence for language detection.");

        assert_eq!(result, DetectedLanguage::Known("en".to_string()));
    }

    #[test]
    fn detect_returns_unknown_for_empty_text() {
        let detector = WhichlangLanguageDetector;

        let result = detector.detect("  ");

        assert_eq!(result, DetectedLanguage::Unknown);
    }

    #[test]
    fn lang_to_libretranslate_code_maps_chinese() {
        assert_eq!(lang_to_libretranslate_code(Lang::Cmn), Some("zh"));
    }
}
```

Modify `src-tauri/src/adapters/outbound/mod.rs`:

```rust
pub mod config;
pub mod dictionary;
pub mod language_detection;
pub mod selection_ui;
pub mod sqlite;
pub mod translation;
pub mod tts;
```

- [ ] **Step 4: 将 detector 注入 TranslationService**

In `src-tauri/src/domain/services/translation_service.rs`, add the import:

```rust
use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};
```

Change the struct:

```rust
#[derive(Clone)]
pub struct TranslationService {
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
    local_engine_config: LocalEngineConfig,
    libretranslate_languages: LibreTranslateLanguages,
    language_detector: Arc<dyn LanguageDetector>,
}
```

Change the constructor:

```rust
pub fn new(
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
    local_engine_config: LocalEngineConfig,
    libretranslate_languages: LibreTranslateLanguages,
    language_detector: Arc<dyn LanguageDetector>,
) -> Self {
    Self {
        dictionary_provider,
        local_engine_config,
        libretranslate_languages,
        language_detector,
    }
}
```

In `src-tauri/src/lib.rs`, add:

```rust
use adapters::outbound::language_detection::whichlang_detector::WhichlangLanguageDetector;
```

Before `TranslationService::new`, add:

```rust
let language_detector = Arc::new(WhichlangLanguageDetector);
```

Pass `language_detector` into `TranslationService::new(...)`.

- [ ] **Step 5: 修改默认设置**

In `src-tauri/src/commands/settings.rs`, replace the translation defaults:

```rust
// 翻译设置
("translationEngine", "local"),
("sourceLanguage", "auto"),
("targetLanguage", "zh"),
("apiEndpoint", ""),
("apiKey", ""),
("apiSecret", ""),
("apiRegion", ""),
("translationModel", ""),
("translationPrompt", ""),
("wordDetailPrompt", ""),
("translationTimeoutMs", "15000"),
```

- [ ] **Step 6: 增加 TranslationService 语言解析和引擎归一化 helper**

In `src-tauri/src/domain/services/translation_service.rs`, add imports:

```rust
use crate::adapters::outbound::translation::libretranslate_languages::{
    is_supported_source_language, is_supported_target_language, normalize_language_code,
    LibreTranslateLanguages,
};
use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};
```

Add helper types and functions before `impl TranslationService`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedLanguages {
    source_lang: String,
    target_lang: String,
}

fn resolve_languages(
    settings: &HashMap<String, String>,
    language_detector: &dyn LanguageDetector,
    text: &str,
    request_source_lang: &str,
    request_target_lang: &str,
) -> ResolvedLanguages {
    let configured_source = setting_or_default(settings, "sourceLanguage", request_source_lang);
    let configured_target = setting_or_default(settings, "targetLanguage", request_target_lang);
    let source_lang = if configured_source.trim().is_empty()
        || configured_source.trim().eq_ignore_ascii_case("auto")
    {
        match language_detector.detect(text) {
            DetectedLanguage::Known(lang) => lang,
            DetectedLanguage::Unknown => "auto".to_string(),
        }
    } else {
        configured_source
    };

    ResolvedLanguages {
        source_lang,
        target_lang: if configured_target.trim().is_empty() {
            "zh".to_string()
        } else {
            configured_target
        },
    }
}

fn validate_local_language_support(
    languages: &LibreTranslateLanguages,
    source_lang: &str,
    target_lang: &str,
) -> Result<(), TranslationError> {
    if !is_supported_source_language(languages, source_lang) {
        return Err(TranslationError::UnsupportedLanguage(source_lang.to_string()));
    }

    if !is_supported_target_language(languages, target_lang) {
        return Err(TranslationError::UnsupportedLanguage(target_lang.to_string()));
    }

    Ok(())
}
```

- [ ] **Step 7: 修改 translate 路由**

Replace the body of `TranslationService::translate` with:

```rust
pub async fn translate(
    &self,
    settings: HashMap<String, String>,
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<TranslationResult, String> {
    validate_text(&text).map_err(|e| e.to_string())?;

    let config = build_translation_config(&settings);
    let engine = config.engine.trim().to_lowercase();
    let resolved_languages = resolve_languages(
        &settings,
        self.language_detector.as_ref(),
        &text,
        &source_lang,
        &target_lang,
    );

    if engine == "local" {
        if should_lookup_dictionary(&text) {
            if let Some(provider) = &self.dictionary_provider {
                if provider.supports_language_pair(
                    &resolved_languages.source_lang,
                    &resolved_languages.target_lang,
                ) {
                    match provider.lookup(DictionaryLookupRequest {
                        text: text.clone(),
                        source_lang: resolved_languages.source_lang.clone(),
                        target_lang: resolved_languages.target_lang.clone(),
                    }) {
                        Ok(Some(result)) => {
                            return Ok(TranslationResult {
                                translation: result.translation,
                                detected_source_lang: Some(resolved_languages.source_lang),
                                phonetic: result.phonetic,
                                part_of_speech: result.part_of_speech,
                                definitions: result.definitions,
                                examples: result.examples,
                            });
                        }
                        Ok(None) => {}
                        Err(error) => {
                            warn!("Dictionary lookup failed, falling back to local LibreTranslate: {error}");
                        }
                    }
                }
            }
        }

        validate_local_language_support(
            &self.libretranslate_languages,
            &resolved_languages.source_lang,
            &resolved_languages.target_lang,
        )
        .map_err(|error| error.to_string())?;

        let provider = LibreTranslateProvider::new(TranslationConfig {
            engine: "libretranslate".to_string(),
            api_endpoint: self.local_engine_config.libretranslate_endpoint.clone(),
            api_key: String::new(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: config.timeout_ms,
        })
        .map_err(|error| error.to_string())?;

        let request = TranslationRequest {
            text,
            source_lang: normalize_language_code(&resolved_languages.source_lang),
            target_lang: normalize_language_code(&resolved_languages.target_lang),
        };
        return provider.translate(request).await.map_err(|e| e.to_string());
    }

    let provider = create_translation_provider(TranslationConfig {
        engine,
        ..config
    })?;
    let request = TranslationRequest {
        text,
        source_lang: resolved_languages.source_lang,
        target_lang: resolved_languages.target_lang,
    };
    provider.translate(request).await.map_err(|e| e.to_string())
}
```

- [ ] **Step 8: 更新 provider factory**

In `create_translation_provider`, remove `"libretranslate"` from the match arm. The factory must not accept `"local"`, `"libretranslate"`, or LLM model provider names as first-class `translationEngine` values. The final match should start with:

```rust
match config.engine.trim().to_lowercase().as_str() {
    "local" => Err(TranslationError::UnsupportedEngine("local".to_string()).to_string()),
    "custom" => CustomTranslationProvider::new(config)
        .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
        .map_err(|error| error.to_string()),
```

Keep the existing vendor arms for `deepl/google/microsoft/baidu/tencent/youdao`.

- [ ] **Step 9: 更新 word detail provider 对 local 的错误**

In `create_word_insight_provider`, include `local` in the unsupported arm:

```rust
"local" | "libretranslate" | "deepl" | "google" | "microsoft" | "baidu" | "tencent" | "youdao" => {
    Err(TranslationError::UnsupportedEngine(config.engine).to_string())
}
```

- [ ] **Step 10: 添加 TranslationService 路由测试**

In `src-tauri/src/domain/services/translation_service.rs` test module, add:

```rust
use crate::ports::outbound::dictionary::{DictionaryError, DictionaryLookupResult};
use crate::ports::outbound::language_detection::{DetectedLanguage, LanguageDetector};

struct MockDictionaryProvider {
    supports: bool,
    result: Option<DictionaryLookupResult>,
}

impl DictionaryProvider for MockDictionaryProvider {
    fn supports_language_pair(&self, _source_lang: &str, _target_lang: &str) -> bool {
        self.supports
    }

    fn lookup(
        &self,
        _request: DictionaryLookupRequest,
    ) -> Result<Option<DictionaryLookupResult>, DictionaryError> {
        Ok(self.result.clone())
    }
}

struct MockLanguageDetector {
    result: DetectedLanguage,
}

impl LanguageDetector for MockLanguageDetector {
    fn detect(&self, _text: &str) -> DetectedLanguage {
        self.result.clone()
    }
}

fn service_with_dictionary(
    supports: bool,
    result: Option<DictionaryLookupResult>,
) -> TranslationService {
    TranslationService::new(
        Some(Arc::new(MockDictionaryProvider { supports, result })),
        LocalEngineConfig::default_local(),
        test_libretranslate_languages(),
        Arc::new(MockLanguageDetector {
            result: DetectedLanguage::Known("en".to_string()),
        }),
    )
}

#[test]
fn resolve_languages_uses_detected_source_when_configured_auto() {
    let settings = HashMap::from([
        ("sourceLanguage".to_string(), "auto".to_string()),
        ("targetLanguage".to_string(), "zh-CN".to_string()),
    ]);

    let detector = MockLanguageDetector {
        result: DetectedLanguage::Known("en".to_string()),
    };
    let result = resolve_languages(&settings, &detector, "hello", "auto", "ja");

    assert_eq!(result.source_lang, "en");
}

#[test]
fn validate_local_language_support_rejects_unknown_target() {
    let languages = test_libretranslate_languages();
    let result = validate_local_language_support(&languages, "en", "xx");

    assert_eq!(
        result,
        Err(TranslationError::UnsupportedLanguage("xx".to_string()))
    );
}

#[tokio::test]
async fn translate_returns_dictionary_result_for_local_supported_pair() {
    let service = service_with_dictionary(
        true,
        Some(DictionaryLookupResult {
            word: "hello".to_string(),
            translation: "int. 你好".to_string(),
            phonetic: Some("həˈləʊ".to_string()),
            part_of_speech: vec!["int".to_string()],
            definitions: vec!["int. 你好".to_string()],
            examples: Vec::new(),
        }),
    );
    let settings = HashMap::from([
        ("translationEngine".to_string(), "local".to_string()),
        ("sourceLanguage".to_string(), "en".to_string()),
        ("targetLanguage".to_string(), "zh-CN".to_string()),
    ]);

    let result = service
        .translate(settings, "hello".to_string(), "auto".to_string(), "zh-CN".to_string())
        .await
        .unwrap();

    assert_eq!(result.translation, "int. 你好");
}

#[tokio::test]
async fn translate_skips_dictionary_for_custom_engine() {
    let service = service_with_dictionary(
        true,
        Some(DictionaryLookupResult {
            word: "hello".to_string(),
            translation: "int. 你好".to_string(),
            phonetic: None,
            part_of_speech: Vec::new(),
            definitions: Vec::new(),
            examples: Vec::new(),
        }),
    );
    let settings = HashMap::from([
        ("translationEngine".to_string(), "custom".to_string()),
        ("sourceLanguage".to_string(), "en".to_string()),
        ("targetLanguage".to_string(), "zh-CN".to_string()),
    ]);

    let error = service
        .translate(settings, "hello".to_string(), "en".to_string(), "zh-CN".to_string())
        .await
        .unwrap_err();

    assert_ne!(error, "int. 你好");
}
```

- [ ] **Step 11: 运行后端路由测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::language_detection::whichlang_detector
cd src-tauri && cargo test domain::services::translation_service
```

Expected: whichlang adapter and translation service tests pass without requiring a running LibreTranslate service.

- [ ] **Step 12: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/ports/outbound/language_detection.rs src-tauri/src/ports/outbound/mod.rs src-tauri/src/adapters/outbound/language_detection src-tauri/src/adapters/outbound/mod.rs src-tauri/src/lib.rs src-tauri/src/domain/services/translation_service.rs src-tauri/src/commands/settings.rs
git commit -m "feat: route local translations through dictionary first"
```

---

### Task 5: 设置页从 LibreTranslate 配置读取源语言和目标语言

**Files:**
- Create: `src-tauri/src/commands/translation_languages.rs`
- Create: `src/lib/api/translationLanguages.ts`
- Create: `src/pages/Settings/panels/translationSettingsModel.ts`
- Create: `src/pages/Settings/panels/__test__/translationSettingsModel.test.ts`
- Modify: `src-tauri/src/domain/services/translation_service.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/api/index.ts`
- Modify: `src/pages/Settings/panels/TranslationPanel.tsx`
- Modify: `src/locales/ar/common.json`
- Modify: `src/locales/de/common.json`
- Modify: `src/locales/en/common.json`
- Modify: `src/locales/es/common.json`
- Modify: `src/locales/fr/common.json`
- Modify: `src/locales/hi/common.json`
- Modify: `src/locales/id/common.json`
- Modify: `src/locales/ja/common.json`
- Modify: `src/locales/ko/common.json`
- Modify: `src/locales/pt/common.json`
- Modify: `src/locales/ru/common.json`
- Modify: `src/locales/th/common.json`
- Modify: `src/locales/vi/common.json`
- Modify: `src/locales/zh-CN/common.json`
- Modify: `src/locales/zh-TW/common.json`

- [ ] **Step 1: 给 TranslationService 暴露语言配置只读访问器**

In `src-tauri/src/domain/services/translation_service.rs`, add this method inside `impl TranslationService`:

```rust
pub fn libretranslate_languages(&self) -> &LibreTranslateLanguages {
    &self.libretranslate_languages
}
```

- [ ] **Step 2: 新增 Tauri 命令返回语言配置**

Create `src-tauri/src/commands/translation_languages.rs`:

```rust
use crate::adapters::outbound::translation::libretranslate_languages::LibreTranslateLanguages;
use crate::commands::AppState;

#[tauri::command]
pub fn get_translation_languages(
    state: tauri::State<'_, AppState>,
) -> Result<LibreTranslateLanguages, String> {
    Ok(state.translation_service.libretranslate_languages().clone())
}
```

- [ ] **Step 3: 注册命令模块**

Modify `src-tauri/src/commands/mod.rs`:

```rust
pub mod review;
pub mod settings;
pub mod tags;
pub mod translate;
pub mod translation_languages;
pub mod tts;
pub mod window;
pub mod word_details;
pub mod words;
```

Modify `src-tauri/src/lib.rs` invoke handler and add:

```rust
commands::translation_languages::get_translation_languages,
```

- [ ] **Step 4: 新增前端 API**

Create `src/lib/api/translationLanguages.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

export type TranslationLanguage = {
  code: string;
  name: string;
};

export type TranslationLanguages = {
  sourceLanguages: TranslationLanguage[];
  targetLanguages: TranslationLanguage[];
};

type RustTranslationLanguages = {
  source_languages: TranslationLanguage[];
  target_languages: TranslationLanguage[];
};

export async function getTranslationLanguages(): Promise<TranslationLanguages> {
  const result = await invoke<RustTranslationLanguages>("get_translation_languages");

  return {
    sourceLanguages: Array.isArray(result.source_languages)
      ? result.source_languages
      : [],
    targetLanguages: Array.isArray(result.target_languages)
      ? result.target_languages
      : [],
  };
}
```

Modify `src/lib/api/index.ts` and add:

```ts
export * from "./translationLanguages";
```

- [ ] **Step 5: 新增设置页纯逻辑文件**

Create `src/pages/Settings/panels/translationSettingsModel.ts`:

```ts
import type { TranslationLanguage, TranslationLanguages } from "../../../lib/api";

export type TranslationEngine =
  | "local"
  | "google"
  | "deepl"
  | "microsoft"
  | "baidu"
  | "tencent"
  | "youdao"
  | "custom";

export const emptyTranslationLanguages: TranslationLanguages = {
  sourceLanguages: [],
  targetLanguages: [],
};

export function getTranslationFieldVisibility(engine: TranslationEngine) {
  return {
    needsEndpoint: ["google", "deepl", "microsoft", "custom"].includes(engine),
    endpointOptional: ["google", "deepl", "microsoft"].includes(engine),
    needsApiKey: [
      "google",
      "deepl",
      "microsoft",
      "baidu",
      "tencent",
      "youdao",
      "custom",
    ].includes(engine),
    needsApiSecret: ["baidu", "tencent", "youdao"].includes(engine),
    needsApiRegion: ["microsoft", "tencent"].includes(engine),
    isCustomEngine: engine === "custom",
  };
}

export function hasLanguage(languages: TranslationLanguage[], code: string) {
  return languages.some((language) => language.code === code);
}
```

- [ ] **Step 6: 新增设置逻辑测试**

Create `src/pages/Settings/panels/__test__/translationSettingsModel.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import {
  getTranslationFieldVisibility,
  hasLanguage,
} from "../translationSettingsModel";

describe("translationSettingsModel", () => {
  it("hides credential fields for local engine", () => {
    expect(getTranslationFieldVisibility("local")).toEqual({
      needsEndpoint: false,
      endpointOptional: false,
      needsApiKey: false,
      needsApiSecret: false,
      needsApiRegion: false,
      isCustomEngine: false,
    });
  });

  it("checks language membership from config data", () => {
    expect(hasLanguage([{ code: "en", name: "English" }], "en")).toBe(true);
  });
});
```

- [ ] **Step 7: 修改 TranslationPanel imports 和语言状态**

In `src/pages/Settings/panels/TranslationPanel.tsx`, change the React import to include `useEffect`:

```ts
import { useEffect, useState } from "react";
```

Add API/model imports:

```ts
import { getTranslationLanguages, setSetting } from "../../../lib/api";
import {
  emptyTranslationLanguages,
  getTranslationFieldVisibility,
  hasLanguage,
} from "./translationSettingsModel";
```

Remove `setSetting` from the existing import if it still imports only from `"../../../lib/api"`.

Inside `TranslationPanel`, after `updateSetting`, add:

```ts
const [translationLanguages, setTranslationLanguages] = useState(
  emptyTranslationLanguages,
);

useEffect(() => {
  let disposed = false;

  getTranslationLanguages()
    .then((languages) => {
      if (!disposed) {
        setTranslationLanguages(languages);
      }
    })
    .catch((error) => {
      console.error("读取翻译语言列表失败", error);
    });

  return () => {
    disposed = true;
  };
}, []);
```

- [ ] **Step 8: 修改引擎选项和设置派生值**

Replace the system group in `engineOptionGroups`:

```ts
{
  i18nKey: "engineGroupSystem",
  options: [{ i18nKey: "engineLocal", value: "local" }],
},
```

Replace current engine and visibility constants with:

```ts
const translationEngine = (settings.translationEngine || "local") as TranslationEngine;
const sourceLanguage = hasLanguage(
  translationLanguages.sourceLanguages,
  settings.sourceLanguage || "auto",
)
  ? settings.sourceLanguage || "auto"
  : "auto";
const targetLanguage = hasLanguage(
  translationLanguages.targetLanguages,
  settings.targetLanguage || "zh",
)
  ? settings.targetLanguage || "zh"
  : "zh";
const apiEndpoint = settings.apiEndpoint || "";
const apiKey = settings.apiKey || "";
const apiSecret = settings.apiSecret || "";
const apiRegion = settings.apiRegion || "";
const translationModel = settings.translationModel || "";
const translationPrompt = settings.translationPrompt || "";
const wordDetailPrompt = settings.wordDetailPrompt || "";
const parsedTranslationTimeoutMs = Number(
  settings.translationTimeoutMs || String(DEFAULT_TRANSLATION_TIMEOUT_MS),
);
const translationTimeoutMs = Number.isFinite(parsedTranslationTimeoutMs)
  ? parsedTranslationTimeoutMs
  : DEFAULT_TRANSLATION_TIMEOUT_MS;
const {
  needsEndpoint,
  endpointOptional,
  needsApiKey,
  needsApiSecret,
  needsApiRegion,
  isCustomEngine,
} = getTranslationFieldVisibility(translationEngine);
```

- [ ] **Step 9: 删除旧引擎迁移逻辑**

Remove the existing engine migration block entirely. `translationEngine` only accepts current values from the settings UI; old database values are fixed by database migration outside this code change.


- [ ] **Step 10: 在引擎设置下方加入源语言和目标语言 Select**

Insert after the translation engine `SettingItem`:

```tsx
<Separator />
<SettingItem
  title={t("settings.translation.sourceLanguage.title")}
  description={t("settings.translation.sourceLanguage.desc")}
>
  <Select
    className="w-48"
    value={sourceLanguage}
    onChange={(value) => value && saveSetting("sourceLanguage", String(value))}
  >
    <Label>{t("settings.translation.sourceLanguage.label")}</Label>
    <Select.Trigger>
      <Select.Value />
      <Select.Indicator />
    </Select.Trigger>
    <Select.Popover>
      <ListBox>
        {translationLanguages.sourceLanguages.map((option) => (
          <ListBox.Item key={option.code} id={option.code} textValue={option.name}>
            {option.name}
            <ListBox.ItemIndicator />
          </ListBox.Item>
        ))}
      </ListBox>
    </Select.Popover>
  </Select>
</SettingItem>

<Separator />
<SettingItem
  title={t("settings.translation.targetLanguage.title")}
  description={t("settings.translation.targetLanguage.desc")}
>
  <Select
    className="w-48"
    value={targetLanguage}
    onChange={(value) => value && saveSetting("targetLanguage", String(value))}
  >
    <Label>{t("settings.translation.targetLanguage.label")}</Label>
    <Select.Trigger>
      <Select.Value />
      <Select.Indicator />
    </Select.Trigger>
    <Select.Popover>
      <ListBox>
        {translationLanguages.targetLanguages.map((option) => (
          <ListBox.Item key={option.code} id={option.code} textValue={option.name}>
            {option.name}
            <ListBox.ItemIndicator />
          </ListBox.Item>
        ))}
      </ListBox>
    </Select.Popover>
  </Select>
</SettingItem>
```

- [ ] **Step 11: 更新中文 locale**

In `src/locales/zh-CN/common.json`, under `settings.translation`, replace:

```json
"engineLibreTranslate": "LibreTranslate"
```

with:

```json
"engineLocal": "本地"
```

Add these keys under `settings.translation`:

```json
"sourceLanguage": {
  "title": "源语言",
  "desc": "仅显示本地 LibreTranslate 支持的源语言",
  "label": "源语言"
},
"targetLanguage": {
  "title": "目标语言",
  "desc": "仅显示本地 LibreTranslate 支持的目标语言",
  "label": "目标语言"
}
```

- [ ] **Step 12: 更新繁中 locale**

In `src/locales/zh-TW/common.json`, replace `engineLibreTranslate` with:

```json
"engineLocal": "本地"
```

Add:

```json
"sourceLanguage": {
  "title": "來源語言",
  "desc": "僅顯示本地 LibreTranslate 支援的來源語言",
  "label": "來源語言"
},
"targetLanguage": {
  "title": "目標語言",
  "desc": "僅顯示本地 LibreTranslate 支援的目標語言",
  "label": "目標語言"
}
```

- [ ] **Step 13: 更新其他 locale 为英文 fallback**

For each of these files:

```txt
src/locales/ar/common.json
src/locales/de/common.json
src/locales/en/common.json
src/locales/es/common.json
src/locales/fr/common.json
src/locales/hi/common.json
src/locales/id/common.json
src/locales/ja/common.json
src/locales/ko/common.json
src/locales/pt/common.json
src/locales/ru/common.json
src/locales/th/common.json
src/locales/vi/common.json
```

Replace `engineLibreTranslate` with:

```json
"engineLocal": "Local"
```

Add:

```json
"sourceLanguage": {
  "title": "Source Language",
  "desc": "Only languages supported by the local LibreTranslate configuration are shown",
  "label": "Source Language"
},
"targetLanguage": {
  "title": "Target Language",
  "desc": "Only languages supported by the local LibreTranslate configuration are shown",
  "label": "Target Language"
}
```

- [ ] **Step 14: 运行前端设置模型测试**

Run:

```bash
pnpm test -- src/pages/Settings/panels/__test__/translationSettingsModel.test.ts
```

Expected: two `translationSettingsModel` tests pass.

- [ ] **Step 15: 运行 Rust 命令编译检查**

Run:

```bash
cd src-tauri && cargo test commands::translation_languages
```

Expected: command module compiles. It may report 0 tests, which is acceptable for this command wrapper.

- [ ] **Step 16: 运行 TypeScript 构建**

Run:

```bash
pnpm build
```

Expected: TypeScript compile and Vite build pass.

- [ ] **Step 17: 提交**

```bash
git add src-tauri/src/commands/translation_languages.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/domain/services/translation_service.rs src/lib/api/translationLanguages.ts src/lib/api/index.ts src/pages/Settings/panels/TranslationPanel.tsx src/pages/Settings/panels/translationSettingsModel.ts src/pages/Settings/panels/__test__/translationSettingsModel.test.ts src/locales
git commit -m "feat: load translation languages from local config"
```

---

### Task 6: 全量验证和收尾

**Files:**
- Review: all files modified in Tasks 1-5

- [ ] **Step 1: 运行 Rust 测试**

Run:

```bash
cd src-tauri && cargo test
```

Expected: all Rust tests pass. If this command attempts to build app resources, `resources/stardict-ecdict/` directory packaging should prevent the previous missing `.ifo` build failure.

- [ ] **Step 2: 运行前端测试**

Run:

```bash
pnpm test
```

Expected: all Vitest suites pass, including existing SelectionPopup and AccessibilityPermission tests.

- [ ] **Step 3: 运行前端构建**

Run:

```bash
pnpm build
```

Expected: TypeScript and Vite build pass.

- [ ] **Step 4: 运行 Rust 格式化和检查**

Run:

```bash
cd src-tauri && cargo fmt --check
```

Expected: no formatting diff.

Run:

```bash
cd src-tauri && cargo clippy --all-targets --all-features --locked -- -D warnings
```

Expected: clippy passes without warnings.

- [ ] **Step 5: 手动本地服务验证**

Start local LibreTranslate outside this plan so it listens on:

```txt
http://localhost:5005
```

Run the app:

```bash
pnpm tauri dev
```

Manual checks:

```txt
1. 设置页翻译引擎显示“本地”，源语言默认“自动检测”，目标语言默认“简体中文”。
2. 选择本地引擎，设置源语言 en、目标语言 Chinese（代码 zh），查询 hello；有 ECDICT 资源时优先返回词典释义。
3. 选择本地引擎，设置源语言 ja、目标语言 Chinese（代码 zh），查询日文；词典不支持该语言对，调用 localhost:5005 LibreTranslate。
4. 选择 Google、DeepL、Microsoft、百度、腾讯、有道或自定义引擎；查询时不调用 ECDICT，不调用 localhost:5005。
5. 设置本地引擎，目标语言改成不在 LibreTranslate 支持列表中的测试值；后端返回“当前翻译引擎不支持该语言：xx”。
```

- [ ] **Step 6: 查看 diff**

Run:

```bash
git diff --stat HEAD
git diff HEAD -- src-tauri/src/domain/services/translation_service.rs
git diff HEAD -- src/pages/Settings/panels/TranslationPanel.tsx
```

Expected: diff only contains local translation routing, settings UI, locale, resource configuration, and tests.

- [ ] **Step 7: 最终提交**

If Task 6 introduced verification-only fixes, commit them:

```bash
git add -A
git commit -m "test: verify local translation routing"
```

If there are no changes after verification, skip this commit and keep the branch at the last feature commit.

---

## 自检

- Spec coverage:
  - `translationEngine` 持久化为 `local`：Task 4、Task 5。
  - 设置中加入源语言和目标语言，源语言默认自动检测：Task 4、Task 5。
  - 轻量语言检测使用现成 Rust 库，不从零实现：Task 4 使用 `whichlang` crate，并通过 outbound port/adapter 注入。
  - 本地引擎先调用词典，词典不支持再调用 LibreTranslate：Task 3、Task 4。
  - 厂商 API 和自定义 API 不调用词典与 LibreTranslate：Task 4 测试覆盖 custom，Task 6 手动验证覆盖 vendor/custom。
  - LibreTranslate 支持语言配置文件记录、后端校验、前端选项来源：Task 1、Task 3、Task 5。
  - 查询语言不在支持范围返回错误：Task 3、Task 4。
  - 本地 LibreTranslate endpoint 写入 Tauri resource 配置文件，不进入 settings：Task 1、Task 2。
  - Tauri 官方推荐资源目录方式：Task 1、Task 2。
- Placeholder scan: 未发现占位式步骤、空洞实现说明或留待补全的代码块。
- Type consistency:
  - 前端设置键使用 `translationEngine/sourceLanguage/targetLanguage`。
  - 本地资源配置字段使用 JSON `libretranslateEndpoint`，Rust struct 字段使用 `libretranslate_endpoint`。
  - 后端错误使用现有 `TranslationError` 扩展。
  - 词典能力方法名统一为 `supports_language_pair`。
