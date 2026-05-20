# Translation Engine Categorization Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge OpenAI into the "Custom (LLM)" translation engine category, making engine selection more accurate (dedicated APIs vs LLM prompt translation).

**Architecture:** Replace `OpenAiTranslationProvider` type alias in `custom.rs` with a full implementation (content from `openai.rs`, struct renamed to `CustomTranslationProvider`). Remove `openai.rs` module. Update dispatch logic in `translate.rs` to only match `"custom"` (with `"openai"` compatibility mapping). Change all defaults from OpenAI-specific values to empty strings. Frontend removes OpenAI engine option and maps saved `"openai"` values to `"custom"`. i18n removes `engineOpenAI` key across 14 locale files and updates `engineCustom` label to include "(LLM)".

**Tech Stack:** Rust (Tauri), React 19 + TypeScript, HeroUI v3

---

### Task 1: Rust — Migrate openai.rs into custom.rs and rename struct

**Files:**
- Modify: `src-tauri/src/adapters/outbound/translation/custom.rs`
- Delete: `src-tauri/src/adapters/outbound/translation/openai.rs`
- Modify: `src-tauri/src/adapters/outbound/translation/mod.rs`

- [ ] **Step 1: Replace custom.rs type alias with full implementation from openai.rs**

Replace the entire content of `custom.rs` (currently just a type alias) with the full implementation from `openai.rs`, renaming `OpenAiTranslationProvider` to `CustomTranslationProvider` everywhere. The struct, impl blocks, helper structs (`OpenAiError` → `CustomError`), and test references all get renamed.

Key renames in the migrated content:
- `OpenAiTranslationProvider` → `CustomTranslationProvider`
- `OpenAiError` → `CustomError`
- In tests: `OpenAiTranslationProvider::new(config)` → `CustomTranslationProvider::new(config)`
- In tests: `engine: "openai".to_string()` → `engine: "custom".to_string()`
- Test name `http_error_message_uses_openai_error_fields` → `http_error_message_uses_custom_error_fields`

```rust
// custom.rs — full content after migration (abbreviated, key renames shown)
use crate::ports::outbound::translation::{...};

pub struct CustomTranslationProvider {
    client: Client,
    config: TranslationConfig,
}

// All internal structs remain unchanged (ChatCompletionRequest, ChatMessage, etc.)
// Only OpenAiError renamed:
#[derive(Debug, Deserialize)]
struct CustomError {
    message: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    code: Option<Value>,
}

impl CustomTranslationProvider {
    pub fn new(config: TranslationConfig) -> Result<Self, TranslationError> { ... }
    // ... all methods unchanged
}

impl TranslationProvider for CustomTranslationProvider { ... }
impl WordInsightProvider for CustomTranslationProvider { ... }
// ... all helper functions unchanged
// ... all tests renamed as described above
```

- [ ] **Step 2: Delete openai.rs**

```bash
rm src-tauri/src/adapters/outbound/translation/openai.rs
```

- [ ] **Step 3: Remove `pub mod openai;` from mod.rs**

In `src-tauri/src/adapters/outbound/translation/mod.rs`, remove line 4:

```rust
// Before:
pub mod custom;
pub mod deepl;
pub mod google;
pub mod openai;

// After:
pub mod custom;
pub mod deepl;
pub mod google;
```

- [ ] **Step 4: Verify Rust compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/adapters/outbound/translation/custom.rs src-tauri/src/adapters/outbound/translation/mod.rs
git rm src-tauri/src/adapters/outbound/translation/openai.rs
git commit -m "refactor: migrate OpenAiTranslationProvider into CustomTranslationProvider"
```

---

### Task 2: Rust — Update commands/translate.rs dispatch and defaults

**Files:**
- Modify: `src-tauri/src/commands/translate.rs`
- Modify: `src-tauri/src/commands/settings.rs`

- [ ] **Step 1: Update import in translate.rs**

Change line 3 from `openai::OpenAiTranslationProvider` to `custom::CustomTranslationProvider`:

```rust
// Before:
use crate::adapters::outbound::translation::{
    deepl::DeepLTranslationProvider, google::GoogleTranslationProvider,
    openai::OpenAiTranslationProvider,
};

