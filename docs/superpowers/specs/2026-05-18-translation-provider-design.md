# Bugoo 多引擎翻译与单词详情功能设计

日期：2026-05-18

## 目标

实现统一翻译能力，让前端只调用稳定 API，后端根据设置页配置选择具体翻译服务。同时支持划词翻译弹窗展示例句，并为单词详情页提供更完整的释义、例句、单词详情说明和记忆技巧。

## 范围

本次设计覆盖：

- 后端多引擎 `TranslationProvider` 架构
- 后端单词增强信息 `WordInsightProvider` 架构
- 设置项与 SQLite 存储字段
- `word_details` 独立详情表
- 划词弹窗所需的例句摘要
- 单词详情页所需的完整详情数据
- Prompt 文件外置管理
- 错误处理与第一版实现边界

本次不覆盖：

- 全局划词监听完整实现
- DeepL 和 Google 官方接口的完整接入
- 翻译历史、用量统计、费用统计
- 云端同步或多设备配置同步

## 推荐方案

采用两个后端能力：

1. `TranslationProvider`：负责快速翻译，返回主译文和可选摘要信息。
2. `WordInsightProvider`：负责生成结构化单词信息，包括例句、释义、词性、单词详情说明和记忆技巧。

`openai` 和 `custom` 第一版真实实现两个能力。`deepl` 和 `google` 第一版只预留翻译 adapter，并在未完整支持时返回明确错误。由于 DeepL / Google 不擅长生成记忆技巧和结构化详情，详情生成优先走大模型能力。

## 后端架构

建议目录结构：

```txt
src-tauri/src/
├── commands/
│   ├── translate.rs
│   └── word_details.rs
├── ports/outbound/
│   ├── translation.rs
│   └── word_insight.rs
└── adapters/outbound/translation/
    ├── mod.rs
    ├── deepl.rs
    ├── google.rs
    ├── openai.rs
    ├── custom.rs
    └── prompts/
        ├── system_translation_prompt.txt
        └── word_detail_prompt.txt
```

职责划分：

- `commands/translate.rs`：接收前端快速翻译调用，读取 settings，调用 provider，返回统一翻译结果。
- `commands/word_details.rs`：提供获取、生成、保存单词详情的 Tauri 命令。
- `ports/outbound/translation.rs`：定义 `TranslationProvider`、翻译请求结构、翻译返回结构、错误类型。
- `ports/outbound/word_insight.rs`：定义 `WordInsightProvider`、单词详情生成请求和结构化返回。
- `adapters/outbound/translation/openai.rs`：实现 OpenAI-compatible 翻译与单词详情生成。
- `adapters/outbound/translation/custom.rs`：复用 OpenAI-compatible 协议。
- `adapters/outbound/translation/deepl.rs`：保留 DeepL adapter 文件，第一版可返回明确的“暂未完整支持”错误。
- `adapters/outbound/translation/google.rs`：保留 Google adapter 文件，第一版可返回明确的“暂未完整支持”错误。
- `prompts/system_translation_prompt.txt`：保存系统级翻译提示词，避免把默认系统提示词硬编码在 Rust 代码中。
- `prompts/word_detail_prompt.txt`：保存单词详情生成提示词，要求大模型返回严格 JSON。

## 翻译返回结构

快速翻译接口返回主译文，并允许携带划词弹窗直接需要的摘要信息：

```ts
type TranslationResult = {
  translation: string;
  detectedSourceLang: string | null;
  phonetic: string | null;
  partOfSpeech: string[];
  definitions: string[];
  examples: TranslationExample[];
};

type TranslationExample = {
  sentence: string;
  translation: string;
};
```

划词弹窗展示：

- 主译文
- 音标
- 词性
- 简短释义
- 前 1-2 条例句

如果例句生成失败，主译文仍可展示，例句区域显示可重试状态。

## 单词详情结构

完整详情用于单词详情页和加入生词本后的复用：

```ts
type WordDetail = {
  wordId: string;
  word: string;
  translation: string;
  phonetic: string | null;
  partOfSpeech: string[];
  definitions: string[];
  examples: TranslationExample[];
  memoryTip: string;
  detail: string;
  provider: string;
  rawJson: string;
  createdAt: number;
  updatedAt: number;
};
```

单词详情页展示：

- 完整释义
- 多条例句
- 单词详情说明
- 记忆技巧

## SQLite 存储设计

继续使用现有 `words` 表存储核心列表数据和复习数据。

新增 `word_details` 表存储结构化详情：

```sql
CREATE TABLE IF NOT EXISTS word_details (
    word_id TEXT PRIMARY KEY,
    part_of_speech_json TEXT NOT NULL DEFAULT '[]',
    definitions_json TEXT NOT NULL DEFAULT '[]',
    examples_json TEXT NOT NULL DEFAULT '[]',
    memory_tip TEXT NOT NULL DEFAULT '',
    detail TEXT NOT NULL DEFAULT '',
    provider TEXT NOT NULL DEFAULT '',
    raw_json TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
);
```

设计理由：

- `words` 表保持轻量，便于首页列表快速查询。
- `word_details` 表承载详情页和弹窗增强信息。
- 例句不单独拆表，统一放在 `examples_json`，划词弹窗只读取前 1-2 条摘要。
- 未保存到生词本的划词结果可以先作为临时结果返回；用户点击加入生词本时，将同一份详情写入 `word_details`，避免二次生成。

