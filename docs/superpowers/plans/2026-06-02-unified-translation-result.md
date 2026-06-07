# Unified Translation Result Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将单词查询、划词弹窗、卡片和详情页统一到一个翻译结果数据结构，移除 `WordInsight` 独立详情链路，并把 `summary` 字段统一改名为 `translation`；划词未收录时只展示结果，用户点击“加入生词本”后才写入数据库。

**Architecture:** 后端只保留一条翻译/解析链路：`resolve_word` 先查 `words + word_details`，命中则返回已收录结果，未命中时调用 `translation_service.translate` 并返回同一个完整 `TranslationResult` 结构，但不自动写库。`add_word` 接收划词弹窗里的完整翻译结果，在一个事务中同时写入 `words` 和 `word_details`；类型边界只保留 `ResolvedWord`（查询展示结果）和 `AddWordInput`（保存输入），不再保留 `WordDetailInput`。自定义大模型通过同一个系统翻译提示词返回完整字段，厂商和本地引擎无法提供的字段返回空数组或空字符串。

**Tech Stack:** Tauri 2 + Rust + rusqlite + serde，React 19 + TypeScript + HeroUI v3 + Vitest，pnpm。

---

## File Structure

- Modify: `src-tauri/src/domain/models/word.rs`
  - `Word.summary` 改回 `Word.translation`。
  - 保留 `WordMeaning / EnglishDefinitionGroup / WordFormItem`。
  - 新增 `WORD_FORM_TYPES` 和 `is_valid_word_form_type`。
- Modify: `src-tauri/src/db/migrations.rs`
  - `words.summary` 改回 `words.translation`。
  - 继续保留 `word_details.meanings_json / english_definitions_json / examples_json / word_forms_json / memory_tip`。
- Modify: `src-tauri/src/ports/outbound/translation.rs`
  - 扩展 `TranslationResult` 为完整展示结构。
- Delete: `src-tauri/src/ports/outbound/word_insight.rs`
  - 删除 `WordInsightProvider / WordInsightRequest / GeneratedWordDetail`。
- Modify: `src-tauri/src/ports/outbound/mod.rs`
  - 移除 `pub mod word_insight;`。
- Delete: `src-tauri/src/adapters/outbound/translation/prompts/word_detail_prompt.txt`
  - 不再维护独立详情提示词。
- Modify: `src-tauri/src/adapters/outbound/translation/prompts/system_translation_prompt.txt`
  - 要求自定义大模型直接返回完整展示字段。
- Modify: `src-tauri/src/adapters/outbound/translation/custom.rs`
  - 移除 `WordInsightProvider` 实现。
  - `parse_translation_result` 直接解析完整字段。
- Modify: `src-tauri/src/domain/services/translation_service.rs`
  - 删除 `generate_word_detail` 和 `create_word_insight_provider`。
  - `translate` 保持唯一翻译入口。
- Modify: `src-tauri/src/commands/word_details.rs`
  - 删除 `generate_word_detail` 命令。
  - 删除 `WordDetailInput` 和 `save_word_detail`，详情不再通过独立命令保存。
  - 新增 `resolve_word` 命令，负责查库、调用翻译、返回完整展示结果；未命中时不保存。
- Modify: `src-tauri/src/commands/words.rs`
  - `add_word` 改为接收完整翻译结果，并同时写入 `words` 和 `word_details`。
- Modify: `src-tauri/src/commands/translate.rs`
  - 保留普通 `translate_text`，返回同一个扩展后的 `TranslationResult`。
- Modify: `src-tauri/src/lib.rs`
  - 移除 `generate_word_detail` 注册。
  - 注册 `resolve_word`。
- Modify: `src-tauri/src/db/mod.rs`
  - 删除默认设置 `wordDetailPrompt`。
- Modify: `src/pages/Settings/panels/TranslationPanel.tsx`
  - 删除详情提示词设置项，只保留统一翻译提示词。
- Modify: `src/lib/api/translate.ts`
  - 扩展前端 `TranslationResult`。
- Modify: `src/lib/api/word.ts`
  - `Word.summary` 改回 `Word.translation`。
- Modify: `src/lib/api/wordDetails.ts`
  - 删除 `generateWordDetail`。
  - 新增 `resolveWord(text: string): Promise<ResolvedWord>`。
- Modify: `src/lib/api/word.ts`
  - `addWord` 改为接收划词结果，保存 `words + word_details`。
- Modify: `src/lib/api/index.ts`
  - 更新导出。
- Modify: `src/pages/SelectionPopup/index.tsx`
  - 划词文本变化时调用 `resolveWord`。
  - 未收录时展示“加入生词本”按钮，点击后调用 `addWord` 保存。
- Modify: `src/pages/SelectionPopup/SelectionText.tsx`
  - 展示 `translation / phonetic / meanings / examples` 和保存状态。
- Modify: `src/pages/Home/components/WordGrid.tsx`
- Modify: `src/pages/Home/components/WordList.tsx`
- Modify: `src/pages/Home/components/DetailPanel.tsx`
  - 全部从 `summary` 改回 `translation`。

