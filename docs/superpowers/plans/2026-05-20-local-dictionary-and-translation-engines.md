# 本地词典与多翻译引擎 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 ECDICT 本地词典优先查询，并补齐 LibreTranslate、Google、DeepL、Bing / Microsoft Translator、百度、腾讯、有道、自定义大模型的翻译引擎配置和后端 adapter。

**Architecture:** 保持当前六边形架构：Tauri command 只读取 settings 并调用 domain service；domain service 负责词典优先查询与 provider 编排；`ports/outbound` 定义词典和翻译端口；`adapters/outbound` 实现 ECDICT、LibreTranslate 和厂商 API。前端设置页只管理配置，不直接接触厂商协议细节。

**Tech Stack:** Tauri 2.x, Rust 1.95, rusqlite, reqwest, serde, React 19, TypeScript, HeroUI v3, pnpm.

---

## Scope Check

本计划覆盖一个完整但边界清楚的功能：翻译查询链路。它包含本地词典、翻译 provider、设置页配置，但不包含缓存、服务端代理、LibreTranslate 本地部署、词典下载 UI、单词详情大模型逻辑扩展。

当前工作区已有用户改动：

- `src/pages/Home/index.tsx` 未提交改动，不属于本计划，执行期间不要修改或暂存。

当前代码结构重点：

- `src-tauri/src/commands/translate.rs` 已经变薄，只调用 `TranslationService::translate`。
- `src-tauri/src/domain/services/translation_service.rs` 是翻译编排核心。
- `src-tauri/src/ports/outbound/translation.rs` 定义统一翻译端口。
- `src-tauri/src/adapters/outbound/translation/custom.rs` 是自定义大模型 provider。
- `src-tauri/src/adapters/outbound/translation/deepl.rs` 和 `google.rs` 仍是占位实现。

---

## File Structure

### Create

- `src-tauri/src/ports/outbound/dictionary.rs`  
  定义 `DictionaryProvider`、`DictionaryLookupRequest`、`DictionaryLookupResult`、`DictionaryError`。

- `src-tauri/src/adapters/outbound/dictionary/mod.rs`  
  导出 StarDict ECDICT adapter。

- `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`  
  只读 StarDict ECDICT 查询实现，读取 `.ifo/.idx/.dict`。

- `src-tauri/resources/stardict-ecdict/README.md`  
  记录 StarDict ECDICT 资源来源、授权和预期文件名。

- `src-tauri/src/adapters/outbound/translation/http_utils.rs`  
  共享 HTTP 错误处理、密钥脱敏、timeout 映射。

- `src-tauri/src/adapters/outbound/translation/libretranslate.rs`  
  LibreTranslate HTTP API provider。

- `src-tauri/src/adapters/outbound/translation/microsoft.rs`  
  Bing / Microsoft Translator provider。

- `src-tauri/src/adapters/outbound/translation/baidu.rs`  
  百度翻译 provider。

- `src-tauri/src/adapters/outbound/translation/tencent.rs`  
  腾讯机器翻译 provider。

- `src-tauri/src/adapters/outbound/translation/youdao.rs`  
  有道智云翻译 provider。

### Modify

- `src-tauri/Cargo.toml`  
  增加签名和测试依赖：`md5`、`sha2`、`hmac`、`hex`、`base64`。

- `src-tauri/tauri.conf.json`  
  增加 StarDict ECDICT 资源打包配置。

- `src-tauri/src/ports/outbound/mod.rs`  
  导出 `dictionary`。

- `src-tauri/src/adapters/outbound/mod.rs`  
  导出 `dictionary`。

- `src-tauri/src/adapters/outbound/translation/mod.rs`  
  导出新增 provider 和 `http_utils`。

- `src-tauri/src/ports/outbound/translation.rs`  
  扩展 `TranslationConfig`：`api_secret`、`api_region`；扩展错误：`MissingApiSecret`、`MissingRegion`。

- `src-tauri/src/domain/services/translation_service.rs`  
  从静态工具服务改为可持有 `DictionaryProvider` 的 domain service，并实现词典优先查询。

- `src-tauri/src/commands/mod.rs`  
  `AppState` 增加 `translation_service`。

- `src-tauri/src/lib.rs`  
  setup 阶段解析 StarDict ECDICT 资源路径并构造 `TranslationService`。

- `src-tauri/src/commands/translate.rs`  
  调用 `state.translation_service.translate(...)`。

- `src-tauri/src/commands/word_details.rs`  
  调用 `state.translation_service.generate_word_detail(...)`。

- `src-tauri/src/commands/settings.rs`  
  初始化新增 settings key，并把默认引擎改为 `libretranslate`。

- `src/pages/Settings/panels/TranslationPanel.tsx`  
  翻译引擎分组展示，按引擎动态显示字段。

- `src/locales/*/common.json`  
  补齐新增翻译设置文案。

---

## Task 1: 扩展翻译配置端口和 settings 默认值

**Files:**

- Modify: `src-tauri/src/ports/outbound/translation.rs`
- Modify: `src-tauri/src/commands/settings.rs`
- Test: `src-tauri/src/domain/services/translation_service.rs`

- [ ] **Step 1: 写端口字段测试**

在 `src-tauri/src/domain/services/translation_service.rs` 的 `#[cfg(test)] mod tests` 中增加：

```rust
#[test]
fn build_translation_config_reads_secret_region_and_defaults_to_libretranslate() {
    let settings = HashMap::from([
        ("apiSecret".to_string(), "secret-value".to_string()),
        ("apiRegion".to_string(), "eastasia".to_string()),
    ]);

    let config = build_translation_config(&settings);

    assert_eq!(config.engine, "libretranslate");
    assert_eq!(config.api_secret, "secret-value");
    assert_eq!(config.api_region, "eastasia");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run:

```bash
cd src-tauri && cargo test domain::services::translation_service::tests::build_translation_config_reads_secret_region_and_defaults_to_libretranslate
```

Expected: FAIL，原因是 `TranslationConfig` 还没有 `api_secret` / `api_region` 字段，默认 engine 仍是 `custom`。

- [ ] **Step 3: 扩展 `TranslationConfig` 和错误类型**

修改 `src-tauri/src/ports/outbound/translation.rs`：

```rust
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
```

在 `TranslationError` 中增加：

```rust
#[error("请先在设置页填写 API Secret")]
MissingApiSecret,
#[error("请先在设置页填写服务区域")]
MissingRegion,
```

- [ ] **Step 4: 更新 config 构造**

修改 `src-tauri/src/domain/services/translation_service.rs` 的 `build_translation_config`：

```rust
fn build_translation_config(settings: &HashMap<String, String>) -> TranslationConfig {
    TranslationConfig {
        engine: setting_or_default(settings, "translationEngine", "libretranslate"),
        api_endpoint: setting_or_default(settings, "apiEndpoint", ""),
        api_key: setting_or_default(settings, "apiKey", ""),
        api_secret: setting_or_default(settings, "apiSecret", ""),
        api_region: setting_or_default(settings, "apiRegion", ""),
        translation_model: setting_or_default(settings, "translationModel", ""),
        translation_prompt: setting_or_default(settings, "translationPrompt", ""),
        word_detail_prompt: setting_or_default(settings, "wordDetailPrompt", ""),
        timeout_ms: setting_or_default(settings, "translationTimeoutMs", "15000")
            .parse::<u64>()
            .unwrap_or(15_000),
    }
}
```

更新 `custom.rs` 测试里的 `valid_config()`，补上：

```rust
api_secret: String::new(),
api_region: String::new(),
```

- [ ] **Step 5: 更新 settings 默认值**

修改 `src-tauri/src/commands/settings.rs` 中翻译设置默认值：

```rust
// 翻译设置
("translationEngine", "libretranslate"),
("apiEndpoint", ""),
("apiKey", ""),
("apiSecret", ""),
("apiRegion", ""),
("translationModel", ""),
("translationPrompt", ""),
("wordDetailPrompt", ""),
("translationTimeoutMs", "15000"),
```

- [ ] **Step 6: 验证测试通过**

Run:

```bash
cd src-tauri && cargo test domain::services::translation_service::tests::build_translation_config_reads_secret_region_and_defaults_to_libretranslate
```

Expected: PASS。

- [ ] **Step 7: 验证全量 Rust 测试**

Run:

```bash
cd src-tauri && cargo test
```

Expected: PASS，允许 existing unused warnings。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/ports/outbound/translation.rs src-tauri/src/domain/services/translation_service.rs src-tauri/src/adapters/outbound/translation/custom.rs src-tauri/src/commands/settings.rs
git commit -m "feat: extend translation configuration fields"
```