## 设置项与 SQLite 配置

继续使用现有 `settings` 表的 key-value 结构。

需要规范或新增以下字段：

```txt
translationEngine     deepl | google | openai | custom
apiEndpoint           翻译服务接口地址
apiKey                翻译服务密钥
translationModel      大模型名称，同时用于快速翻译和单词详情生成
translationPrompt     自定义翻译提示词
wordDetailPrompt      用户自定义单词详情提示补充
translationTimeoutMs  请求超时时间，单位毫秒
```

默认值由 `seed_settings` 写入，使用 `INSERT OR IGNORE`，只在字段不存在时初始化，不覆盖已有配置。

建议默认值：

```txt
translationEngine = openai
apiEndpoint = https://api.openai.com/v1
apiKey = ""
translationModel = gpt-4o-mini
translationPrompt = ""
wordDetailPrompt = ""
translationTimeoutMs = 15000
```

## 前端设置页

`TranslationPanel` 继续使用当前设置页风格：

- `Card` 作为面板容器
- `Card.Header` / `Card.Title` 保留卡片标题
- `SettingItem` 展示标题与说明
- `Separator` 分隔设置项
- HeroUI 组件用法优先参考 `CLAUDE.md` 中的 HeroUI v3 文档说明

设置项：

- 翻译引擎：`Select`
- API 地址：`Input`
- API 密钥：`Input type="password"`
- 翻译模型：`Input`，在 `openai` / `custom` 引擎下显示
- 翻译自定义 Prompt：`TextArea`，在 `openai` / `custom` 引擎下显示
- 单词详情自定义 Prompt：`TextArea`
- 超时时间：`NumberField`

前端保存逻辑沿用当前 `updateSetting` 和 `setSetting` 方式。切换引擎时不清空已有字段，避免用户来回切换时丢失配置。

## 请求数据流

快速翻译：

```txt
划词触发
→ src/lib/api/translate.ts
→ invoke("translate_text")
→ commands/translate.rs 读取 settings
→ provider factory 根据 translationEngine 创建适配器
→ provider 请求外部翻译 API
→ 解析为 TranslationResult
→ 返回弹窗展示主译文和例句摘要
```

单词详情：

```txt
加入生词本或打开详情面板
→ invoke("generate_word_detail")
→ commands/word_details.rs 读取 settings 和 word
→ WordInsightProvider 请求大模型
→ 解析严格 JSON
→ 保存 word_details
→ 返回 WordDetail
```

前端 API：

```ts
translate(text, sourceLang, targetLang)
getWordDetail(wordId)
generateWordDetail(wordId)
saveWordDetail(wordId, detail)
```

## Prompt 管理

系统级翻译提示词放在：

```txt
src-tauri/src/adapters/outbound/translation/prompts/system_translation_prompt.txt
```

单词详情生成提示词放在：

```txt
src-tauri/src/adapters/outbound/translation/prompts/word_detail_prompt.txt
```

Rust 侧通过 `include_str!` 读取文件内容，代码中不直接写死提示词正文。

`word_detail_prompt.txt` 必须要求模型只返回 JSON，字段固定为：

```json
{
  "translation": "string",
  "phonetic": "string | null",
  "partOfSpeech": ["string"],
  "definitions": ["string"],
  "examples": [
    {
      "sentence": "string",
      "translation": "string"
    }
  ],
  "memoryTip": "string",
  "detail": "string"
}
```

## 错误处理

后端定义结构化错误，转换为中文可展示文案返回前端。

错误类型：

- `MissingApiKey`：请先在设置页填写 API 密钥。
- `MissingEndpoint`：请先在设置页填写 API 地址。
- `MissingModel`：请先填写模型名称。
- `UnsupportedEngine`：当前翻译引擎暂未完整支持。
- `RequestTimeout`：翻译服务请求超时，请稍后重试。
- `RequestFailed`：翻译服务请求失败。
- `InvalidResponse`：翻译服务返回格式异常。
- `InvalidJson`：单词详情返回格式异常。
- `EmptyText`：翻译文本不能为空。
- `WordNotFound`：单词不存在。

快速翻译的例句生成失败不应阻塞主译文展示；单词详情生成失败不应影响单词保存。

## 第一版实现边界

第一版真实可用：

- `openai` 翻译
- `custom` 翻译
- `openai/custom` 单词详情生成
- `word_details` 本地存储

第一版结构预留：

- `deepl`
- `google`

DeepL / Google 暂未完整支持时必须返回明确错误，不能返回假翻译结果。

## 验证方式

实现完成后需要验证：

- `pnpm tsc --noEmit`
- `pnpm build`
- `cd src-tauri && cargo build`
- `cd src-tauri && cargo test`
- 设置页能保存翻译引擎、API 地址、API 密钥、模型、翻译 Prompt、详情 Prompt、超时时间。
- 在填写 OpenAI-compatible 配置后，前端调用 `translate()` 能拿到真实译文和例句摘要。
- 加入生词本或打开详情面板时能生成并保存 `word_details`。
- 详情页能展示完整释义、例句、单词详情说明和记忆技巧。
- 未填写密钥或接口异常时，前端收到可读中文错误。