---

## Task 1: 统一领域模型和数据库命名

**Files:**
- Modify: `src-tauri/src/domain/models/word.rs`
- Modify: `src-tauri/src/domain/models/mod.rs`
- Modify: `src-tauri/src/db/migrations.rs`
- Modify: `src-tauri/src/adapters/outbound/sqlite.rs`
- Modify: `src-tauri/src/domain/services/word_service.rs`
- Modify: `src-tauri/src/commands/words.rs`

- [ ] **Step 1: Rename `Word.summary` to `Word.translation`**

In `src-tauri/src/domain/models/word.rs`, replace the field:

```rust
pub summary: String,
```

with:

```rust
pub translation: String,
```

Update `Word::new` signature:

```rust
pub fn new(
    id: String,
    word: String,
    translation: String,
    source_lang: String,
    target_lang: String,
) -> Self
```

Set the field:

```rust
translation,
```

- [ ] **Step 2: Add word form type validation**

Add near `WordFormItem`:

```rust
pub const WORD_FORM_TYPES: &[&str] = &[
    "lemma",
    "lemma_variant",
    "past_tense",
    "past_participle",
    "present_participle",
    "third_person_singular",
    "comparative",
    "superlative",
    "plural",
];

pub fn is_valid_word_form_type(value: &str) -> bool {
    WORD_FORM_TYPES.contains(&value)
}
```

Export it from `src-tauri/src/domain/models/mod.rs`:

```rust
pub use word::{
    is_valid_word_form_type, normalize_lang, normalize_word, EnglishDefinitionGroup, Word,
    WordFormItem, WordMeaning, WORD_FORM_TYPES,
};
```

- [ ] **Step 3: Update migrations to use `translation`**

In `src-tauri/src/db/migrations.rs`, change the `words` schema column:

```sql
translation TEXT NOT NULL DEFAULT '',
```

and remove `summary`.

Update rebuild detection:

```rust
Ok(!table_has_column(conn, "words", "normalized_word")?
    || !table_has_column(conn, "words", "translation")?
    || table_has_column(conn, "words", "summary")?
    || !table_has_column(conn, "word_details", "meanings_json")?
    || !table_has_column(conn, "word_details", "english_definitions_json")?
    || !table_has_column(conn, "word_details", "word_forms_json")?)
```

Update migration tests to assert:

```rust
assert!(table_has_column(&conn, "words", "translation").unwrap());
assert!(!table_has_column(&conn, "words", "summary").unwrap());
```

- [ ] **Step 4: Update SQLite repository SQL**

In `src-tauri/src/adapters/outbound/sqlite.rs`:

Use `translation` in row mapping:

```rust
translation: row.get("translation")?,
```

Use `translation` in insert:

```sql
INSERT INTO words (..., translation, ...)
```

Use `translation` in search:

```sql
WHERE w.word LIKE ?1 OR w.translation LIKE ?1
```

Use `translation` in update:

```sql
UPDATE words SET word=?2, normalized_word=?3, translation=?4, ...
```

- [ ] **Step 5: Update word service and command DTO**

In `src-tauri/src/domain/services/word_service.rs`, use `translation` as parameter and update field:

```rust
pub fn add_word(
    &self,
    word: String,
    translation: String,
    source_lang: String,
    target_lang: String,
    tags: String,
) -> Result<Word, String>
```

`WordUpdate` should be:

```rust
#[derive(Debug, Default, serde::Deserialize)]
pub struct WordUpdate {
    #[serde(default)]
    pub translation: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}
```

In `update_word`, set:

```rust
if let Some(translation) = updates.translation {
    word.translation = translation;
}
```

- [ ] **Step 6: Run focused backend check**

Run:

```bash
cd src-tauri && cargo check
```

Expected: remaining compile failures point to translation result and word detail command areas, not to `summary` in word model or repository.

---

## Task 2: 扩展统一 `TranslationResult`

**Files:**
- Modify: `src-tauri/src/ports/outbound/translation.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/http_utils.rs`
- Modify: provider files under `src-tauri/src/adapters/outbound/translation/`
- Modify: dictionary adapter if needed: `src-tauri/src/adapters/outbound/dictionary/stardict_ecdict.rs`

- [ ] **Step 1: Update Rust `TranslationResult`**

In `src-tauri/src/ports/outbound/translation.rs`, import model types:

```rust
use crate::domain::models::{EnglishDefinitionGroup, WordFormItem, WordMeaning};
```