---

## Task 2: 新增 DictionaryProvider 端口

**Files:**

- Create: `src-tauri/src/ports/outbound/dictionary.rs`
- Modify: `src-tauri/src/ports/outbound/mod.rs`
- Test: `src-tauri/src/ports/outbound/dictionary.rs`

- [ ] **Step 1: 创建词典端口文件**

Create `src-tauri/src/ports/outbound/dictionary.rs`：

```rust
use crate::ports::outbound::translation::TranslationExample;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryLookupRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryLookupResult {
    pub word: String,
    pub translation: String,
    pub phonetic: Option<String>,
    pub part_of_speech: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<TranslationExample>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DictionaryError {
    #[error("词典查询文本不能为空")]
    EmptyText,
    #[error("词典资源不存在：{0}")]
    ResourceMissing(String),
    #[error("词典查询失败：{0}")]
    QueryFailed(String),
}

pub trait DictionaryProvider: Send + Sync {
    fn lookup(
        &self,
        request: DictionaryLookupRequest,
    ) -> Result<Option<DictionaryLookupResult>, DictionaryError>;
}

pub fn normalize_dictionary_text(text: &str) -> String {
    text.trim().to_lowercase()
}

pub fn should_lookup_dictionary(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.chars().count() > 48 {
        return false;
    }

    if trimmed.ends_with('.')
        || trimmed.ends_with('?')
        || trimmed.ends_with('!')
        || trimmed.ends_with('。')
        || trimmed.ends_with('？')
        || trimmed.ends_with('！')
    {
        return false;
    }

    trimmed.split_whitespace().count() <= 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_dictionary_text_trims_and_lowercases() {
        assert_eq!(normalize_dictionary_text(" Hello "), "hello");
    }

    #[test]
    fn should_lookup_dictionary_accepts_word_and_short_phrase() {
        assert!(should_lookup_dictionary("hello"));
        assert!(should_lookup_dictionary("look up"));
    }

    #[test]
    fn should_lookup_dictionary_rejects_sentence_like_text() {
        assert!(!should_lookup_dictionary("I like this app."));
        assert!(!should_lookup_dictionary("one two three four five"));
        assert!(!should_lookup_dictionary("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"));
    }
}
```

- [ ] **Step 2: 导出词典端口**

Modify `src-tauri/src/ports/outbound/mod.rs`：

```rust
pub mod dictionary;
pub mod repository;
pub mod translation;
pub mod word_insight;
```

- [ ] **Step 3: 运行端口测试**

Run:

```bash
cd src-tauri && cargo test ports::outbound::dictionary
```

Expected: PASS。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ports/outbound/dictionary.rs src-tauri/src/ports/outbound/mod.rs
git commit -m "feat: add dictionary provider port"
```

---

## Task 3: 实现 StarDict ECDICT 只读 adapter

**Files:**

- Create: `src-tauri/src/adapters/outbound/dictionary/mod.rs`
- Create: `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`
- Modify: `src-tauri/src/adapters/outbound/mod.rs`
- Test: `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`

- [ ] **Step 1: 导出 dictionary adapter 模块**

Create `src-tauri/src/adapters/outbound/dictionary/mod.rs`：

```rust
pub mod stardict_ecdict;
```

Modify `src-tauri/src/adapters/outbound/mod.rs`：

```rust
pub mod dictionary;
pub mod sqlite;
pub mod translation;
```

- [ ] **Step 2: 创建 StarDict ECDICT adapter**

Create `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`：

```rust
use crate::ports::outbound::dictionary::{
    normalize_dictionary_text, DictionaryError, DictionaryLookupRequest, DictionaryLookupResult,
    DictionaryProvider,
};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct StarDictEcdictDictionaryProvider {
    ifo_path: PathBuf,
    idx_path: PathBuf,
    dict_path: PathBuf,
}

#[derive(Debug)]
struct StarDictIndexEntry {
    word: String,
    offset: u32,
    size: u32,
}

impl StarDictEcdictDictionaryProvider {
    pub fn new(resource_dir: PathBuf, file_stem: &str) -> Result<Self, DictionaryError> {
        let ifo_path = resource_dir.join(format!("{file_stem}.ifo"));
        let idx_path = resource_dir.join(format!("{file_stem}.idx"));
        let dict_path = resource_dir.join(format!("{file_stem}.dict"));

        for path in [&ifo_path, &idx_path, &dict_path] {
            if !path.exists() {
                return Err(DictionaryError::ResourceMissing(path.display().to_string()));
            }
        }

        Ok(Self {
            ifo_path,
            idx_path,
            dict_path,
        })
    }

    fn lookup_entry(&self, word: &str) -> Result<Option<StarDictIndexEntry>, DictionaryError> {
        let file = File::open(&self.idx_path)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
        let mut reader = BufReader::new(file);

        loop {
            let Some(entry) = read_index_entry(&mut reader)? else {
                return Ok(None);
            };

            if entry.word == word {
                return Ok(Some(entry));
            }
        }
    }

    fn read_definition(&self, entry: &StarDictIndexEntry) -> Result<String, DictionaryError> {
        let mut file = File::open(&self.dict_path)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
        file.seek(SeekFrom::Start(entry.offset as u64))
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

        let mut buffer = vec![0_u8; entry.size as usize];
        file.read_exact(&mut buffer)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

        String::from_utf8(buffer).map_err(|error| DictionaryError::QueryFailed(error.to_string()))
    }

    fn validate_ifo(&self) -> Result<(), DictionaryError> {
        let file = File::open(&self.ifo_path)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        reader
            .read_line(&mut first_line)
            .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

        if !first_line.trim().starts_with("StarDict") {
            return Err(DictionaryError::ResourceMissing(
                self.ifo_path.display().to_string(),
            ));
        }

        Ok(())
    }
}

