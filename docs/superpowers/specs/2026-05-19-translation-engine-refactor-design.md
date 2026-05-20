# Translation Engine Categorization Refactor

## Context

当前翻译引擎有 4 个独立选项：DeepL、Google、OpenAI、自定义。但 OpenAI 和"自定义"在 Rust 后端共享同一个 `OpenAiTranslationProvider`（`custom.rs` 只是 `openai.rs` 的 type alias），本质都是 LLM prompt 翻译，与 DeepL/Google 的专用翻译 API 是不同范式。将 OpenAI 合入"自定义"类别，使引擎分类更准确。

## Design

### Engine Categories

| 选项 | 值 | 范式 | 配置 |
|------|----|------|------|
| DeepL | `deepl` | 专用翻译 API | API Key |
| Google | `google` | 专用翻译 API | API Key |
| 自定义 (LLM) | `custom` | LLM prompt 翻译 | Endpoint + API Key + Model |

UI 为扁平选择：3 个选项，选中"自定义"后展开 endpoint/model/key/prompt 配置。默认值全部为空字符串，用户需手动填写。

### Changes

#### 1. Rust: `src-tauri/src/commands/translate.rs`

- `create_translation_provider`: 移除 `"openai"` 单独匹配，只保留 `"custom"`
- `create_word_insight_provider`: 同上
- `load_translation_config`: 默认值改为空字符串（engine: `"custom"`, endpoint: `""`, key: `""`, model: `""`）
- 已有 `"openai"` 值兼容：在 match 中 `"openai"` 映射到 `"custom"` 处理逻辑

#### 2. Rust: `src-tauri/src/adapters/outbound/translation/`

- 将 `openai.rs` 内容移入 `custom.rs`（不再是 type alias，而是独立实现）
- 删除 `openai.rs`
- `mod.rs` 移除 `pub mod openai;`
- 结构体命名为 `CustomTranslationProvider`（重命名自 `OpenAiTranslationProvider`）

#### 3. Frontend: `src/pages/Settings/panels/TranslationPanel.tsx`

- `engineOptionKeys` 移除 `{ i18nKey: "engineOpenAI", value: "openai" }`
- 默认 `translationEngine` 改为 `"custom"`
- 默认 `apiEndpoint`、`apiKey`、`translationModel` 改为空字符串

#### 4. i18n: all locale files

- 移除 `settings.translation.engineOpenAI` key
- `settings.translation.engineCustom` 文案更新：
  - en: `"Custom (LLM)"`
  - zh-CN: `"自定义 (LLM)"`
  - 其他语言同步更新

#### 5. Data Migration

- 后端 match 中 `"openai"` | `"custom"` 统一走 `CustomTranslationProvider`
- 前端检测到 `translationEngine === "openai"` 时自动映射为 `"custom"` 并保存
- 无需数据库迁移脚本，字符串兼容即可

### Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/commands/translate.rs` | 移除 openai case，默认值改为空，重命名 provider |
| `src-tauri/src/adapters/outbound/translation/custom.rs` | 从 openai.rs 迁移完整实现，重命名 struct |
| `src-tauri/src/adapters/outbound/translation/openai.rs` | 删除 |
| `src-tauri/src/adapters/outbound/translation/mod.rs` | 移除 `pub mod openai` |
| `src/pages/Settings/panels/TranslationPanel.tsx` | 移除 openai 选项，默认值改为空 |
| `src/locales/*/common.json` (14 files) | 移除 engineOpenAI，更新 engineCustom |

### Verification

1. `cd src-tauri && cargo check` — Rust 编译无错误
2. `cd src-tauri && cargo clippy -- -D warnings` — 无 lint 警告
3. `npx tsc --noEmit` — TypeScript 无错误
4. `npm run tauri dev` — 设置页面引擎选择只有 3 个选项
5. 选中"自定义"后 endpoint/model/key 配置区域正常显示
6. 选中 DeepL/Google 后配置区域条件隐藏（仅显示 API Key）
7. 之前保存 `translationEngine = "openai"` 的用户数据仍能正常工作