Replace `TranslationResult` with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationResult {
    pub translation: String,
    pub detected_source_lang: Option<String>,
    pub phonetic: Option<String>,
    pub meanings: Vec<WordMeaning>,
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    pub examples: Vec<TranslationExample>,
    pub word_forms: Vec<WordFormItem>,
    pub memory_tip: String,
}
```

- [ ] **Step 2: Update empty result helper**

In `src-tauri/src/adapters/outbound/translation/http_utils.rs`, update `empty_translation_result`:

```rust
pub(crate) fn empty_translation_result(
    translation: String,
    detected_source_lang: Option<String>,
) -> TranslationResult {
    TranslationResult {
        translation,
        detected_source_lang,
        phonetic: None,
        meanings: Vec::new(),
        english_definitions: Vec::new(),
        examples: Vec::new(),
        word_forms: Vec::new(),
        memory_tip: String::new(),
    }
}
```

- [ ] **Step 3: Update providers to compile**

For provider code that currently sets `part_of_speech` or `definitions`, replace those fields with:

```rust
meanings: Vec::new(),
english_definitions: Vec::new(),
word_forms: Vec::new(),
memory_tip: String::new(),
```

Keep `examples` if the provider already returns examples. Most vendor providers will return empty arrays through `empty_translation_result`.

- [ ] **Step 4: Run provider tests**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation ports::outbound::dictionary
```

Expected: PASS after updating assertions from `part_of_speech/definitions` to `meanings/english_definitions`.

---

## Task 3: 移除 WordInsight 和独立详情提示词

**Files:**
- Delete: `src-tauri/src/ports/outbound/word_insight.rs`
- Modify: `src-tauri/src/ports/outbound/mod.rs`
- Delete: `src-tauri/src/adapters/outbound/translation/prompts/word_detail_prompt.txt`
- Modify: `src-tauri/src/adapters/outbound/translation/custom.rs`
- Modify: `src-tauri/src/domain/services/translation_service.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src/pages/Settings/panels/TranslationPanel.tsx`

- [ ] **Step 1: Remove the port module**

Delete `src-tauri/src/ports/outbound/word_insight.rs`.

In `src-tauri/src/ports/outbound/mod.rs`, remove:

```rust
pub mod word_insight;
```

- [ ] **Step 2: Remove detail prompt file and include**

Delete `src-tauri/src/adapters/outbound/translation/prompts/word_detail_prompt.txt`.

In `custom.rs`, remove:

```rust
const WORD_DETAIL_PROMPT: &str = include_str!("prompts/word_detail_prompt.txt");
```

Remove:

```rust
fn build_word_detail_system_prompt(&self) -> String
```

Remove `impl WordInsightProvider for CustomTranslationProvider`.

- [ ] **Step 3: Remove service detail path**

In `src-tauri/src/domain/services/translation_service.rs`, remove:

```rust
use crate::ports::outbound::word_insight::{
    GeneratedWordDetail, WordInsightProvider, WordInsightRequest,
};
```

Remove methods:

```rust
pub async fn generate_word_detail(...)
fn create_word_insight_provider(...)
```

- [ ] **Step 4: Remove setting for split detail prompt**

In `src-tauri/src/db/mod.rs`, remove the default:

```rust
("wordDetailPrompt".to_string(), "".to_string()),
```

In `src/pages/Settings/panels/TranslationPanel.tsx`, remove the `wordDetailPrompt` setting item and textarea. Keep the unified `translationPrompt`.

- [ ] **Step 5: Run compile**

Run:

```bash
cd src-tauri && cargo check
pnpm exec tsc --noEmit
```

Expected: remaining failures should only be from custom parser and word detail command changes.

---

## Task 4: 自定义大模型直接返回完整统一结果

**Files:**
- Modify: `src-tauri/src/adapters/outbound/translation/prompts/system_translation_prompt.txt`
- Modify: `src-tauri/src/adapters/outbound/translation/custom.rs`

- [ ] **Step 1: Update unified system prompt**

Replace `system_translation_prompt.txt` with:

```text
你是一个专业的翻译、词典、例句、词形变化和记忆助手。请根据用户给出的文本、来源语言和目标语言返回统一翻译结果。

你必须只返回 JSON，不要使用 Markdown，不要添加解释。JSON 字段固定如下：
{
  "translation": "string",
  "detectedSourceLang": "string | null",
  "phonetic": "string | null",
  "meanings": [
    {
      "partOfSpeech": "string",
      "translations": ["string"]
    }
  ],
  "englishDefinitions": [
    {
      "partOfSpeech": "string",
      "definitions": ["string"]
    }
  ],
  "examples": [
    {
      "sentence": "string",
      "translation": "string"
    }
  ],
  "wordForms": [
    {
      "type": "lemma | lemma_variant | past_tense | past_participle | present_participle | third_person_singular | comparative | superlative | plural",
      "words": ["string"]
    }
  ],
  "memoryTip": "string"
}

要求：
- translation 用目标语言返回核心翻译；短词返回 1 到 3 个核心释义，短语或句子返回自然译文。
- meanings 至少返回 1 项；每个 partOfSpeech 对应自己的 translations。
- englishDefinitions 返回英文定义；如果没有可靠英文定义，返回空数组。
- examples 返回 2 到 4 条自然例句，sentence 使用来源语言，translation 使用目标语言。
- wordForms 只返回能确定的词形变化；type 必须使用上面列出的完整类型名，不要使用简写。
- memoryTip 用目标语言写；如果文本不是适合记忆的单词或短语，返回空字符串。
```