impl DictionaryProvider for StarDictEcdictDictionaryProvider {
    fn lookup(
        &self,
        request: DictionaryLookupRequest,
    ) -> Result<Option<DictionaryLookupResult>, DictionaryError> {
        let word = normalize_dictionary_text(&request.text);
        if word.is_empty() {
            return Err(DictionaryError::EmptyText);
        }

        self.validate_ifo()?;

        let Some(entry) = self.lookup_entry(&word)? else {
            return Ok(None);
        };

        let raw_definition = self.read_definition(&entry)?;
        let parsed = parse_ecdict_definition(&raw_definition);

        Ok(Some(DictionaryLookupResult {
            word: entry.word,
            translation: parsed.translation,
            phonetic: parsed.phonetic,
            part_of_speech: parsed.part_of_speech,
            definitions: parsed.definitions,
            examples: parsed.examples,
        }))
    }
}

fn read_index_entry<R: Read>(reader: &mut R) -> Result<Option<StarDictIndexEntry>, DictionaryError> {
    let mut word_bytes = Vec::new();
    let mut byte = [0_u8; 1];

    loop {
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                if byte[0] == 0 {
                    break;
                }
                word_bytes.push(byte[0]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                if word_bytes.is_empty() {
                    return Ok(None);
                }
                return Err(DictionaryError::QueryFailed(error.to_string()));
            }
            Err(error) => return Err(DictionaryError::QueryFailed(error.to_string())),
        }
    }

    let mut offset_bytes = [0_u8; 4];
    let mut size_bytes = [0_u8; 4];
    reader
        .read_exact(&mut offset_bytes)
        .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;
    reader
        .read_exact(&mut size_bytes)
        .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

    let word = String::from_utf8(word_bytes)
        .map_err(|error| DictionaryError::QueryFailed(error.to_string()))?;

    Ok(Some(StarDictIndexEntry {
        word: normalize_dictionary_text(&word),
        offset: u32::from_be_bytes(offset_bytes),
        size: u32::from_be_bytes(size_bytes),
    }))
}

#[derive(Debug, Default)]
struct ParsedEcdictDefinition {
    translation: String,
    phonetic: Option<String>,
    part_of_speech: Vec<String>,
    definitions: Vec<String>,
    examples: Vec<crate::ports::outbound::translation::TranslationExample>,
}

fn parse_ecdict_definition(raw: &str) -> ParsedEcdictDefinition {
    let mut parsed = ParsedEcdictDefinition::default();
    let lines = raw
        .lines()
        .flat_map(|line| line.split('\\n'))
        .map(|line| line.trim().trim_start_matches(';').trim())
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    for line in lines {
        if parsed.phonetic.is_none() && line.starts_with('[') && line.ends_with(']') {
            parsed.phonetic = Some(
                line.trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
            );
            continue;
        }

        if let Some((left, right)) = line.split_once('.') {
            let pos = left.trim();
            let definition = right.trim();
            if !pos.is_empty()
                && !definition.is_empty()
                && pos.chars().all(|char| char.is_ascii_alphabetic())
            {
                if !parsed.part_of_speech.iter().any(|value| value == pos) {
                    parsed.part_of_speech.push(pos.to_string());
                }
                parsed.definitions.push(line.clone());
                continue;
            }
        }

        parsed.definitions.push(line);
    }

    parsed.translation = parsed.definitions.join("\\n");
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_dictionary() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bugoo-stardict-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();

        let stem = "stardict-ecdict-2.4.2";
        fs::write(
            dir.join(format!("{stem}.ifo")),
            "StarDict's dict ifo file\nversion=2.4.2\nwordcount=1\nidxfilesize=14\nbookname=ECDICT\n",
        )
        .unwrap();

        let definition = "[həˈləʊ]\nint. 你好\nn. 问候";
        fs::write(dir.join(format!("{stem}.dict")), definition.as_bytes()).unwrap();

        let mut idx = Vec::new();
        idx.extend_from_slice(b"hello");
        idx.push(0);
        idx.extend_from_slice(&0_u32.to_be_bytes());
        idx.extend_from_slice(&(definition.len() as u32).to_be_bytes());
        fs::write(dir.join(format!("{stem}.idx")), idx).unwrap();

        dir
    }

    #[test]
    fn lookup_returns_dictionary_result_when_word_exists() {
        let dir = create_test_dictionary();
        let provider =
            StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

        let result = provider
            .lookup(DictionaryLookupRequest {
                text: "Hello".to_string(),
                source_lang: "en".to_string(),
                target_lang: "zh-CN".to_string(),
            })
            .unwrap()
            .unwrap();

        assert_eq!(result.word, "hello");
        assert_eq!(result.translation, "int. 你好\\nn. 问候");
        assert_eq!(result.phonetic, Some("həˈləʊ".to_string()));
        assert_eq!(result.part_of_speech, vec!["int", "n"]);
        assert_eq!(result.definitions, vec!["int. 你好", "n. 问候"]);
        assert!(result.examples.is_empty());
    }

    #[test]
    fn lookup_returns_none_when_word_is_missing() {
        let dir = create_test_dictionary();
        let provider =
            StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2").unwrap();

        let result = provider
            .lookup(DictionaryLookupRequest {
                text: "missing".to_string(),
                source_lang: "en".to_string(),
                target_lang: "zh-CN".to_string(),
            })
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn new_rejects_missing_dictionary_files() {
        let dir = std::env::temp_dir().join(format!("missing-{}", uuid::Uuid::new_v4()));

        let result = StarDictEcdictDictionaryProvider::new(dir, "stardict-ecdict-2.4.2");

        assert!(matches!(result, Err(DictionaryError::ResourceMissing(_))));
    }

    #[test]
    fn parse_definition_extracts_phonetic_and_pos() {
        let parsed = parse_ecdict_definition("[test]\nn. 测试\nv. 测验");

        assert_eq!(parsed.phonetic, Some("test".to_string()));
        assert_eq!(parsed.part_of_speech, vec!["n", "v"]);
        assert_eq!(parsed.definitions, vec!["n. 测试", "v. 测验"]);
    }
}
```

- [ ] **Step 3: 运行 StarDict ECDICT adapter 测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::dictionary::stardict_ecdict
```

Expected: PASS。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/adapters/outbound/mod.rs src-tauri/src/adapters/outbound/dictionary
git commit -m "feat: add stardict ecdict dictionary adapter"
```

---

## Task 4: 打包 StarDict ECDICT 资源并接入 AppState

**Files:**

- Create: `src-tauri/resources/stardict-ecdict/README.md`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`

- [ ] **Step 1: 创建词典资源说明**

Create `src-tauri/resources/stardict-ecdict/README.md`：

```markdown
# StarDict ECDICT

This directory contains the read-only StarDict ECDICT resources used by Bugoo.

Current extracted size in this workspace is about 190 MB. Keep these files out of normal source commits unless the project adopts Git LFS or a release-asset download pipeline.

Expected files:

```txt
stardict-ecdict-2.4.2.ifo
stardict-ecdict-2.4.2.idx
stardict-ecdict-2.4.2.dict
```

Current source package:

```txt
ecdict-stardict-28.zip
```

Source:

```txt
https://github.com/skywind3000/ECDICT/releases
```

The app reads the extracted StarDict files directly from:

```txt
src-tauri/resources/stardict-ecdict/
```

Do not commit vendor API keys or generated user data into this directory.
ECDICT is MIT licensed. Keep attribution in the app's About / Licenses section before public distribution.
```