// After:
use crate::adapters::outbound::translation::{
    custom::CustomTranslationProvider, deepl::DeepLTranslationProvider,
    google::GoogleTranslationProvider,
};
```

- [ ] **Step 2: Update create_translation_provider match**

Remove `"openai"` standalone case, keep `"openai" | "custom"` compatibility mapping, change provider reference:

```rust
// Before:
"openai" | "custom" => OpenAiTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),

// After:
"openai" | "custom" => CustomTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
    .map_err(|error| error.to_string()),
```

- [ ] **Step 3: Update create_word_insight_provider match**

Same change as Step 2:

```rust
// Before:
"openai" | "custom" => OpenAiTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn WordInsightProvider>)
    .map_err(|error| error.to_string()),

// After:
"openai" | "custom" => CustomTranslationProvider::new(config)
    .map(|provider| Box::new(provider) as Box<dyn WordInsightProvider>)
    .map_err(|error| error.to_string()),
```

- [ ] **Step 4: Change default values in load_translation_config**

```rust
// Before:
engine: setting_or_default(&settings, "translationEngine", "openai"),
api_endpoint: setting_or_default(&settings, "apiEndpoint", "https://api.openai.com/v1"),
api_key: setting_or_default(&settings, "apiKey", ""),
translation_model: setting_or_default(&settings, "translationModel", "gpt-4o-mini"),

// After:
engine: setting_or_default(&settings, "translationEngine", "custom"),
api_endpoint: setting_or_default(&settings, "apiEndpoint", ""),
api_key: setting_or_default(&settings, "apiKey", ""),
translation_model: setting_or_default(&settings, "translationModel", ""),
```

- [ ] **Step 5: Update seed_settings defaults in settings.rs**

```rust
// Before:
("translationEngine", "openai"),
("apiEndpoint", "https://api.openai.com/v1"),
("apiKey", ""),
("translationModel", "gpt-4o-mini"),

// After:
("translationEngine", "custom"),
("apiEndpoint", ""),
("apiKey", ""),
("translationModel", ""),
```

Note: `seed_settings` uses `INSERT OR IGNORE`, so existing user data won't be overwritten.

- [ ] **Step 6: Verify Rust compilation and tests**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors

Run: `cd src-tauri && cargo test`
Expected: All tests pass (including renamed tests in custom.rs)

Run: `cd src-tauri && cargo clippy -- -D warnings`
Expected: No lint warnings

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/translate.rs src-tauri/src/commands/settings.rs
git commit -m "refactor: update translation dispatch and defaults to custom engine"
```

---

### Task 3: Frontend — Update TranslationPanel component

**Files:**
- Modify: `src/pages/Settings/panels/TranslationPanel.tsx`

- [ ] **Step 1: Remove OpenAI engine option**

```typescript
// Before:
const engineOptionKeys = [
  { i18nKey: "engineDeepL", value: "deepl" },
  { i18nKey: "engineGoogle", value: "google" },
  { i18nKey: "engineOpenAI", value: "openai" },
  { i18nKey: "engineCustom", value: "custom" },
];

// After:
const engineOptionKeys = [
  { i18nKey: "engineDeepL", value: "deepl" },
  { i18nKey: "engineGoogle", value: "google" },
  { i18nKey: "engineCustom", value: "custom" },
];
```

- [ ] **Step 2: Change default engine and endpoint/model fallbacks**

Add data migration: if saved engine is `"openai"`, auto-map to `"custom"`.

```typescript
// Before:
const translationEngine = settings.translationEngine || "openai";
const apiEndpoint = settings.apiEndpoint || "https://api.openai.com/v1";
const apiKey = settings.apiKey || "";
const translationModel = settings.translationModel;

// After:
const rawEngine = settings.translationEngine || "custom";
const translationEngine = rawEngine === "openai" ? "custom" : rawEngine;
const apiEndpoint = settings.apiEndpoint || "";
const apiKey = settings.apiKey || "";
const translationModel = settings.translationModel || "";
```