- [ ] **Step 2: Update `TranslationResponse`**

In `custom.rs`, replace `TranslationResponse` with:

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslationResponse {
    translation: String,
    #[serde(default, alias = "detected_source_lang")]
    detected_source_lang: Option<String>,
    #[serde(default)]
    phonetic: Option<String>,
    meanings: Vec<WordMeaning>,
    #[serde(default, alias = "english_definitions")]
    english_definitions: Vec<EnglishDefinitionGroup>,
    examples: Vec<TranslationExample>,
    #[serde(default, alias = "word_forms")]
    word_forms: Vec<WordFormItem>,
    #[serde(default)]
    memory_tip: String,
}
```

Add imports:

```rust
use crate::domain::models::{
    is_valid_word_form_type, EnglishDefinitionGroup, WordFormItem, WordMeaning,
};
```

- [ ] **Step 3: Update `translate_inner` user prompt**

Replace the JSON shape inside `translate_inner` user prompt with the same fields as the system prompt:

```rust
let user_prompt = format!(
    "请将以下文本从 {source_lang} 翻译为 {target_lang}，并返回统一 JSON 结构。\n\n文本：{text}",
    source_lang = request.source_lang,
    target_lang = request.target_lang,
    text = request.text
);
```

- [ ] **Step 4: Update `parse_translation_result` validation**

Add helper:

```rust
fn validate_translation_result(result: &TranslationResult) -> Result<(), TranslationError> {
    if result.translation.trim().is_empty() || result.meanings.is_empty() || result.examples.is_empty() {
        return Err(TranslationError::InvalidJson);
    }

    for meaning in &result.meanings {
        if meaning.part_of_speech.trim().is_empty()
            || meaning.translations.is_empty()
            || meaning.translations.iter().any(|value| value.trim().is_empty())
        {
            return Err(TranslationError::InvalidJson);
        }
    }

    for example in &result.examples {
        if example.sentence.trim().is_empty() || example.translation.trim().is_empty() {
            return Err(TranslationError::InvalidJson);
        }
    }

    for form in &result.word_forms {
        if !is_valid_word_form_type(&form.r#type)
            || form.words.is_empty()
            || form.words.iter().any(|value| value.trim().is_empty())
        {
            return Err(TranslationError::InvalidJson);
        }
    }

    Ok(())
}
```

Update `parse_translation_result`:

```rust
fn parse_translation_result(content: &str) -> Result<TranslationResult, TranslationError> {
    let json_str = extract_json(content);
    let parsed = serde_json::from_str::<TranslationResponse>(json_str)
        .map_err(|_| TranslationError::InvalidJson)?;
    let result = TranslationResult {
        translation: parsed.translation,
        detected_source_lang: parsed.detected_source_lang,
        phonetic: parsed.phonetic,
        meanings: parsed.meanings,
        english_definitions: parsed.english_definitions,
        examples: parsed.examples,
        word_forms: parsed.word_forms,
        memory_tip: parsed.memory_tip,
    };

    validate_translation_result(&result)?;
    Ok(result)
}
```

- [ ] **Step 5: Update custom parser tests**

Update `parse_translation_result_accepts_valid_json` to expect full fields:

```rust
let content = r#"{
    "translation": "破产的；身无分文的",
    "detectedSourceLang": "en",
    "phonetic": "broʊk",
    "meanings": [
        {"partOfSpeech": "adj", "translations": ["破产的", "身无分文的"]}
    ],
    "englishDefinitions": [
        {"partOfSpeech": "adj", "definitions": ["having no money"]}
    ],
    "examples": [
        {"sentence": "He went broke.", "translation": "他破产了。"}
    ],
    "wordForms": [
        {"type": "lemma", "words": ["break"]}
    ],
    "memoryTip": "broke 可以联想到 break。"
}"#;

let result = parse_translation_result(content).unwrap();