- [ ] **Step 2: 更新 Tauri 资源配置**

Modify `src-tauri/tauri.conf.json` bundle section:

```json
"bundle": {
  "active": false,
  "resources": [
    "resources/stardict-ecdict/stardict-ecdict-2.4.2.ifo",
    "resources/stardict-ecdict/stardict-ecdict-2.4.2.idx",
    "resources/stardict-ecdict/stardict-ecdict-2.4.2.dict"
  ]
}
```

- [ ] **Step 3: 修改 `TranslationService` 为实例服务**

Modify `src-tauri/src/domain/services/translation_service.rs` imports:

```rust
use crate::ports::outbound::dictionary::{
    should_lookup_dictionary, DictionaryLookupRequest, DictionaryProvider,
};
use log::warn;
use std::sync::Arc;
```

Replace `pub struct TranslationService;` with:

```rust
#[derive(Clone)]
pub struct TranslationService {
    dictionary_provider: Option<Arc<dyn DictionaryProvider>>,
}

impl TranslationService {
    pub fn new(dictionary_provider: Option<Arc<dyn DictionaryProvider>>) -> Self {
        Self { dictionary_provider }
    }
```

Keep the existing associated functions as methods by adding `&self` to:

```rust
pub async fn translate(
    &self,
    settings: HashMap<String, String>,
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<TranslationResult, String> {
    validate_text(&text).map_err(|e| e.to_string())?;

    if should_lookup_dictionary(&text) {
        if let Some(provider) = &self.dictionary_provider {
            match provider.lookup(DictionaryLookupRequest {
                text: text.clone(),
                source_lang: source_lang.clone(),
                target_lang: target_lang.clone(),
            }) {
                Ok(Some(result)) => {
                    return Ok(TranslationResult {
                        translation: result.translation,
                        detected_source_lang: Some(source_lang),
                        phonetic: result.phonetic,
                        part_of_speech: result.part_of_speech,
                        definitions: result.definitions,
                        examples: result.examples,
                    });
                }
                Ok(None) => {}
                Err(error) => {
                    warn!("Dictionary lookup failed, falling back to translation provider: {error}");
                }
            }
        }
    }

    let config = build_translation_config(&settings);
    let provider = create_translation_provider(config)?;
    let request = TranslationRequest {
        text,
        source_lang,
        target_lang,
    };
    provider.translate(request).await.map_err(|e| e.to_string())
}
```

Change `generate_word_detail` signature to:

```rust
pub async fn generate_word_detail(
    &self,
    settings: HashMap<String, String>,
    word: String,
    translation: String,
    source_lang: String,
    target_lang: String,
) -> Result<GeneratedWordDetail, String> {
```

The body remains the same.

- [ ] **Step 4: 更新 AppState**

Modify `src-tauri/src/commands/mod.rs`:

```rust
use crate::domain::services::translation_service::TranslationService;
```

Update struct:

```rust
pub struct AppState {
    pub db: Arc<Database>,
    pub word_service: WordService,
    pub translation_service: TranslationService,
}
```

Update constructor:

```rust
impl AppState {
    pub fn new(db: Arc<Database>, translation_service: TranslationService) -> Self {
        AppState {
            db: db.clone(),
            word_service: WordService::new(db),
            translation_service,
        }
    }
}
```

- [ ] **Step 5: 构造 StarDict ECDICT provider**

Modify `src-tauri/src/lib.rs` imports:

```rust
use crate::adapters::outbound::dictionary::stardict_ecdict::StarDictEcdictDictionaryProvider;
use crate::domain::services::translation_service::TranslationService;
use std::sync::Arc;
use tauri::path::BaseDirectory;
```

Inside `.setup(|app| { ... })`, before `let app_state = ...`:

```rust
let dictionary_dir = app
    .path()
    .resolve("resources/stardict-ecdict", BaseDirectory::Resource)
    .unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("stardict-ecdict")
    });

let dictionary_provider = match StarDictEcdictDictionaryProvider::new(
    dictionary_dir.clone(),
    "stardict-ecdict-2.4.2",
) {
    Ok(provider) => Some(Arc::new(provider) as Arc<dyn crate::ports::outbound::dictionary::DictionaryProvider>),
    Err(error) => {
        log::warn!(
            "StarDict ECDICT dictionary unavailable at {:?}, dictionary lookup disabled: {}",
            dictionary_dir,
            error
        );
        None
    }
};

let translation_service = TranslationService::new(dictionary_provider);
```

Update AppState creation:

```rust
let app_state = AppState::new(db.clone(), translation_service);
```

- [ ] **Step 6: 更新 commands 使用实例服务**

Modify `src-tauri/src/commands/translate.rs`:

```rust
state
    .translation_service
    .translate(settings, text, source_lang, target_lang)
    .await
```

Modify `src-tauri/src/commands/word_details.rs`:

```rust
let generated = app_state
    .translation_service
    .generate_word_detail(
        settings,
        word.word.clone(),
        word.translation.clone(),
        word.source_lang,
        word.target_lang,
    )
    .await?;
```

- [ ] **Step 7: 运行编译验证**

Run:

```bash
cd src-tauri && cargo build
```

Expected: PASS。`src-tauri/resources/stardict-ecdict/stardict-ecdict-2.4.2.ifo/.idx/.dict` 已存在时启用本地词典；任一文件缺失时服务仍可启动，并在运行时记录 warning 后跳过词典查询。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/resources/stardict-ecdict/README.md src-tauri/tauri.conf.json src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/domain/services/translation_service.rs src-tauri/src/commands/translate.rs src-tauri/src/commands/word_details.rs
git commit -m "feat: wire dictionary provider into translation service"
```

---

## Task 5: 增加共享 HTTP 工具和翻译空字段 helper

**Files:**

- Create: `src-tauri/src/adapters/outbound/translation/http_utils.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/custom.rs`

- [ ] **Step 1: 创建共享 HTTP 工具**

Create `src-tauri/src/adapters/outbound/translation/http_utils.rs`：

```rust
use crate::ports::outbound::translation::{
    TranslationError, TranslationExample, TranslationResult,
};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Option<ProviderError>,
}

#[derive(Debug, Deserialize)]
struct ProviderError {
    message: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    code: Option<Value>,
}

pub fn timeout_duration(timeout_ms: u64) -> Duration {
    if timeout_ms == 0 {
        Duration::from_secs(30)
    } else {
        Duration::from_millis(timeout_ms)
    }
}

pub fn empty_translation_result(
    translation: String,
    detected_source_lang: Option<String>,
) -> TranslationResult {
    TranslationResult {
        translation,
        detected_source_lang,
        phonetic: None,
        part_of_speech: Vec::new(),
        definitions: Vec::new(),
        examples: Vec::<TranslationExample>::new(),
    }
}

pub fn map_reqwest_error(error: reqwest::Error) -> TranslationError {
    if error.is_timeout() {
        TranslationError::RequestTimeout
    } else {
        TranslationError::RequestFailed(error.to_string())
    }
}