Also add auto-save migration when `"openai"` is detected:

```typescript
// Add after the translationEngine derivation, before return statement:
if (rawEngine === "openai") {
  saveSetting("translationEngine", "custom");
}
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `npx tsc --noEmit`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/pages/Settings/panels/TranslationPanel.tsx
git commit -m "refactor: remove openai engine option, change defaults to empty"
```

---

### Task 4: i18n — Remove engineOpenAI and update engineCustom across 14 locale files

**Files:**
- Modify: `src/locales/*/common.json` (14 files: ar, de, en, es, fr, hi, id, ja, ko, pt, ru, th, vi, zh-CN, zh-TW)

- [ ] **Step 1: Update zh-CN**

```json
// Remove line with "engineOpenAI": "OpenAI"
// Change "engineCustom": "自定义" → "engineCustom": "自定义 (LLM)"
```

- [ ] **Step 2: Update en**

```json
// Remove line with "engineOpenAI": "OpenAI"
// Change "engineCustom": "Custom" → "engineCustom": "Custom (LLM)"
```

- [ ] **Step 3: Update zh-TW**

```json
// Remove line with "engineOpenAI": "OpenAI"
// Change "engineCustom": "自訂" → "engineCustom": "自訂 (LLM)"
```

- [ ] **Step 4: Update ja**

```json
// Remove line with "engineOpenAI": "OpenAI"
// Change "engineCustom": "カスタム" → "engineCustom": "カスタム (LLM)"
```

- [ ] **Step 5: Update ko**

```json
// Remove line with "engineOpenAI": "OpenAI"
// Change "engineCustom": "사용자 정의" → "engineCustom": "사용자 정의 (LLM)"
```

- [ ] **Step 6: Update remaining 9 locale files (ar, de, es, fr, hi, id, pt, ru, th, vi)**

Each: Remove `engineOpenAI` line, update `engineCustom` to append "(LLM)":

| Locale | Old value | New value |
|--------|-----------|-----------|
| ar | "مخصص" | "مخصص (LLM)" |
| de | "Benutzerdefiniert" | "Benutzerdefiniert (LLM)" |
| es | "Personalizado" | "Personalizado (LLM)" |
| fr | "Personnalisé" | "Personnalisé (LLM)" |
| hi | "कस्टम" | "कस्टम (LLM)" |
| id | "Kustom" | "Kustom (LLM)" |
| pt | "Personalizado" | "Personalizado (LLM)" |
| ru | "Пользовательский" | "Пользовательский (LLM)" |
| th | "กำหนดเอง" | "กำหนดเอง (LLM)" |
| vi | "Tùy chỉnh" | "Tùy chỉnh (LLM)" |

- [ ] **Step 7: Verify TypeScript compilation**

Run: `npx tsc --noEmit`
Expected: No errors (removed key should not be referenced anywhere)

- [ ] **Step 8: Commit**

```bash
git add src/locales/*/common.json
git commit -m "refactor: remove engineOpenAI i18n key, update engineCustom to include (LLM)"
```

---

### Verification

After all tasks are complete, run end-to-end verification:

1. **Rust build**: `cd src-tauri && cargo check` — no errors
2. **Rust lint**: `cd src-tauri && cargo clippy -- -D warnings` — no warnings
3. **Rust tests**: `cd src-tauri && cargo test` — all pass
4. **TypeScript**: `npx tsc --noEmit` — no errors
5. **App launch**: `npm run tauri dev` — settings page shows only 3 engine options (DeepL, Google, Custom (LLM))
6. **Custom engine config**: Select "Custom (LLM)" → endpoint/model/key fields appear with empty placeholders
7. **Dedicated API config**: Select DeepL or Google → only API Key field shown (endpoint/model fields are irrelevant)
8. **Data migration**: If previously saved `translationEngine = "openai"` → auto-mapped to `"custom"` on load