assert_eq!(result.translation, "破产的；身无分文的");
assert_eq!(result.meanings[0].part_of_speech, "adj");
assert_eq!(result.english_definitions[0].definitions[0], "having no money");
assert_eq!(result.word_forms[0].r#type, "lemma");
```

Add rejection tests for empty `meanings`, empty `examples`, and invalid `wordForms.type`.

- [ ] **Step 6: Run custom tests**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::custom
```

Expected: PASS.

---

## Task 5: 新增统一 `resolve_word` 查询路径和 `add_word` 保存路径

**Files:**
- Modify: `src-tauri/src/commands/word_details.rs`
- Modify: `src-tauri/src/commands/words.rs`
- Modify: `src-tauri/src/commands/translate.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update `WordDetail` DTO, remove `WordDetailInput`, and add popup `ResolvedWord` DTO**

In `word_details.rs`, change:

```rust
pub summary: String,
```

to:

```rust
pub translation: String,
```

Remove the old independent detail input and save command:

```rust
pub struct WordDetailInput { ... }

#[tauri::command]
pub fn save_word_detail(...)
```

`word_details.rs` should no longer expose a separate detail-save DTO. The only save input for selected words is `AddWordInput` in `words.rs`.

Update SQL reads:

```sql
w.translation,
```

Add a DTO for popup lookup results that may not be saved yet:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedWord {
    pub word_id: Option<String>,
    pub word: String,
    pub translation: String,
    pub detected_source_lang: Option<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub phonetic: Option<String>,
    pub meanings: Vec<WordMeaning>,
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    pub examples: Vec<TranslationExample>,
    pub word_forms: Vec<WordFormItem>,
    pub memory_tip: String,
}
```

- [ ] **Step 2: Convert saved and unsaved results to `ResolvedWord`**

Add helpers:

```rust
fn word_detail_to_resolved(detail: WordDetail, source_lang: String, target_lang: String) -> ResolvedWord {
    ResolvedWord {
        word_id: Some(detail.word_id),
        word: detail.word,
        translation: detail.translation,
        detected_source_lang: Some(source_lang.clone()),
        source_lang,
        target_lang,
        phonetic: detail.phonetic,
        meanings: detail.meanings,
        english_definitions: detail.english_definitions,
        examples: detail.examples,
        word_forms: detail.word_forms,
        memory_tip: detail.memory_tip,
    }
}

fn translation_result_to_resolved(
    word: String,
    source_lang: String,
    target_lang: String,
    result: TranslationResult,
) -> ResolvedWord {
    ResolvedWord {
        word_id: None,
        word,
        translation: result.translation,
        detected_source_lang: result.detected_source_lang,
        source_lang,
        target_lang,
        phonetic: result.phonetic,
        meanings: result.meanings,
        english_definitions: result.english_definitions,
        examples: result.examples,
        word_forms: result.word_forms,
        memory_tip: result.memory_tip,
    }
}
```

- [ ] **Step 3: Add `resolve_word` command that does not save misses**

Add:

```rust
#[tauri::command]
pub async fn resolve_word(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<ResolvedWord, String> {
    let app_state = state.inner();
    let query = text.trim().to_string();
    if query.is_empty() {
        return Err(TranslationError::EmptyText.to_string());
    }

    let settings = app_state.settings_cache_read()?;
    let source_setting = settings_value(&settings, "sourceLanguage", "auto");
    let target_lang = settings_value(&settings, "targetLanguage", "zh");
    let source_lookup = if source_setting == "auto" {
        None
    } else {
        Some(source_setting.as_str())
    };

    if let Some(existing) = app_state
        .word_service
        .find_existing_word(&query, source_lookup, &target_lang)?
    {
        if let Some(detail) = get_word_detail_by_id(app_state, &existing.id)? {
            return Ok(word_detail_to_resolved(
                detail,
                existing.source_lang,
                existing.target_lang,
            ));
        }
    }

    let result = app_state
        .translation_service
        .translate(settings, query.clone())
        .await?;
    let resolved_source_lang = result
        .detected_source_lang
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if source_setting == "auto" {
                "en".to_string()
            } else {
                source_setting.clone()
            }
        });

    Ok(translation_result_to_resolved(
        query,
        resolved_source_lang,
        target_lang,
        result,
    ))
}
```

Add helper:

```rust
fn settings_value(
    settings: &std::collections::HashMap<String, String>,
    key: &str,
    default: &str,
) -> String {
    settings
        .get(key)
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}
```

- [ ] **Step 4: Add `AddWordInput` and `add_word` command that writes both tables in one transaction**

In `src-tauri/src/commands/words.rs`, replace the old scalar command parameters with:

```rust
use crate::commands::word_details::WordDetail;
use crate::domain::models::{EnglishDefinitionGroup, WordFormItem, WordMeaning};
use crate::ports::outbound::translation::TranslationExample;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWordInput {
    pub word: String,
    pub translation: String,
    pub source_lang: String,
    pub target_lang: String,
    #[serde(default)]
    pub phonetic: Option<String>,
    #[serde(default)]
    pub meanings: Vec<WordMeaning>,
    #[serde(default)]
    pub english_definitions: Vec<EnglishDefinitionGroup>,
    #[serde(default)]
    pub examples: Vec<TranslationExample>,
    #[serde(default)]
    pub word_forms: Vec<WordFormItem>,
    #[serde(default)]
    pub memory_tip: String,
    #[serde(default)]
    pub tags: String,
}
```

Implement `add_word` so it writes `words` and `word_details` in the same SQLite transaction:

```rust
#[tauri::command]
pub fn add_word(
    state: State<'_, AppState>,
    input: AddWordInput,
) -> Result<WordDetail, String> {
    use crate::domain::models::{normalize_lang, normalize_word};
    use chrono::Utc;
    use rusqlite::params;

    let word_text = input.word.trim().to_string();
    if word_text.is_empty() {
        return Err("单词不能为空".to_string());
    }

    let source_lang = normalize_lang(&input.source_lang);
    let target_lang = normalize_lang(&input.target_lang);
    let normalized_word = normalize_word(&word_text);

    if let Some(existing) = state.word_service.find_existing_word(
        &word_text,
        Some(&source_lang),
        &target_lang,
    )? {
        if let Some(detail) = crate::commands::word_details::get_word_detail_by_id(state.inner(), &existing.id)? {
            return Ok(detail);
        }
    }

    let word_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    let meanings_json = serde_json::to_string(&input.meanings).map_err(|error| error.to_string())?;
    let english_definitions_json =
        serde_json::to_string(&input.english_definitions).map_err(|error| error.to_string())?;
    let examples_json = serde_json::to_string(&input.examples).map_err(|error| error.to_string())?;
    let word_forms_json =
        serde_json::to_string(&input.word_forms).map_err(|error| error.to_string())?;

    {
        let mut conn = state.db.connection();
        let tx = conn.transaction().map_err(|error| error.to_string())?;

        tx.execute(
            "INSERT INTO words
                (id, word, normalized_word, source_lang, target_lang, translation, phonetic,
                 status, tags, ease_factor, interval, repetitions, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'new', ?8, 2.5, 0, 0, ?9, ?10)",
            params![
                &word_id,
                &word_text,
                &normalized_word,
                &source_lang,
                &target_lang,
                &input.translation,
                &input.phonetic,
                &input.tags,
                now,
                now,
            ],
        )
        .map_err(|error| error.to_string())?;

        tx.execute(
            "INSERT INTO word_details
                (word_id, meanings_json, english_definitions_json, examples_json,
                 word_forms_json, memory_tip, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &word_id,
                &meanings_json,
                &english_definitions_json,
                &examples_json,
                &word_forms_json,
                &input.memory_tip,
                now,
                now,
            ],
        )
        .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())?;
    }

    crate::commands::word_details::get_word_detail_by_id(state.inner(), &word_id)?
        .ok_or_else(|| "单词保存失败".to_string())
}
```

- [ ] **Step 5: Remove old detail generation and detail save commands**

Remove:

```rust
pub async fn generate_word_detail(...)
pub fn save_word_detail(...)
pub struct WordDetailInput { ... }
```

Remove their registrations in `src-tauri/src/lib.rs`.

Register:

```rust
commands::word_details::resolve_word,
```

- [ ] **Step 6: Update `translate_text` cached result mapping**

In `src-tauri/src/commands/translate.rs`, update cached DB result to fill the expanded `TranslationResult`:

```rust
Ok(Some(TranslationResult {
    translation: word.translation,
    detected_source_lang: Some(word.source_lang),
    phonetic: word.phonetic,
    meanings: word.meanings,
    english_definitions,
    examples,
    word_forms,
    memory_tip,
}))
```

Read `english_definitions`, `word_forms`, and `memory_tip` from `get_word_detail_by_id`.

- [ ] **Step 7: Run backend tests**

Run:

```bash
cd src-tauri && cargo test commands::word_details commands::translate
```

Expected: PASS.

---

## Task 6: 更新前端统一类型和页面消费

**Files:**
- Modify: `src/lib/api/translate.ts`
- Modify: `src/lib/api/word.ts`
- Modify: `src/lib/api/wordDetails.ts`
- Modify: `src/lib/api/index.ts`
- Modify: `src/pages/Home/components/WordGrid.tsx`
- Modify: `src/pages/Home/components/WordList.tsx`
- Modify: `src/pages/Home/components/DetailPanel.tsx`
- Modify: `src/pages/SelectionPopup/index.tsx`
- Modify: `src/pages/SelectionPopup/SelectionText.tsx`

- [ ] **Step 1: Update frontend `TranslationResult`**

In `src/lib/api/translate.ts`, use:

```ts
import type {
  EnglishDefinitionGroup,
  WordFormItem,
} from "./wordDetails";
import type { WordMeaning } from "./word";

export interface TranslationResult {
  translation: string;
  detectedSourceLang: string | null;
  phonetic: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
}
```

Update `RustTranslationResult` to use snake-case backend fields:

```ts
interface RustTranslationResult {
  translation: string;
  detected_source_lang: string | null;
  phonetic: string | null;
  meanings: WordMeaning[];
  english_definitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  word_forms: WordFormItem[];
  memory_tip: string;
}
```

Map arrays defensively.

- [ ] **Step 2: Rename frontend `Word.summary` to `Word.translation`**

In `src/lib/api/word.ts`, replace:

```ts
summary: string;
```

with:

```ts
translation: string;
```

`WordUpdate` should use:

```ts
translation?: string;
```

Replace the old scalar `addWord` parameters with a single input object:

```ts
export interface AddWordInput {
  word: string;
  translation: string;
  sourceLang: string;
  targetLang: string;
  phonetic?: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
  tags?: string;
}

export async function addWord(input: AddWordInput): Promise<WordDetail> {
  return invoke("add_word", { input });
}
```

This API is used by the selection popup after the user clicks “加入生词本”.

- [ ] **Step 3: Update `wordDetails.ts`**

Remove:

```ts
generateWordDetail
```

Add a `ResolvedWord` type for popup lookup results:

```ts
export interface ResolvedWord {
  wordId: string | null;
  word: string;
  translation: string;
  detectedSourceLang: string | null;
  sourceLang: string;
  targetLang: string;
  phonetic: string | null;
  meanings: WordMeaning[];
  englishDefinitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  wordForms: WordFormItem[];
  memoryTip: string;
}
```

Add the backend response type and mapper:

```ts
interface RustResolvedWord {
  word_id: string | null;
  word: string;
  translation: string;
  detected_source_lang: string | null;
  source_lang: string;
  target_lang: string;
  phonetic: string | null;
  meanings: WordMeaning[];
  english_definitions: EnglishDefinitionGroup[];
  examples: TranslationExample[];
  word_forms: WordFormItem[];
  memory_tip: string;
}

function toResolvedWord(result: RustResolvedWord): ResolvedWord {
  return {
    wordId: result.word_id,
    word: result.word,
    translation: result.translation,
    detectedSourceLang: result.detected_source_lang,
    sourceLang: result.source_lang,
    targetLang: result.target_lang,
    phonetic: result.phonetic,
    meanings: Array.isArray(result.meanings) ? result.meanings : [],
    englishDefinitions: Array.isArray(result.english_definitions)
      ? result.english_definitions
      : [],
    examples: Array.isArray(result.examples) ? result.examples : [],
    wordForms: Array.isArray(result.word_forms) ? result.word_forms : [],
    memoryTip: result.memory_tip,
  };
}
```

Add:

```ts
export async function resolveWord(text: string): Promise<ResolvedWord> {
  const result = await invoke<RustResolvedWord>("resolve_word", { text });
  return toResolvedWord(result);
}
```

Change `WordDetail.summary` to:

```ts
translation: string;
```

Map backend `translation`.

- [ ] **Step 4: Update page components**

In `WordGrid.tsx`, `WordList.tsx`, and `DetailPanel.tsx`, replace `word.summary` with `word.translation`.

In `DetailPanel.tsx`, remove `generateWordDetail` import and generation button. Detail pages should read saved detail via `getWordDetail`; missing detail can show an empty state because unsaved popup results do not enter the Home card/detail flow until the user clicks “加入生词本”.

- [ ] **Step 5: Update selection popup**

In `SelectionPopup/index.tsx`, import:

```ts
import { addWord, resolveWord, type ResolvedWord } from "../../lib/api";
```

Call `resolveWord(text)` whenever the selected text is initialized or updated. Store the result in:

```ts
const [resolvedWord, setResolvedWord] = useState<ResolvedWord | null>(null);
const [isSavingWord, setIsSavingWord] = useState(false);
```

Add save handler:

```ts
const handleAddWord = useCallback(async () => {
  if (!resolvedWord || resolvedWord.wordId) {
    return;
  }

  setIsSavingWord(true);
  try {
    const saved = await addWord({
      word: resolvedWord.word,
      translation: resolvedWord.translation,
      sourceLang: resolvedWord.sourceLang,
      targetLang: resolvedWord.targetLang,
      phonetic: resolvedWord.phonetic,
      meanings: resolvedWord.meanings,
      englishDefinitions: resolvedWord.englishDefinitions,
      examples: resolvedWord.examples,
      wordForms: resolvedWord.wordForms,
      memoryTip: resolvedWord.memoryTip,
    });
    setResolvedWord({
      wordId: saved.wordId,
      word: saved.word,
      translation: saved.translation,
      detectedSourceLang: resolvedWord.detectedSourceLang,
      sourceLang: resolvedWord.sourceLang,
      targetLang: resolvedWord.targetLang,
      phonetic: saved.phonetic,
      meanings: saved.meanings,
      englishDefinitions: saved.englishDefinitions,
      examples: saved.examples,
      wordForms: saved.wordForms,
      memoryTip: saved.memoryTip,
    });
  } finally {
    setIsSavingWord(false);
  }
}, [resolvedWord]);
```

In `SelectionText.tsx`, render:

```tsx
resolvedWord.translation
resolvedWord.meanings
resolvedWord.examples.slice(0, 2)
```

Show a button only when the result is not saved:

```tsx
{resolvedWord && !resolvedWord.wordId && (
  <Button isPending={isSavingWord} onPress={onAddWord}>
    加入生词本
  </Button>
)}
{resolvedWord?.wordId && (
  <span className="text-xs text-success">已加入生词本</span>
)}
```

- [ ] **Step 6: Run frontend compile**

Run:

```bash
pnpm exec tsc --noEmit
```

Expected: PASS.

---

## Task 7: 更新测试

**Files:**
- Modify: `src/lib/api/__test__/translate.test.ts`
- Create or Modify: `src/lib/api/__test__/wordDetails.test.ts`
- Modify: `src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx`
- Modify Rust tests in `src-tauri/src/adapters/outbound/translation/custom.rs`
- Modify migration tests in `src-tauri/src/db/migrations.rs`

- [ ] **Step 1: Update API translate test**

In `src/lib/api/__test__/translate.test.ts`, expect:

```ts
{
  translation: "你好，世界",
  detectedSourceLang: "en",
  phonetic: null,
  meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
  englishDefinitions: [],
  examples: [{ sentence: "Hello world", translation: "你好，世界" }],
  wordForms: [],
  memoryTip: "",
}
```

- [ ] **Step 2: Add `resolveWord` and `addWord` API tests**

Create or update `src/lib/api/__test__/wordDetails.test.ts` to assert `resolve_word` maps backend `word_id`, `translation`, `meanings`, `english_definitions`, `examples`, `word_forms`, and `memory_tip` into frontend fields. The frontend should treat `wordId !== null` as saved and `wordId === null` as unsaved.

Create or update `src/lib/api/__test__/word.test.ts` to assert `addWord` sends one `input` object containing `word`, `translation`, `sourceLang`, `targetLang`, `meanings`, `englishDefinitions`, `examples`, `wordForms`, and `memoryTip`.

- [ ] **Step 3: Update popup tests**

Mock `resolve_word`:

```ts
if (command === "resolve_word") {
  return Promise.resolve({
    word_id: null,
    word: "hello",
    translation: "你好",
    detected_source_lang: "en",
    source_lang: "en",
    target_lang: "zh",
    phonetic: null,
    meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
    english_definitions: [],
    examples: [{ sentence: "Hello there.", translation: "你好。" }],
    word_forms: [],
    memory_tip: "hello 是问候语。",
  });
}
if (command === "add_word") {
  return Promise.resolve({
    word_id: "word-1",
    word: "hello",
    translation: "你好",
    phonetic: null,
    meanings: [{ partOfSpeech: "interjection", translations: ["你好"] }],
    english_definitions: [],
    examples: [{ sentence: "Hello there.", translation: "你好。" }],
    word_forms: [],
    memory_tip: "hello 是问候语。",
    created_at: 1,
    updated_at: 1,
  });
}
```

Assert popup renders:

```ts
expect(screen.getByText("你好")).toBeTruthy();
expect(screen.getByText("interjection")).toBeTruthy();
expect(screen.getByText("Hello there.")).toBeTruthy();
expect(screen.getByRole("button", { name: "加入生词本" })).toBeTruthy();
```

Add save assertion:

```ts
await userEvent.click(screen.getByRole("button", { name: "加入生词本" }));

expect(invokeMock).toHaveBeenCalledWith("add_word", {
  input: expect.objectContaining({
    word: "hello",
    translation: "你好",
    sourceLang: "en",
    targetLang: "zh",
  }),
});
expect(screen.getByText("已加入生词本")).toBeTruthy();
```

- [ ] **Step 4: Run focused tests**

Run:

```bash
cd src-tauri && cargo test adapters::outbound::translation::custom db::migrations
pnpm test -- src/lib/api/__test__/translate.test.ts
pnpm test -- src/lib/api/__test__/wordDetails.test.ts
pnpm test -- src/pages/SelectionPopup/__test__/SelectionPopup.test.tsx
```

Expected: PASS. If frontend single-file Vitest shows duplicate React or unrelated worktree paths, inspect the path leakage before changing code.

---

## Task 8: Full verification

**Files:**
- No new files.

- [ ] **Step 1: Run Rust full test suite**

Run:

```bash
cd src-tauri && cargo test
```

Expected: PASS. Existing dead-code warning is acceptable only if it is unrelated to this feature.

- [ ] **Step 2: Run frontend build**

Run:

```bash
pnpm build
```

Expected: PASS. Existing Vite chunk-size warning is acceptable.

- [ ] **Step 3: Manual smoke test**

Run:

```bash
pnpm tauri dev
```

Manual checks:
- Select a word and confirm the popup shows translation, phonetic if present, part-of-speech meanings, and examples.
- Confirm an unsaved popup result shows “加入生词本” and does not appear in Home cards before saving.
- Click “加入生词本” and confirm the word appears in Home cards with the same translation and meanings.
- Select the same word again after saving and confirm it returns from the DB-backed path with “已加入生词本”.
- Open the Home card/detail page and confirm it uses the same `translation + word_details` data.
- Confirm settings page no longer shows a separate word detail prompt field.

---

## Self-Review

- Spec coverage: The plan implements the updated request: no `WordInsight`, no split `word_detail_prompt`, all result fields in one structure, `summary` renamed to `translation`, and unsaved popup results are saved only when the user clicks “加入生词本”.
- Placeholder scan: No placeholder markers or unspecified implementation steps are present.
- Type consistency: Backend and frontend both use `translation / meanings / englishDefinitions / examples / wordForms / memoryTip`.
- Scope control: Other engines share the unified result type but only custom LLM is expected to fill rich fields. Vendor engines return empty rich fields unless they already provide them.