pub fn format_http_error(status: reqwest::StatusCode, body: &str, secrets: &[&str]) -> String {
    let status_text = format!("HTTP {status}");
    let redacted_body = redact_secrets(body, secrets);
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

pub fn redact_secrets(value: &str, secrets: &[&str]) -> String {
    secrets.iter().fold(value.to_string(), |current, secret| {
        let secret = secret.trim();
        if secret.is_empty() {
            current
        } else {
            current.replace(secret, "[redacted]")
        }
    })
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
            redact_secrets("key=abc secret=def", &["abc", "def"]),
            "key=[redacted] secret=[redacted]"
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
```

- [ ] **Step 2: 导出共享模块**

Modify `src-tauri/src/adapters/outbound/translation/mod.rs`：

```rust
pub mod custom;
pub mod deepl;
pub mod google;
pub mod http_utils;
```

- [ ] **Step 3: 让 custom.rs 使用共享工具**

Modify `src-tauri/src/adapters/outbound/translation/custom.rs` imports:

```rust
use crate::adapters::outbound::translation::http_utils::{
    format_http_error, map_reqwest_error, timeout_duration,
};
```

Replace timeout construction:

```rust
let client = Client::builder()
    .timeout(timeout_duration(config.timeout_ms))
    .build()
    .map_err(|error| TranslationError::RequestFailed(error.to_string()))?;
```

Replace HTTP error call:

```rust
return Err(TranslationError::RequestFailed(format_http_error(
    status,
    &body,
    &[&self.config.api_key],
)));
```

Remove private duplicate helpers from `custom.rs`:

```rust
format_http_error
redact_api_key
non_empty_string
error_code_to_string
truncate_for_error
map_reqwest_error
ErrorResponse
CustomError
```

Update tests to import `format_http_error` from `http_utils`.

- [ ] **Step 4: 运行共享工具测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::http_utils
```

Expected: PASS。

- [ ] **Step 5: 运行 custom provider 测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::custom
```

Expected: PASS。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/adapters/outbound/translation/http_utils.rs src-tauri/src/adapters/outbound/translation/mod.rs src-tauri/src/adapters/outbound/translation/custom.rs
git commit -m "refactor: share translation HTTP utilities"
```

---

## Task 6: 实现 LibreTranslate provider

**Files:**

- Create: `src-tauri/src/adapters/outbound/translation/libretranslate.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`

- [ ] **Step 1: 创建 provider**

Create `src-tauri/src/adapters/outbound/translation/libretranslate.rs`：

```rust
use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::{normalize_endpoint, validate_text};
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider,
    TranslationRequest,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct LibreTranslateProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct LibreTranslateRequest {
    q: String,
    source: String,
    target: String,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LibreTranslateResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

impl LibreTranslateProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_endpoint.trim().is_empty() {
            return Err(TranslationError::MissingEndpoint);
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
        let payload = LibreTranslateRequest {
            q: request.text,
            source: source.clone(),
            target,
            format: "text".to_string(),
            api_key: non_empty_api_key(&self.config.api_key),
        };

        let response = self
            .client
            .post(format!("{}/translate", normalize_endpoint(&self.config.api_endpoint)))
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
            .json::<LibreTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;

        if body.translated_text.trim().is_empty() {
            return Err(TranslationError::InvalidResponse);
        }

        Ok(empty_translation_result(
            body.translated_text,
            if source == "auto" { None } else { Some(source) },
        ))
    }
}

impl TranslationProvider for LibreTranslateProvider {
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

fn non_empty_api_key(api_key: &str) -> Option<String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "libretranslate".to_string(),
            api_endpoint: "http://localhost:5000".to_string(),
            api_key: String::new(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn new_rejects_missing_endpoint() {
        let mut config = config();
        config.api_endpoint = " ".to_string();

        let result = LibreTranslateProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingEndpoint)));
    }

    #[test]
    fn normalize_source_lang_uses_auto_for_empty() {
        assert_eq!(normalize_source_lang(""), "auto");
        assert_eq!(normalize_source_lang("EN"), "en");
    }
}
```

- [ ] **Step 2: 导出并接入 factory**

Modify `src-tauri/src/adapters/outbound/translation/mod.rs`：

```rust
pub mod libretranslate;
```

Modify `src-tauri/src/domain/services/translation_service.rs` imports:

```rust
libretranslate::LibreTranslateProvider,
```

Modify `create_translation_provider`:

```rust
"libretranslate" => LibreTranslateProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
```

Modify `create_word_insight_provider` unsupported list:

```rust
"libretranslate" | "deepl" | "google" | "microsoft" | "baidu" | "tencent" | "youdao" => {
    Err(TranslationError::UnsupportedEngine(config.engine).to_string())
}
```

- [ ] **Step 3: 运行测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::libretranslate
```

Expected: PASS。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/adapters/outbound/translation/libretranslate.rs src-tauri/src/adapters/outbound/translation/mod.rs src-tauri/src/domain/services/translation_service.rs
git commit -m "feat: add LibreTranslate provider"
```

---

## Task 7: 实现 Google 和 DeepL provider

**Files:**

- Modify: `src-tauri/src/adapters/outbound/translation/google.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/deepl.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`

- [ ] **Step 1: 实现 Google provider**

Replace `src-tauri/src/adapters/outbound/translation/google.rs` with:

```rust
use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::{normalize_endpoint, validate_text};
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider,
    TranslationRequest,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GoogleTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Serialize)]
struct GoogleTranslateRequest {
    q: String,
    target: String,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateResponse {
    data: GoogleTranslateData,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslation {
    #[serde(rename = "translatedText")]
    translated_text: String,
    #[serde(rename = "detectedSourceLanguage")]
    detected_source_language: Option<String>,
}

impl GoogleTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
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
        let endpoint = if self.config.api_endpoint.trim().is_empty() {
            "https://translation.googleapis.com/language/translate/v2".to_string()
        } else {
            normalize_endpoint(&self.config.api_endpoint)
        };
        let source = optional_lang(&request.source_lang);
        let payload = GoogleTranslateRequest {
            q: request.text,
            target: request.target_lang.trim().to_lowercase(),
            format: "text".to_string(),
            source: source.clone(),
        };

        let response = self
            .client
            .post(endpoint)
            .query(&[("key", self.config.api_key.trim())])
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
            .json::<GoogleTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let translated = body
            .data
            .translations
            .into_iter()
            .next()
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(
            translated.translated_text,
            translated.detected_source_language.or(source),
        ))
    }
}

impl TranslationProvider for GoogleTranslationProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

fn optional_lang(lang: &str) -> Option<String> {
    let value = lang.trim().to_lowercase();
    if value.is_empty() || value == "auto" {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "google".to_string(),
            api_endpoint: String::new(),
            api_key: "key".to_string(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn new_rejects_missing_api_key() {
        let mut config = config();
        config.api_key = " ".to_string();

        let result = GoogleTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingApiKey)));
    }

    #[test]
    fn optional_lang_omits_auto() {
        assert_eq!(optional_lang("auto"), None);
        assert_eq!(optional_lang("EN"), Some("en".to_string()));
    }
}
```

- [ ] **Step 2: 实现 DeepL provider**

Replace `src-tauri/src/adapters/outbound/translation/deepl.rs` with:

```rust
use crate::adapters::outbound::translation::http_utils::{
    empty_translation_result, format_http_error, map_reqwest_error, timeout_duration,
};
use crate::domain::services::translation_service::{normalize_endpoint, validate_text};
use crate::ports::outbound::translation::{
    TranslationConfig, TranslationError, TranslationFuture, TranslationProvider,
    TranslationRequest,
};
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
pub struct DeepLTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslateResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslation {
    text: String,
    detected_source_language: Option<String>,
}

impl DeepLTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> {
        if config.api_key.trim().is_empty() {
            return Err(TranslationError::MissingApiKey);
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
        let endpoint = if self.config.api_endpoint.trim().is_empty() {
            "https://api-free.deepl.com/v2/translate".to_string()
        } else {
            normalize_endpoint(&self.config.api_endpoint)
        };

        let mut form = vec![
            ("text".to_string(), request.text),
            ("target_lang".to_string(), request.target_lang.trim().to_uppercase()),
        ];
        if let Some(source) = optional_deepl_lang(&request.source_lang) {
            form.push(("source_lang".to_string(), source));
        }

        let response = self
            .client
            .post(endpoint)
            .header("Authorization", format!("DeepL-Auth-Key {}", self.config.api_key.trim()))
            .form(&form)
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
            .json::<DeepLTranslateResponse>()
            .await
            .map_err(map_reqwest_error)?;
        let translated = body
            .translations
            .into_iter()
            .next()
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(empty_translation_result(
            translated.text,
            translated.detected_source_language.map(|value| value.to_lowercase()),
        ))
    }
}

impl TranslationProvider for DeepLTranslationProvider {
    fn translate<'a>(&'a self, request: TranslationRequest) -> TranslationFuture<'a> {
        Box::pin(async move { self.translate_inner(request).await })
    }
}

fn optional_deepl_lang(lang: &str) -> Option<String> {
    let value = lang.trim().to_uppercase();
    if value.is_empty() || value == "AUTO" {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TranslationConfig {
        TranslationConfig {
            engine: "deepl".to_string(),
            api_endpoint: String::new(),
            api_key: "key".to_string(),
            api_secret: String::new(),
            api_region: String::new(),
            translation_model: String::new(),
            translation_prompt: String::new(),
            word_detail_prompt: String::new(),
            timeout_ms: 1_000,
        }
    }

    #[test]
    fn new_rejects_missing_api_key() {
        let mut config = config();
        config.api_key = " ".to_string();

        let result = DeepLTranslationProvider::new(config);

        assert!(matches!(result, Err(TranslationError::MissingApiKey)));
    }

    #[test]
    fn optional_deepl_lang_omits_auto() {
        assert_eq!(optional_deepl_lang("auto"), None);
        assert_eq!(optional_deepl_lang("en"), Some("EN".to_string()));
    }
}
```

- [ ] **Step 3: 更新 provider factory**

Modify `src-tauri/src/domain/services/translation_service.rs`:

```rust
"deepl" => DeepLTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
"google" => GoogleTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
```

- [ ] **Step 4: 运行 provider 测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::google adapters::outbound::translation::deepl
```

Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/adapters/outbound/translation/google.rs src-tauri/src/adapters/outbound/translation/deepl.rs src-tauri/src/domain/services/translation_service.rs
git commit -m "feat: implement google and deepl translation providers"
```

---

## Task 8: 实现 Microsoft、百度、有道 provider

**Files:**

- Create: `src-tauri/src/adapters/outbound/translation/microsoft.rs`
- Create: `src-tauri/src/adapters/outbound/translation/baidu.rs`
- Create: `src-tauri/src/adapters/outbound/translation/youdao.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 增加签名依赖**

Modify `src-tauri/Cargo.toml`:

```toml
md5 = "0.7"
sha2 = "0.10"
```

- [ ] **Step 2: 创建 Microsoft provider**

Create `src-tauri/src/adapters/outbound/translation/microsoft.rs` with this exact protocol mapping:

```rust
// Headers:
// Ocp-Apim-Subscription-Key: api_key
// Ocp-Apim-Subscription-Region: api_region
// Content-Type: application/json
//
// Default endpoint:
// https://api.cognitive.microsofttranslator.com/translate
//
// Query:
// api-version=3.0&to={target}&from={source when not auto}
```

Implementation must:

- reject missing `api_key` with `MissingApiKey`
- reject missing `api_region` with `MissingRegion`
- parse response array first item: `translations[0].text`
- map `detectedLanguage.language` to `detected_source_lang`
- return empty learning fields
- redact `api_key` in HTTP errors

Add tests:

```rust
#[test]
fn new_rejects_missing_region() {
    let mut config = config();
    config.api_region = " ".to_string();

    let result = MicrosoftTranslationProvider::new(config);

    assert!(matches!(result, Err(TranslationError::MissingRegion)));
}
```

- [ ] **Step 3: 创建百度 provider**

Create `src-tauri/src/adapters/outbound/translation/baidu.rs` with this exact protocol mapping:

```rust
// Endpoint:
// https://fanyi-api.baidu.com/api/trans/vip/translate
//
// Parameters:
// q, from, to, appid, salt, sign
//
// sign:
// md5(appid + q + salt + secret)
```

Implementation must:

- use `api_key` as Baidu App ID
- use `api_secret` as Baidu Secret Key
- reject missing fields with `MissingApiKey` / `MissingApiSecret`
- default source language to `auto`
- parse `trans_result[0].dst`
- return empty learning fields
- redact both `api_key` and `api_secret`

Add signer test:

```rust
#[test]
fn build_sign_matches_baidu_rule() {
    assert_eq!(
        build_sign("appid", "apple", "salt", "secret"),
        format!("{:x}", md5::compute("appidapplesaltsecret"))
    );
}
```

- [ ] **Step 4: 创建有道 provider**

Create `src-tauri/src/adapters/outbound/translation/youdao.rs` with this exact protocol mapping:

```rust
// Endpoint:
// https://openapi.youdao.com/api
//
// Parameters:
// q, from, to, appKey, salt, sign, signType=v3, curtime
//
// sign:
// sha256(appKey + truncate(q) + salt + curtime + appSecret)
```

Implementation must:

- use `api_key` as Youdao App Key
- use `api_secret` as Youdao App Secret
- reject missing fields with `MissingApiKey` / `MissingApiSecret`
- implement input truncate rule:

```rust
fn truncate_for_sign(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    if chars.len() <= 20 {
        input.to_string()
    } else {
        format!(
            "{}{}{}",
            chars.iter().take(10).collect::<String>(),
            chars.len(),
            chars.iter().skip(chars.len() - 10).collect::<String>()
        )
    }
}
```

- parse `translation[0]`
- return empty learning fields
- redact both `api_key` and `api_secret`

Add tests for `truncate_for_sign` short and long input.

- [ ] **Step 5: 导出并接入 factory**

Modify `src-tauri/src/adapters/outbound/translation/mod.rs`:

```rust
pub mod baidu;
pub mod microsoft;
pub mod youdao;
```

Modify `src-tauri/src/domain/services/translation_service.rs` imports and factory:

```rust
baidu::BaiduTranslationProvider,
microsoft::MicrosoftTranslationProvider,
youdao::YoudaoTranslationProvider,
```

```rust
"microsoft" => MicrosoftTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
"baidu" => BaiduTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
"youdao" => YoudaoTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
```

- [ ] **Step 6: 运行测试**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::microsoft adapters::outbound::translation::baidu adapters::outbound::translation::youdao
```

Expected: PASS。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/adapters/outbound/translation/microsoft.rs src-tauri/src/adapters/outbound/translation/baidu.rs src-tauri/src/adapters/outbound/translation/youdao.rs src-tauri/src/adapters/outbound/translation/mod.rs src-tauri/src/domain/services/translation_service.rs
git commit -m "feat: add microsoft baidu and youdao providers"
```

---

## Task 9: 实现腾讯 provider

**Files:**

- Create: `src-tauri/src/adapters/outbound/translation/tencent.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 增加 TC3 签名依赖**

Modify `src-tauri/Cargo.toml`:

```toml
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
```

- [ ] **Step 2: 创建 Tencent provider**

Create `src-tauri/src/adapters/outbound/translation/tencent.rs`.

Provider rules:

```txt
api_key = SecretId
api_secret = SecretKey
api_region = Region
service = tmt
host = tmt.tencentcloudapi.com
action = TextTranslate
version = 2018-03-21
```

Request JSON body:

```json
{
  "SourceText": "hello",
  "Source": "en",
  "Target": "zh",
  "ProjectId": 0
}
```

Response path:

```txt
Response.TargetText
```

Implement helper signatures:

```rust
fn sha256_hex(input: &str) -> String
fn hmac_sha256(key: &[u8], message: &str) -> Vec<u8>
fn build_authorization(
    secret_id: &str,
    secret_key: &str,
    timestamp: i64,
    payload: &str,
) -> String
```

`build_authorization` must follow TC3-HMAC-SHA256:

```txt
canonical_request =
  POST
  /

  content-type:application/json; charset=utf-8
  host:tmt.tencentcloudapi.com

  content-type;host
  sha256(payload)

credential_scope = date/tmt/tc3_request
string_to_sign =
  TC3-HMAC-SHA256
  timestamp
  credential_scope
  sha256(canonical_request)
```

Provider must:

- reject missing `api_key`, `api_secret`, `api_region`
- redact `api_key` and `api_secret`
- return empty learning fields

Add tests:

```rust
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
```

- [ ] **Step 3: 导出并接入 factory**

Modify `src-tauri/src/adapters/outbound/translation/mod.rs`:

```rust
pub mod tencent;
```

Modify `src-tauri/src/domain/services/translation_service.rs` imports and factory:

```rust
tencent::TencentTranslationProvider,
```

```rust
"tencent" => TencentTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test adapters::outbound::translation::tencent
```

Expected: PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/adapters/outbound/translation/tencent.rs src-tauri/src/adapters/outbound/translation/mod.rs src-tauri/src/domain/services/translation_service.rs
git commit -m "feat: add tencent translation provider"
```

---

## Task 10: 更新设置页引擎分组和动态字段

**Files:**

- Modify: `src/pages/Settings/panels/TranslationPanel.tsx`
- Modify: `src/locales/*/common.json`

- [ ] **Step 1: 更新 TranslationPanel 引擎选项**

Replace `engineOptionKeys` in `src/pages/Settings/panels/TranslationPanel.tsx`:

```typescript
const engineOptionGroups = [
  {
    i18nKey: "engineGroupSystem",
    options: [{ i18nKey: "engineLibreTranslate", value: "libretranslate" }],
  },
  {
    i18nKey: "engineGroupVendor",
    options: [
      { i18nKey: "engineGoogle", value: "google" },
      { i18nKey: "engineDeepL", value: "deepl" },
      { i18nKey: "engineMicrosoft", value: "microsoft" },
      { i18nKey: "engineBaidu", value: "baidu" },
      { i18nKey: "engineTencent", value: "tencent" },
      { i18nKey: "engineYoudao", value: "youdao" },
    ],
  },
  {
    i18nKey: "engineGroupCustom",
    options: [{ i18nKey: "engineCustom", value: "custom" }],
  },
];
```

Add derived helpers:

```typescript
const needsEndpoint = ["libretranslate", "google", "deepl", "microsoft", "custom"].includes(translationEngine);
const endpointOptional = ["google", "deepl", "microsoft"].includes(translationEngine);
const needsApiKey = ["libretranslate", "google", "deepl", "microsoft", "baidu", "tencent", "youdao", "custom"].includes(translationEngine);
const apiKeyOptional = translationEngine === "libretranslate";
const needsApiSecret = ["baidu", "tencent", "youdao"].includes(translationEngine);
const needsApiRegion = ["microsoft", "tencent"].includes(translationEngine);
const isCustomEngine = translationEngine === "custom";
```

- [ ] **Step 2: Render grouped ListBox items**

Replace `ListBox` contents with disabled section header rows instead of `ListBox.Section`; this avoids HeroUI collection nesting errors seen earlier in this project:

```tsx
<ListBox>
  {engineOptionGroups.flatMap((group) => [
    <ListBox.Item
      key={group.i18nKey}
      id={group.i18nKey}
      textValue={t(`settings.translation.${group.i18nKey}`)}
      isDisabled
    >
      <span className="text-xs font-medium text-default-500">
        {t(`settings.translation.${group.i18nKey}`)}
      </span>
    </ListBox.Item>,
    ...group.options.map((opt) => (
      <ListBox.Item
        key={opt.value}
        id={opt.value}
        textValue={t(`settings.translation.${opt.i18nKey}`)}
      >
        {t(`settings.translation.${opt.i18nKey}`)}
        <ListBox.ItemIndicator />
      </ListBox.Item>
    )),
  ])}
</ListBox>
```

- [ ] **Step 3: Dynamic field rendering**

Only render endpoint item when `needsEndpoint`:

```tsx
{needsEndpoint && (
  <>
    <Separator />
    <SettingItem
      title={t("settings.translation.endpoint.title")}
      description={
        endpointOptional
          ? t("settings.translation.endpoint.optionalDesc")
          : t("settings.translation.endpoint.desc")
      }
    >
      <Input
        value={apiEndpoint}
        onChange={(e) => saveSetting("apiEndpoint", e.target.value)}
        placeholder={t(`settings.translation.endpointPlaceholder.${translationEngine}`, {
          defaultValue: t("settings.translation.endpointPlaceholder.default"),
        })}
        className="w-64"
      />
    </SettingItem>
  </>
)}
```

Add `apiSecret` and `apiRegion` from settings:

```typescript
const apiSecret = settings.apiSecret || "";
const apiRegion = settings.apiRegion || "";
```

Render `apiSecret` and `apiRegion` conditionally with `Input type="password"` for secret.

Only render model/prompt/wordDetailPrompt when `isCustomEngine`.

Always render timeout.

- [ ] **Step 4: Add zh-CN translation keys**

Modify `src/locales/zh-CN/common.json` under `settings.translation`:

```json
"engineGroupSystem": "系统内置",
"engineGroupVendor": "厂商 API",
"engineGroupCustom": "自定义",
"engineLibreTranslate": "LibreTranslate",
"engineMicrosoft": "Bing / Microsoft Translator",
"engineBaidu": "百度",
"engineTencent": "腾讯",
"engineYoudao": "有道",
"apiSecret": {
  "title": "API Secret",
  "desc": "厂商 API 的密钥或 Secret Key"
},
"apiSecretPlaceholder": "secret-xxx",
"apiRegion": {
  "title": "服务区域",
  "desc": "Microsoft 或腾讯云要求的区域"
},
"apiRegionPlaceholder": "例如：eastasia / ap-guangzhou",
"endpoint": {
  "title": "API地址",
  "desc": "翻译服务的接口地址",
  "optionalDesc": "不填写时使用该厂商默认接口地址"
},
"endpointPlaceholder": {
  "default": "https://api.example.com",
  "libretranslate": "http://localhost:5000",
  "google": "https://translation.googleapis.com/language/translate/v2",
  "deepl": "https://api-free.deepl.com/v2/translate",
  "microsoft": "https://api.cognitive.microsofttranslator.com/translate",
  "custom": "https://api.example.com/v1"
}
```

- [ ] **Step 5: Add en translation keys**

Modify `src/locales/en/common.json` under `settings.translation`:

```json
"engineGroupSystem": "Built-in",
"engineGroupVendor": "Vendor API",
"engineGroupCustom": "Custom",
"engineLibreTranslate": "LibreTranslate",
"engineMicrosoft": "Bing / Microsoft Translator",
"engineBaidu": "Baidu",
"engineTencent": "Tencent",
"engineYoudao": "Youdao",
"apiSecret": {
  "title": "API Secret",
  "desc": "Secret key required by the selected vendor"
},
"apiSecretPlaceholder": "secret-xxx",
"apiRegion": {
  "title": "Region",
  "desc": "Region required by Microsoft or Tencent Cloud"
},
"apiRegionPlaceholder": "Example: eastasia / ap-guangzhou",
"endpoint": {
  "title": "API Endpoint",
  "desc": "API endpoint URL for the translation service",
  "optionalDesc": "Leave empty to use the vendor default endpoint"
},
"endpointPlaceholder": {
  "default": "https://api.example.com",
  "libretranslate": "http://localhost:5000",
  "google": "https://translation.googleapis.com/language/translate/v2",
  "deepl": "https://api-free.deepl.com/v2/translate",
  "microsoft": "https://api.cognitive.microsofttranslator.com/translate",
  "custom": "https://api.example.com/v1"
}
```

- [ ] **Step 6: Add fallback keys to other locale files**

Apply the exact English values from Step 5 to these files:

```txt
src/locales/ar/common.json
src/locales/de/common.json
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
src/locales/zh-TW/common.json
```

For `zh-TW/common.json`, use Traditional Chinese values:

```json
"engineGroupSystem": "系統內建",
"engineGroupVendor": "廠商 API",
"engineGroupCustom": "自訂",
"engineLibreTranslate": "LibreTranslate",
"engineMicrosoft": "Bing / Microsoft Translator",
"engineBaidu": "百度",
"engineTencent": "騰訊",
"engineYoudao": "有道",
"apiSecret": {
  "title": "API Secret",
  "desc": "廠商 API 的密鑰或 Secret Key"
},
"apiSecretPlaceholder": "secret-xxx",
"apiRegion": {
  "title": "服務區域",
  "desc": "Microsoft 或騰訊雲要求的區域"
},
"apiRegionPlaceholder": "例如：eastasia / ap-guangzhou"
```

- [ ] **Step 7: Type check**

Run:

```bash
/Users/frank/.nvm/versions/node/v24.14.0/lib/node_modules/corepack/shims/pnpm tsc --noEmit
```

Expected: PASS。

- [ ] **Step 8: Commit**

```bash
git add src/pages/Settings/panels/TranslationPanel.tsx src/locales
git commit -m "feat: update translation engine settings UI"
```

---

## Task 11: Final verification and docs

**Files:**

- Modify: `docs/superpowers/specs/2026-05-20-local-dictionary-and-translation-engines-design.md` if implementation discovers a necessary design correction.

- [ ] **Step 1: Run frontend type check**

Run:

```bash
/Users/frank/.nvm/versions/node/v24.14.0/lib/node_modules/corepack/shims/pnpm tsc --noEmit
```

Expected: PASS.

- [ ] **Step 2: Run frontend build**

Run:

```bash
PATH=/Users/frank/.nvm/versions/node/v24.14.0/bin:$PATH /Users/frank/.nvm/versions/node/v24.14.0/lib/node_modules/corepack/shims/pnpm build
```

Expected: PASS. Use the nvm Node path to avoid Rollup native module signature issues seen under Codex App embedded Node.

- [ ] **Step 3: Run Rust build**

Run:

```bash
cd src-tauri && cargo build
```

Expected: PASS.

- [ ] **Step 4: Run Rust tests**

Run:

```bash
cd src-tauri && cargo test
```

Expected: PASS.

- [ ] **Step 5: Manual smoke checks**

Run app:

```bash
PATH=/Users/frank/.nvm/versions/node/v24.14.0/bin:$PATH /Users/frank/.nvm/versions/node/v24.14.0/lib/node_modules/corepack/shims/pnpm tauri dev
```

Check:

- 设置页翻译引擎显示 3 个分组。
- 选择 LibreTranslate 时只显示 Endpoint、可选 API Key、Timeout。
- 选择百度时显示 API Key 和 API Secret。
- 选择 Tencent 时显示 API Key、API Secret、Region。
- 选择 Custom 时显示 Endpoint、API Key、Model、Translation Prompt、Word Detail Prompt、Timeout。
- 未配置 Endpoint/Key 时错误信息明确且不泄露密钥。

- [ ] **Step 6: Commit verification-only doc updates**

Run this command only when the implementation changed the design contract documented in the spec:

```bash
git add docs/superpowers/specs/2026-05-20-local-dictionary-and-translation-engines-design.md
git commit -m "docs: align translation engine design with implementation"
```

When the spec file has no diff, record `git diff -- docs/superpowers/specs/2026-05-20-local-dictionary-and-translation-engines-design.md` output as empty in the task notes and do not create a docs commit.

---

## Implementation Notes

- 不要修改或暂存当前无关的 `src/pages/Home/index.tsx`。
- 不要把真实厂商 API Key 写入代码、测试或文档示例。
- 厂商 provider 的 live API 测试不进入默认测试套件，默认测试只做参数、签名、解析、错误脱敏。
- `WordInsightProvider` 第一版仍只支持 `custom`，其他翻译引擎返回 `UnsupportedEngine`。
- 第一版使用已解压的 StarDict ECDICT 三件套：`src-tauri/resources/stardict-ecdict/stardict-ecdict-2.4.2.ifo/.idx/.dict`。当前解压后约 190 MB，不接入 `ecdict-sqlite-*.zip`，也不支持 `.mdx`。
- StarDict 三件套作为本地构建资源处理；普通代码提交只提交 README、路径配置和加载逻辑。发布分发前需要通过 Git LFS、release asset 下载或安装包流水线保证三件套存在。

---

## References

- ECDICT releases: `https://github.com/skywind3000/ECDICT/releases`
- LibreTranslate docs: `https://docs.libretranslate.com/`
- DeepL text translation API: `https://developers.deepl.com/api-reference/translate`
- Google Cloud Translation API: `https://cloud.google.com/translate/docs/reference/rest/v2/translate`
- Microsoft Translator Text API: `https://learn.microsoft.com/azure/ai-services/translator/text-translation/reference/v3/translate`
- 百度翻译开放平台通用翻译 API: `https://api.fanyi.baidu.com/doc/21`
- 腾讯云机器翻译 TextTranslate: `https://cloud.tencent.com/document/product/551/15619`
- 有道智云自然语言翻译 API: `https://ai.youdao.com/DOCSIRMA/html/transapi/trans/api/wbfy/index.html`
